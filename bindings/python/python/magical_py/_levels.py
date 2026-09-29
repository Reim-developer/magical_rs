"""Custom detection rules: levels 2, 3 and 4 of ``magical_rs``.

Level 1 is :func:`magical_py.detect` and :func:`magical_py.detect_bytes`,
which read the crate's built-in table of 114 formats. This module is the other
three levels, the ones where you decide what a file is.

They are not three spellings of one thing. They differ in what a match costs
and in what a rule is allowed to carry.

**Level 2 is declarative.** A rule is signatures at offsets, optionally
combined with predicates. Matching one is a single Rust call with no Python in
it, which is the whole reason it is a separate level from the next. Its kind is
fixed when the rule is built, every rule in a set carries the same kind type,
and :func:`match_types_custom` answers with a ``fallback`` rather than
``None``, so a caller always gets a value.

**Level 3 hands the whole decision to a Python callable.** Every match crosses
into Python and runs your code. Kinds may differ between rules, and
:func:`match_dyn_types` answers ``None`` when nothing matches.

**Level 4 is level 3 with an awaitable matcher**, for a rule that has to ask
something: a network lookup, a database, a service call.

Level 5 of the crate, raw pointers, is deliberately absent. It is ``unsafe`` by
definition and has no Python counterpart worth shipping.

Where this differs from the crate, and why
----------------------------------------

**Level 2 does not call the crate's ``MagicCustom``.** That struct holds
``&'static`` slices, because a level 2 rule is meant to be a ``static``. A rule
assembled from Python data at run time would have to be ``Box::leak``ed, and a
process that builds rules in a loop would leak without bound. The comparison is
reproduced in Rust instead, and pinned to the crate's behaviour from both
sides: ``tests/magic_custom.rs`` in the crate, ``tests/test_levels.py`` here.

**Level 4 runs on your event loop.** The crate's ``AsyncDynMagic`` takes a
closure returning a future, and ``magical_rs`` depends on no async runtime at
all. A Python awaitable nearly always needs the caller's event loop, so a
matcher here is awaited where you await it. Polling it from Rust on a worker
thread would add a thread, a channel and a loop handle to do what
``await rule.matches(data)`` does in three characters, and would deadlock on
any matcher that touches the loop. Level 4 therefore needs no feature flag and
no extra dependency: there is nothing to switch on.

**Construction is checked.** The crate ignores a rule's ``signatures`` and
``offsets`` entirely when ``rules`` is set, because its fields are ``&'static``
and it cannot do better. Here that combination raises :exc:`ValueError`, because
a Python caller who set them meant to use them.

**No downcasting.** The crate returns ``&dyn Any`` and hands you
``kind_downcast_ref::<T>()``, because Rust has to recover a type it erased.
Python never erased it, so every rule here simply returns its ``kind``. That is
the reason a heterogeneous list of level 3 rules needs no cast at read time.
"""

from __future__ import annotations

import dataclasses
from typing import (
    Awaitable,
    Callable,
    Generic,
    Literal,
    Sequence,
    TypeVar,
)

from ._magical_rs import signatures_match

__all__ = [
    "AsyncDynMagic",
    "DynMagicCustom",
    "MagicCustom",
    "MatchRules",
    "match_async_dyn_types",
    "match_async_dyn_types_all",
    "match_dyn_types",
    "match_dyn_types_all",
    "match_types_custom",
    "match_types_custom_all",
]

# Invariant, for the free functions: the kind is both an input, through the
# rules, and an output, through the fallback or the return value.
K = TypeVar("K")

# Covariant, for the rule classes. Safe only because `kind` is read-only, which
# the frozen dataclass guarantees. That is what lets a list of
# `DynMagicCustom[str]` be passed where `Sequence[DynMagicCustom[object]]` is
# expected, so rules with different kinds can be matched in one call.
K_co = TypeVar("K_co", covariant=True)

#: A level 3 matcher: the whole decision, as a plain boolean.
Predicate = Callable[[bytes], bool]

#: A level 4 matcher: the same decision, but it may await.
AsyncPredicate = Callable[[bytes], Awaitable[bool]]

_Strategy = Literal["all", "any", "with_fn"]


@dataclasses.dataclass(frozen=True)
class MatchRules:
    """How a :class:`MagicCustom` combines predicates into one answer.

    Mirrors the crate's ``CustomMatchRules`` together with its three
    ``all_matches!``, ``any_matches!`` and ``with_fn_matches!`` macros. The
    crate spells them as macros because a macro can name a bare ``fn`` item; a
    predicate in Python is a value, so a constructor does the same job.

    ``all`` and ``any`` short-circuit, so a predicate after a decisive one is
    never called.
    """

    strategy: _Strategy
    predicates: tuple[Predicate, ...]

    def __post_init__(self) -> None:
        if not self.predicates:
            raise ValueError(
                f"MatchRules.{self.strategy} needs at least one predicate; "
                "the crate's macros take at least one too"
            )

    @classmethod
    def all(cls, *predicates: Predicate) -> MatchRules:
        """Every predicate must return ``True``, as in ``all_matches!``."""
        return cls("all", predicates)

    @classmethod
    def any(cls, *predicates: Predicate) -> MatchRules:
        """One predicate returning ``True`` is enough, as in ``any_matches!``."""
        return cls("any", predicates)

    @classmethod
    def with_fn(cls, predicate: Predicate) -> MatchRules:
        """A single predicate, as in ``with_fn_matches!``.

        The crate has a dedicated variant for this because a one-element slice
        and a bare ``fn`` are different types to it. In Python a one-tuple and a
        bare callable are the same shape, so ``with_fn(f)`` is ``all(f)`` and is
        spelled separately only to keep the crate's three names available.
        """
        return cls("with_fn", (predicate,))

    def evaluate(self, data: bytes) -> bool:
        """Apply the strategy to *data*, short-circuiting like the crate."""
        if self.strategy == "all":
            return all(predicate(data) for predicate in self.predicates)
        if self.strategy == "any":
            return any(predicate(data) for predicate in self.predicates)
        return self.predicates[0](data)


@dataclasses.dataclass(frozen=True)
class MagicCustom(Generic[K_co]):
    """A declarative rule: signatures at offsets, and optionally predicates.

    The crate's ``MagicCustom``. Level 2 is the level where you describe a
    format rather than hand over a function, and where matching costs one Rust
    call instead of a trip into Python.

    :param kind: What a match yields, carried through untouched. Any object.
    :param signatures: Byte strings to look for. Any one of them matching at
        any of *offsets* is a match.
    :param offsets: Where in the buffer each signature may start. Only used
        when *rules* is ``None``.
    :param max_bytes_read: How many bytes are worth reading for this rule.
        Advisory, exactly as in the crate: matching never enforces it, and it
        is the caller's job to read far enough to satisfy its own rules.
    :param rules: Predicates to apply instead of the signature comparison. When
        given, *signatures* and *offsets* must be left empty, because the crate
        ignores them and this binding would rather say so.

    :raises ValueError: If *rules* is given alongside signatures or offsets, if
        only one of signatures and offsets is given, if an offset is negative,
        or if *max_bytes_read* is negative.

    An empty *signatures* with empty *offsets* and no *rules* is legal and never
    matches. That is the crate's behaviour, not an oversight here: its arm is
    ``signatures.any(offsets.any(...))``, so either side being empty means no.
    """

    kind: K_co
    signatures: Sequence[bytes] = ()
    offsets: Sequence[int] = ()
    max_bytes_read: int = 0
    rules: MatchRules | None = None

    def __post_init__(self) -> None:
        if self.max_bytes_read < 0:
            raise ValueError(
                f"max_bytes_read cannot be negative, got {self.max_bytes_read}"
            )
        negative = [offset for offset in self.offsets if offset < 0]
        if negative:
            raise ValueError(f"offsets cannot be negative, got {negative}")

        if self.rules is not None:
            if self.signatures or self.offsets:
                raise ValueError(
                    "signatures and offsets are ignored when rules is set, so "
                    "leave both empty; the crate cannot say so because its "
                    "fields are 'static and it discards them silently"
                )
            return

        if bool(self.signatures) != bool(self.offsets):
            raise ValueError(
                "signatures and offsets go together: a rule needs both, and "
                "the crate's arm is signatures.any(offsets.any(...)), so one "
                "side alone can never match"
            )

        # The dataclass is frozen, so normalising the two sequences to tuples
        # has to go through `object.__setattr__`. That is the documented way to
        # do it, and it is worth the one line: it keeps a rule hashable and
        # keeps a caller who passes a list from mutating the rule afterwards.
        object.__setattr__(self, "signatures", tuple(self.signatures))
        object.__setattr__(self, "offsets", tuple(self.offsets))

    def matches(self, data: bytes) -> bool:
        """Whether *data* satisfies this rule.

        Never raises for content reasons. A predicate that raises propagates
        its exception, because a rule that cannot decide is not a rule that
        said no.
        """
        if self.rules is not None:
            return self.rules.evaluate(data)
        return signatures_match(data, self.signatures, self.offsets)


def match_types_custom(
    data: bytes, rules: Sequence[MagicCustom[K]], fallback: K
) -> K:
    """The ``kind`` of the first rule that matches, or *fallback*.

    The crate's ``match_types_custom``. First match wins, and the answer is
    never ``None``: a level 2 rule set is homogeneous and its caller is
    expected to have decided what "none of these" means.

    :param data: The bytes to test, normally a file header.
    :param rules: The rules to try, in priority order.
    :param fallback: What to return when no rule matches. Must be the same type
        as every rule's kind.
    """
    for rule in rules:
        if rule.matches(data):
            return rule.kind
    return fallback


def match_types_custom_all(data: bytes, rules: Sequence[MagicCustom[K]]) -> list[K]:
    """The ``kind`` of every rule that matches, in rule order.

    A rule set can match more than once: two rules for overlapping formats, or
    one rule listed twice. Both are kept, in the order the rules were given,
    because the caller is the only one who knows whether a second match is a
    contradiction or a description.
    """
    return [rule.kind for rule in rules if rule.matches(data)]


@dataclasses.dataclass(frozen=True)
class DynMagicCustom(Generic[K_co]):
    """A rule whose whole decision is a Python callable.

    The crate's ``DynMagicCustom``, for level 3. Use it when the format is not
    known when you write the code: a rule loaded from a config file, chosen by
    a plugin, or assembled in a loop.

    The trade against :class:`MagicCustom` is the cost of a match. Each call
    crosses from Rust into Python and runs your code, so a level 3 rule is
    markedly slower than the equivalent level 2 rule. Reach for level 2 for a
    format you know and level 3 for one you do not.

    :param matcher: Called with the whole buffer. Return a ``bool``.
    :param kind: What a match yields. Any object, and different rules may carry
        different kinds.
    :param max_bytes_read: How many bytes are worth reading. Advisory, as in
        the crate, and not enforced.

    Unlike :class:`MagicCustom` this takes no signatures or offsets: in the
    crate too, a level 3 rule has only a matcher, a kind and a size, because a
    closure can look wherever it likes.
    """

    matcher: Predicate
    kind: K_co
    max_bytes_read: int = 0

    def __post_init__(self) -> None:
        if self.max_bytes_read < 0:
            raise ValueError(
                f"max_bytes_read cannot be negative, got {self.max_bytes_read}"
            )

    def matches(self, data: bytes) -> bool:
        """Whether *data* satisfies this rule, as your matcher decides.

        Short-circuits are the caller's, not this method's: it calls
        *matcher* once and returns what it said.
        """
        return self.matcher(data)


def match_dyn_types(
    data: bytes, rules: Sequence[DynMagicCustom[K]]
) -> K | None:
    """The ``kind`` of the first rule that matches, or ``None``.

    The crate's ``match_dyn_types``. First match wins.

    Rules are tried in order and not concurrently, so a rule whose matcher is
    expensive or has side effects runs only if the rules before it declined.

    :param data: The bytes to test, normally a file header.
    :param rules: The rules to try, in priority order. Their kinds need not
        agree; the type parameter is whatever the rules' kinds join to.
    """
    for rule in rules:
        if rule.matches(data):
            return rule.kind
    return None


def match_dyn_types_all(data: bytes, rules: Sequence[DynMagicCustom[K]]) -> list[K]:
    """The ``kind`` of every rule that matches, in rule order.

    Unlike :func:`match_dyn_types`, this does not stop at the first match, so
    every rule's matcher runs even after one has matched.
    """
    return [rule.kind for rule in rules if rule.matches(data)]


@dataclasses.dataclass(frozen=True)
class AsyncDynMagic(Generic[K_co]):
    """A level 3 rule whose matcher may await.

    The crate's ``AsyncDynMagic``, for level 4. Use it when deciding requires
    something that takes time and something else: a checksum registry, a MIME
    database, a service call.

    :param matcher: Called with the whole buffer, returning an awaitable that
        yields a ``bool``. An ``async def`` function qualifies, and so does any
        other callable returning an awaitable, including a coroutine object you
        already have.
    :param kind: What a match yields. Any object.
    :param max_bytes_read: How many bytes are worth reading. Advisory.

    The matcher is awaited on the caller's event loop, not on a thread this
    library owns. See the module docstring for why.
    """

    matcher: AsyncPredicate
    kind: K_co
    max_bytes_read: int = 0

    def __post_init__(self) -> None:
        if self.max_bytes_read < 0:
            raise ValueError(
                f"max_bytes_read cannot be negative, got {self.max_bytes_read}"
            )

    async def matches(self, data: bytes) -> bool:
        """Whether *data* satisfies this rule, as your matcher decides.

        Propagates whatever the matcher raises. An awaitable that fails is a
        rule that could not decide, which is not the same as a rule that said
        no, and swallowing it would turn a broken lookup into a clean negative.
        """
        return await self.matcher(data)


async def match_async_dyn_types(
    data: bytes, rules: Sequence[AsyncDynMagic[K]]
) -> K | None:
    """The ``kind`` of the first rule that matches, or ``None``.

    The crate's ``async_dyn_magic::match_dyn_types`, renamed because a single
    Python namespace cannot hold the level 3 and level 4 versions under one
    name the way two Rust modules can.

    Rules are awaited in order and not concurrently. The first match returns
    immediately and the rules after it never run, so a matcher's latency is
    paid only while the rules before it declined. Awaiting them concurrently
    would be faster and would change the meaning: a slower rule could then win
    over a faster one that came later in the list.
    """
    for rule in rules:
        if await rule.matches(data):
            return rule.kind
    return None


async def match_async_dyn_types_all(
    data: bytes, rules: Sequence[AsyncDynMagic[K]]
) -> list[K]:
    """The ``kind`` of every rule that matches, in rule order.

    Every rule's matcher is awaited, so a rule that raises stops the search and
    propagates, rather than being reported as a non-match.
    """
    kinds: list[K] = []
    for rule in rules:
        if await rule.matches(data):
            kinds.append(rule.kind)
    return kinds
