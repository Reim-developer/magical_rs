"""Tests for the custom detection levels, 2, 3 and 4.

The level 2 signature cases below are the same table as
``tests/magic_custom.rs`` in the crate, branch for branch, on purpose. That
crate test is the definition and this one is the copy; running both means a
change to the crate's ``CustomMatchRules::Default`` arm shows up here as a
failure instead of as a detection that quietly stops working.

Every fixture is a handful of literal bytes. There is no file to check in and
nothing to keep in step, and a failing case names the exact byte string that
broke rather than pointing at a binary.
"""

from __future__ import annotations

import asyncio
import dataclasses
import enum
from typing import Awaitable, Coroutine, TypeVar

import pytest

from magical_py import (
    AsyncDynMagic,
    DynMagicCustom,
    FileKind,
    MagicCustom,
    MatchRules,
    Predicate,
    detect_bytes,
    match_async_dyn_types,
    match_async_dyn_types_all,
    match_dyn_types,
    match_dyn_types_all,
    match_types_custom,
    match_types_custom_all,
)
from magical_py import _magical_rs

T = TypeVar("T")

ACAD = b"ACAD"
MAGIC = b"MAGIC"

# Real headers rather than invented ones, so a rule built from these is the
# rule a user would actually write. The PNG one is eight bytes at offset 0,
# which is the whole point of level 2 being cheap.
PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
GIF87A = b"GIF87a"
GIF89A = b"GIF89a"


def run(coro: Coroutine[object, object, T]) -> T:
    """Run *coro* on a fresh loop, as a caller with no event loop would."""
    return asyncio.run(coro)


# ---------------------------------------------------------------------------
# Level 2: the signature comparison
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("data", "signatures", "offsets", "expected"),
    [
        pytest.param(b"ACAD\0\0", [ACAD], [0], True, id="offset-zero"),
        pytest.param(b"nope----", [ACAD], [0], False, id="no-match"),
        pytest.param(b"yyACADzzzz", [ACAD], [2], True, id="positive-offset-room-after"),
        pytest.param(b"xxxxACAD", [ACAD], [4], True, id="offset-touching-the-end"),
        pytest.param(b"xxxxACAD", [ACAD], [5], False, id="offset-past-the-end"),
        pytest.param(b"xxxxACAD", [ACAD], [99], False, id="offset-far-past-the-end"),
        pytest.param(b"", [ACAD], [0], False, id="empty-input"),
        pytest.param(b"MAGIC", [ACAD, MAGIC], [0], True, id="second-signature"),
        pytest.param(b"MAGIC", [MAGIC], [99, 0], True, id="second-offset"),
        pytest.param(b"__ACAD", [ACAD, MAGIC], [0, 2], True, id="right-pair-of-each"),
        pytest.param(b"MAGIC", [ACAD], [0, 1], False, id="no-pair-matches"),
        pytest.param(b"xxxACA", [ACAD], [3], False, id="one-byte-short-at-offset"),
        pytest.param(b"ACA", [ACAD], [0], False, id="one-byte-short-at-zero"),
    ],
)
def test_signature_matching(
    data: bytes, signatures: list[bytes], offsets: list[int], expected: bool
) -> None:
    """Every branch of the comparison the crate's ``Default`` arm performs."""
    rule = MagicCustom("k", signatures, offsets)
    assert rule.matches(data) is expected


def test_an_empty_side_of_the_comparison_never_matches() -> None:
    """The crate's behaviour, reachable only through the primitive.

    Its arm is ``signatures.any(offsets.any(...))``, so an empty side can never
    match, whatever the other side holds. ``MagicCustom`` will not express that
    state: it raises, because a caller who passed signatures and no offsets has
    made a mistake worth naming. The comparison underneath is pinned here so
    the two are known to be the same comparison.
    """
    assert _magical_rs.signatures_match(ACAD, [], [0]) is False
    assert _magical_rs.signatures_match(ACAD, [ACAD], []) is False
    assert _magical_rs.signatures_match(ACAD, [], []) is False


def test_an_empty_signature_matches_at_offset_zero() -> None:
    """A consequence of the comparison rather than a rule anyone wants.

    An empty signature is a zero-length window, and a zero-length window is
    present at any valid offset. The crate does this too; it is pinned on both
    sides so a fix in one is visible in the other.
    """
    assert _magical_rs.signatures_match(b"anything", [b""], [0]) is True


def test_a_nonsensical_offset_reports_no_match_rather_than_panicking() -> None:
    """The one deliberate difference from the crate.

    ``match_types_custom`` computes ``offset + signature.len()`` and panics
    when that overflows, so an offset anywhere near ``usize::MAX`` aborts the
    process. An offset here is a Python integer, which can be arbitrarily large
    and is not the crate's to trust, so the addition saturates and a signature
    that cannot possibly be there reports no match. The crate has no test for
    this, because there the answer is a panic rather than a result.
    """
    assert MagicCustom("k", [ACAD], [2**64 - 1]).matches(b"xxxxACAD") is False


def test_a_matcher_receives_the_whole_buffer() -> None:
    """Predicates get everything the caller passed, not a truncated header.

    The crate's ``max_bytes_read`` is a hint to the reader, and a caller who
    hands over fewer bytes than a rule needs gets fewer bytes. Nothing here
    shortens the buffer on its own.
    """
    seen: list[bytes] = []

    def record(data: bytes) -> bool:
        seen.append(data)
        return True

    data = b"header" + b"x" * 4096
    assert MagicCustom("k", rules=MatchRules.with_fn(record)).matches(data) is True
    assert seen == [data]


def test_a_predicate_sees_empty_input() -> None:
    seen: list[bytes] = []

    def record(data: bytes) -> bool:
        seen.append(data)
        return True

    MagicCustom("k", rules=MatchRules.with_fn(record)).matches(b"")
    assert seen == [b""]


# ---------------------------------------------------------------------------
# Level 2: predicate strategies
# ---------------------------------------------------------------------------


def test_all_requires_every_predicate() -> None:
    yes: Predicate = lambda data: True
    no: Predicate = lambda data: False
    assert MatchRules.all(yes, yes).evaluate(PNG) is True
    assert MatchRules.all(yes, no).evaluate(PNG) is False
    assert MatchRules.all(no, yes).evaluate(PNG) is False


def test_any_requires_one_predicate() -> None:
    yes: Predicate = lambda data: True
    no: Predicate = lambda data: False
    assert MatchRules.any(no, yes).evaluate(PNG) is True
    assert MatchRules.any(yes, no).evaluate(PNG) is True
    assert MatchRules.any(no, no).evaluate(PNG) is False


def test_with_fn_is_a_single_predicate() -> None:
    yes: Predicate = lambda data: True
    no: Predicate = lambda data: False
    assert MatchRules.with_fn(yes).evaluate(PNG) is True
    assert MatchRules.with_fn(no).evaluate(PNG) is False


def test_all_stops_at_the_first_false_predicate() -> None:
    calls: list[str] = []

    def first(data: bytes) -> bool:
        calls.append("first")
        return False

    def second(data: bytes) -> bool:
        calls.append("second")
        return True

    assert MatchRules.all(first, second).evaluate(PNG) is False
    assert calls == ["first"], "the second predicate ran after the first said no"


def test_any_stops_at_the_first_true_predicate() -> None:
    calls: list[str] = []

    def first(data: bytes) -> bool:
        calls.append("first")
        return True

    def second(data: bytes) -> bool:
        calls.append("second")
        return False

    assert MatchRules.any(first, second).evaluate(PNG) is True
    assert calls == ["first"], "the second predicate ran after the first said yes"


def test_a_strategy_needs_at_least_one_predicate() -> None:
    with pytest.raises(ValueError, match="at least one predicate"):
        MatchRules("all", ())


# ---------------------------------------------------------------------------
# Level 2: match_types_custom
# ---------------------------------------------------------------------------


def test_match_types_custom_returns_the_matching_rule_kind() -> None:
    rules = [
        MagicCustom("gif", [GIF87A, GIF89A], [0]),
        MagicCustom("png", [PNG], [0]),
    ]
    assert match_types_custom(PNG, rules, "fallback") == "png"
    assert match_types_custom(GIF89A, rules, "fallback") == "gif"


def test_match_types_custom_takes_the_first_match() -> None:
    """First match wins, so rule order is priority order."""
    rules = [
        MagicCustom("first", [PNG], [0]),
        MagicCustom("second", [PNG], [0]),
    ]
    assert match_types_custom(PNG, rules, "fallback") == "first"


def test_match_types_custom_returns_the_fallback_when_nothing_matches() -> None:
    rules = [MagicCustom("png", [PNG], [0])]
    assert match_types_custom(b"nothing here", rules, "fallback") == "fallback"


def test_match_types_custom_returns_the_fallback_for_no_rules() -> None:
    assert match_types_custom(PNG, [], "fallback") == "fallback"


def test_the_fallback_is_returned_unchanged() -> None:
    """It comes back as the same object, not a copy or a repr of it."""
    marker = object()
    assert match_types_custom(PNG, [], marker) is marker

    @dataclasses.dataclass(frozen=True)
    class Fallback:
        reason: str

    dataclass_fallback = Fallback(reason="nothing matched")
    assert match_types_custom(PNG, [], dataclass_fallback) is dataclass_fallback

    assert match_types_custom(PNG, [], None) is None
    assert match_types_custom(PNG, [], (1, 2)) == (1, 2)


def test_a_rules_matcher_is_not_run_once_a_signature_rule_has_matched() -> None:
    """The reason first match wins is that later rules cost nothing to skip."""
    calls: list[bytes] = []

    def count(data: bytes) -> bool:
        calls.append(data)
        return True

    rules: list[MagicCustom[str]] = [
        MagicCustom("cheap", [PNG], [0]),
        MagicCustom("expensive", rules=MatchRules.with_fn(count)),
    ]
    assert match_types_custom(PNG, rules, "fallback") == "cheap"
    assert calls == []


def test_match_types_custom_all_keeps_rule_order() -> None:
    rules = [
        MagicCustom("png", [PNG], [0]),
        MagicCustom("gif", [GIF89A], [0]),
        MagicCustom("also-png", [PNG], [0]),
    ]
    assert match_types_custom_all(PNG, rules) == ["png", "also-png"]


def test_match_types_custom_all_reports_a_rule_listed_twice() -> None:
    """Two matches are a fact about the rules, not an error to tidy away."""
    once = MagicCustom("png", [PNG], [0])
    assert match_types_custom_all(PNG, [once, once]) == ["png", "png"]


def test_match_types_custom_all_is_empty_when_nothing_matches() -> None:
    assert match_types_custom_all(PNG, [MagicCustom("gif", [GIF89A], [0])]) == []


def test_match_types_custom_all_runs_every_rule() -> None:
    """Unlike the first-match form, this does not stop early."""
    calls: list[bytes] = []

    def count(data: bytes) -> bool:
        calls.append(data)
        return True

    rules = [MagicCustom("a", [PNG], [0]), MagicCustom("b", rules=MatchRules.with_fn(count))]
    assert match_types_custom_all(PNG, rules) == ["a", "b"]
    assert calls == [PNG]


# ---------------------------------------------------------------------------
# Level 2: construction
# ---------------------------------------------------------------------------


def test_a_rule_with_rules_may_not_carry_signatures() -> None:
    """The crate ignores the signatures here; this says so instead.

    Its fields are ``&'static`` and it cannot report the mistake usefully, so
    it drops them. A Python caller who set them meant to use them.
    """
    with pytest.raises(ValueError, match="rules is set"):
        MagicCustom("k", [ACAD], [], rules=MatchRules.with_fn(lambda d: True))


def test_a_rule_with_rules_may_not_carry_offsets() -> None:
    with pytest.raises(ValueError, match="rules is set"):
        MagicCustom("k", [], [0], rules=MatchRules.with_fn(lambda d: True))


def test_signatures_without_offsets_is_rejected() -> None:
    with pytest.raises(ValueError, match="go together"):
        MagicCustom("k", [ACAD], [])


def test_offsets_without_signatures_is_rejected() -> None:
    with pytest.raises(ValueError, match="go together"):
        MagicCustom("k", [], [0])


def test_a_negative_offset_is_rejected() -> None:
    with pytest.raises(ValueError, match="cannot be negative"):
        MagicCustom("k", [ACAD], [0, -1])


def test_a_negative_max_bytes_read_is_rejected() -> None:
    with pytest.raises(ValueError, match="cannot be negative"):
        MagicCustom("k", [ACAD], [0], max_bytes_read=-1)


def test_an_empty_rule_is_legal_and_never_matches() -> None:
    """The crate's behaviour, not an oversight here.

    Its arm is ``signatures.any(offsets.any(...))``, so an empty side means no,
    whichever way round it is. A rule that never matches is how a caller
    disables a format without rebuilding the list.
    """
    rule = MagicCustom("k")
    assert rule.signatures == ()
    assert rule.offsets == ()
    assert rule.max_bytes_read == 0
    assert rule.rules is None
    assert rule.matches(PNG) is False
    assert rule.matches(b"") is False


def test_a_rule_is_immutable() -> None:
    rule = MagicCustom("k", [ACAD], [0])
    with pytest.raises(dataclasses.FrozenInstanceError):
        rule.kind = "other"  # type: ignore[misc]
    with pytest.raises(dataclasses.FrozenInstanceError):
        rule.signatures = [MAGIC]  # type: ignore[misc]
    with pytest.raises(dataclasses.FrozenInstanceError):
        rule.offsets = [1]  # type: ignore[misc]
    with pytest.raises(dataclasses.FrozenInstanceError):
        rule.max_bytes_read = 10  # type: ignore[misc]


def test_a_rule_normalises_its_sequences() -> None:
    """A caller who passes a list cannot change the rule afterwards."""
    signatures = [ACAD]
    offsets = [0]
    rule = MagicCustom("k", signatures, offsets)
    assert rule.signatures == (ACAD,)
    assert rule.offsets == (0,)
    signatures.append(MAGIC)
    offsets.append(99)
    assert rule.signatures == (ACAD,)
    assert rule.offsets == (0,)


def test_max_bytes_read_is_advisory_and_never_enforced() -> None:
    """The crate treats it as a hint to the reader, and so does this.

    Nothing truncates the buffer to it, so a rule can match a signature past
    its own ``max_bytes_read`` and a predicate sees the whole buffer regardless
    of what the rule claims to need.
    """
    deep = b"..........ACAD"
    assert MagicCustom("k", [ACAD], [10], max_bytes_read=4).matches(deep) is True

    seen: list[bytes] = []

    def record(data: bytes) -> bool:
        seen.append(data)
        return True

    MagicCustom("k", rules=MatchRules.with_fn(record), max_bytes_read=1).matches(deep)
    assert seen == [deep]


# ---------------------------------------------------------------------------
# Kinds are passed through untouched
# ---------------------------------------------------------------------------


class Colour(enum.Enum):
    RED = "red"
    BLUE = "blue"


@dataclasses.dataclass(frozen=True)
class Kind:
    label: str
    weight: int


KINDS: list[object] = ["a string", 42, None, (1, 2), Colour.BLUE, Kind("x", 1), FileKind.Png]


@pytest.mark.parametrize("kind", KINDS)
def test_a_kind_crosses_unchanged(kind: object) -> None:
    """No coercion, no comparison, no requirement that it be a string."""
    rule = MagicCustom(kind, [PNG], [0])
    assert rule.matches(PNG) is True
    assert rule.kind is kind
    assert match_types_custom(PNG, [rule], None) is kind
    assert match_types_custom_all(PNG, [rule]) == [kind]

    dyn = DynMagicCustom(lambda data: True, kind)
    assert match_dyn_types(PNG, [dyn]) is kind
    assert match_dyn_types_all(PNG, [dyn]) == [kind]

    async def matcher(data: bytes) -> bool:
        return True

    assert run(match_async_dyn_types(PNG, [AsyncDynMagic(matcher, kind)])) is kind
    assert run(match_async_dyn_types_all(PNG, [AsyncDynMagic(matcher, kind)])) == [kind]


# ---------------------------------------------------------------------------
# Level 3: runtime matchers
# ---------------------------------------------------------------------------


def test_level_3_matcher_decides() -> None:
    assert DynMagicCustom(lambda data: True, "k").matches(PNG) is True
    assert DynMagicCustom(lambda data: False, "k").matches(PNG) is False


def test_level_3_matcher_receives_the_whole_buffer() -> None:
    seen: list[bytes] = []

    def record(data: bytes) -> bool:
        seen.append(data)
        return True

    data = b"header" + b"x" * 4096
    DynMagicCustom(record, "k").matches(data)
    assert seen == [data]


def test_match_dyn_types_returns_the_kind() -> None:
    rules = [
        DynMagicCustom(lambda data: data.startswith(b"GI"), "gif"),
        DynMagicCustom(lambda data: data.startswith(b"\x89P"), "png"),
    ]
    assert match_dyn_types(PNG, rules) == "png"
    assert match_dyn_types(GIF87A, rules) == "gif"


def test_match_dyn_types_is_none_when_nothing_matches() -> None:
    rules = [DynMagicCustom(lambda data: False, "png")]
    assert match_dyn_types(PNG, rules) is None


def test_match_dyn_types_takes_the_first_match() -> None:
    rules = [
        DynMagicCustom(lambda data: True, "first"),
        DynMagicCustom(lambda data: True, "second"),
    ]
    assert match_dyn_types(PNG, rules) == "first"


def test_match_dyn_types_is_none_for_no_rules() -> None:
    no_rules: list[DynMagicCustom[str]] = []
    assert match_dyn_types(PNG, no_rules) is None


def test_match_dyn_types_all_keeps_order_and_duplicates() -> None:
    yes = DynMagicCustom(lambda data: True, "yes")
    no = DynMagicCustom(lambda data: False, "no")
    assert match_dyn_types_all(PNG, [no, yes, yes, no]) == ["yes", "yes"]


def test_match_dyn_types_all_is_empty_for_no_rules() -> None:
    no_rules: list[DynMagicCustom[str]] = []
    assert match_dyn_types_all(PNG, no_rules) == []


def test_a_level_3_matcher_runs_once_and_only_as_far_as_it_must() -> None:
    """Rules are tried in order, not concurrently, and stop at the first match."""
    calls: list[str] = []

    def counted(name: str, answer: bool) -> DynMagicCustom[str]:
        def matcher(data: bytes) -> bool:
            calls.append(name)
            return answer

        return DynMagicCustom(matcher, name)

    rules = [counted("a", False), counted("b", True), counted("c", True)]
    assert match_dyn_types(PNG, rules) == "b"
    assert calls == ["a", "b"], "a rule after the winner must not run"


def test_a_level_3_matcher_runs_for_every_rule_in_the_all_form() -> None:
    calls: list[str] = []

    def counted(name: str) -> DynMagicCustom[str]:
        def matcher(data: bytes) -> bool:
            calls.append(name)
            return True

        return DynMagicCustom(matcher, name)

    assert match_dyn_types_all(PNG, [counted("a"), counted("b")]) == ["a", "b"]
    assert calls == ["a", "b"]


def test_a_level_3_matcher_exception_propagates() -> None:
    """A rule that cannot decide did not say no, and must not look like it did."""

    class Boom(Exception):
        pass

    def explode(data: bytes) -> bool:
        raise Boom("no answer")

    rules = [DynMagicCustom(lambda data: False, "ok"), DynMagicCustom(explode, "boom")]
    with pytest.raises(Boom, match="no answer"):
        match_dyn_types(PNG, rules)

    with pytest.raises(Boom, match="no answer"):
        DynMagicCustom(explode, "boom").matches(PNG)


def test_a_level_3_matcher_may_be_stateful() -> None:
    """A callable is a callable: it can remember, and be a closure over state."""
    seen: list[bytes] = []

    def matcher(data: bytes) -> bool:
        seen.append(data)
        return len(seen) == 2

    rule = DynMagicCustom(matcher, "second-call")
    assert rule.matches(PNG) is False
    assert rule.matches(PNG) is True
    assert rule.matches(PNG) is False
    assert seen == [PNG, PNG, PNG]


def test_a_level_3_matcher_may_be_any_callable() -> None:
    """Not only a function: the annotation says callable, and it is one."""
    calls: list[bytes] = []

    class Matcher:
        def __call__(self, data: bytes) -> bool:
            calls.append(data)
            return True

    assert DynMagicCustom(Matcher(), "k").matches(PNG) is True
    assert calls == [PNG]


def test_level_3_max_bytes_read_is_advisory() -> None:
    seen: list[bytes] = []

    def record(data: bytes) -> bool:
        seen.append(data)
        return True

    deep = b"..........ACAD"
    DynMagicCustom(record, "k", max_bytes_read=1).matches(deep)
    assert seen == [deep]


def test_level_3_rejects_a_negative_max_bytes_read() -> None:
    with pytest.raises(ValueError, match="cannot be negative"):
        DynMagicCustom(lambda data: True, "k", max_bytes_read=-1)


# ---------------------------------------------------------------------------
# Level 4: awaitable matchers
# ---------------------------------------------------------------------------


def test_rules_of_different_kinds_match_in_one_call() -> None:
    """Covariance is what makes this type-check: the kinds are not one type.

    The crate reaches the same place with ``&dyn Any`` and
    ``kind_downcast_ref::<T>()``, because Rust erased the type and has to get it
    back. Python never erased it, so a heterogeneous list needs no cast at read
    time. The annotation on ``rules`` is the assertion, and pyright is what
    checks it.
    """
    rules: list[DynMagicCustom[object]] = [
        DynMagicCustom(lambda data: True, "a string"),
        DynMagicCustom(lambda data: True, 42),
        DynMagicCustom(lambda data: True, None),
    ]
    assert match_dyn_types(PNG, rules) == "a string"
    assert match_dyn_types_all(PNG, rules) == ["a string", 42, None]


def test_async_rules_of_different_kinds_match_in_one_call() -> None:
    async def yes(data: bytes) -> bool:
        return True

    rules: list[AsyncDynMagic[object]] = [
        AsyncDynMagic(yes, "a string"),
        AsyncDynMagic(yes, 42),
    ]
    assert run(match_async_dyn_types(PNG, rules)) == "a string"
    assert run(match_async_dyn_types_all(PNG, rules)) == ["a string", 42]


def test_level_4_matcher_decides() -> None:
    async def yes(data: bytes) -> bool:
        return True

    async def no(data: bytes) -> bool:
        return False

    assert run(AsyncDynMagic(yes, "k").matches(PNG)) is True
    assert run(AsyncDynMagic(no, "k").matches(PNG)) is False


def test_level_4_matcher_receives_the_whole_buffer() -> None:
    seen: list[bytes] = []

    async def record(data: bytes) -> bool:
        seen.append(data)
        return True

    data = b"header" + b"x" * 4096
    run(AsyncDynMagic(record, "k").matches(data))
    assert seen == [data]


def test_level_4_matcher_really_suspends() -> None:
    """Awaiting means suspending, not a synchronous call wearing a coroutine.

    The matcher here cannot finish before ``other`` runs, because it waits on a
    gate that only ``other`` opens. So if the order comes out as it does, the
    matcher genuinely gave the loop a turn in the middle, which is the whole
    difference between level 4 and level 3 with a slow body.
    """
    order: list[str] = []

    async def main() -> None:
        gate = asyncio.Event()

        async def matcher(data: bytes) -> bool:
            order.append("matcher start")
            await gate.wait()
            order.append("matcher end")
            return True

        async def other() -> None:
            await asyncio.sleep(0)  # let the matcher reach its wait first
            order.append("other")
            gate.set()

        task = asyncio.ensure_future(other())
        kind = await match_async_dyn_types(PNG, [AsyncDynMagic(matcher, "k")])
        await task
        assert kind == "k"

    run(main())
    assert order == ["matcher start", "other", "matcher end"]


def test_level_4_runs_on_the_callers_loop() -> None:
    """No hidden thread and no loop of our own: it is the caller's loop.

    The crate takes a closure returning a future and depends on no async
    runtime. A Python awaitable nearly always needs the loop it was called from,
    so that is the loop it runs on. This is why level 4 needs no feature flag
    and no extra dependency, unlike the crate's own async module.
    """
    seen: list[asyncio.AbstractEventLoop] = []

    async def matcher(data: bytes) -> bool:
        seen.append(asyncio.get_running_loop())
        return True

    async def main() -> None:
        running = asyncio.get_running_loop()
        await match_async_dyn_types(PNG, [AsyncDynMagic(matcher, "k")])
        assert seen == [running]

    run(main())


def test_level_4_takes_the_first_match() -> None:
    calls: list[str] = []

    def counted(name: str, answer: bool) -> AsyncDynMagic[str]:
        async def matcher(data: bytes) -> bool:
            calls.append(name)
            return answer

        return AsyncDynMagic(matcher, name)

    rules = [counted("a", False), counted("b", True), counted("c", True)]
    assert run(match_async_dyn_types(PNG, rules)) == "b"
    assert calls == ["a", "b"], "a rule after the winner must not run"


def test_level_4_is_none_when_nothing_matches() -> None:
    async def no(data: bytes) -> bool:
        return False

    assert run(match_async_dyn_types(PNG, [AsyncDynMagic(no, "k")])) is None


def test_level_4_is_none_for_no_rules() -> None:
    no_rules: list[AsyncDynMagic[str]] = []
    assert run(match_async_dyn_types(PNG, no_rules)) is None


def test_level_4_all_keeps_order_and_duplicates() -> None:
    def counted(name: str, answer: bool) -> AsyncDynMagic[str]:
        async def matcher(data: bytes) -> bool:
            return answer

        return AsyncDynMagic(matcher, name)

    rules = [counted("yes", True), counted("no", False), counted("yes-again", True)]
    assert run(match_async_dyn_types_all(PNG, rules)) == ["yes", "yes-again"]
    no_rules: list[AsyncDynMagic[str]] = []
    assert run(match_async_dyn_types_all(PNG, no_rules)) == []


def test_level_4_runs_every_matcher_in_the_all_form() -> None:
    calls: list[str] = []

    def counted(name: str) -> AsyncDynMagic[str]:
        async def matcher(data: bytes) -> bool:
            calls.append(name)
            return True

        return AsyncDynMagic(matcher, name)

    run(match_async_dyn_types_all(PNG, [counted("a"), counted("b")]))
    assert calls == ["a", "b"]


def test_a_level_4_matcher_exception_propagates_unchanged() -> None:
    class Boom(Exception):
        pass

    async def explode(data: bytes) -> bool:
        raise Boom("lookup failed")

    with pytest.raises(Boom, match="lookup failed"):
        run(AsyncDynMagic(explode, "k").matches(PNG))

    rules = [AsyncDynMagic(explode, "boom")]
    with pytest.raises(Boom, match="lookup failed"):
        run(match_async_dyn_types(PNG, rules))
    with pytest.raises(Boom, match="lookup failed"):
        run(match_async_dyn_types_all(PNG, rules))


def test_a_level_4_matcher_may_be_stateful() -> None:
    seen: list[bytes] = []

    async def matcher(data: bytes) -> bool:
        seen.append(data)
        return len(seen) == 2

    rule = AsyncDynMagic(matcher, "second-call")
    assert run(rule.matches(PNG)) is False
    assert run(rule.matches(PNG)) is True
    assert seen == [PNG, PNG]


def test_a_level_4_matcher_may_be_any_callable_returning_an_awaitable() -> None:
    """Not only an ``async def``: any callable producing an awaitable will do."""

    class Matcher:
        def __call__(self, data: bytes) -> Awaitable[bool]:
            async def decide() -> bool:
                return data.startswith(PNG)

            return decide()

    assert run(AsyncDynMagic(Matcher(), "k").matches(PNG)) is True


def test_a_pre_made_coroutine_is_not_a_matcher() -> None:
    """It is a single-use awaitable, and a rule is not single use.

    A rule gets matched against many buffers. An awaitable that was built when
    the rule was built cannot be awaited again, so accepting one would turn a
    reused rule into a rule that fails on its second call, with a confusing
    message from the event loop. Requiring a callable moves that to the point
    where it is written.
    """

    async def decide(data: bytes) -> bool:
        return True

    pending = decide(PNG)
    rule = AsyncDynMagic(pending, "k")  # type: ignore[arg-type]
    try:
        with pytest.raises(TypeError, match="not callable"):
            run(rule.matches(PNG))
    finally:
        # Never awaited, so without this it warns when the frame is collected.
        pending.close()


def test_level_4_rejects_a_negative_max_bytes_read() -> None:
    async def yes(data: bytes) -> bool:
        return True

    with pytest.raises(ValueError, match="cannot be negative"):
        AsyncDynMagic(yes, "k", max_bytes_read=-1)


def test_level_4_max_bytes_read_is_advisory() -> None:
    seen: list[bytes] = []

    async def record(data: bytes) -> bool:
        seen.append(data)
        return True

    deep = b"..........ACAD"
    run(AsyncDynMagic(record, "k", max_bytes_read=1).matches(deep))
    assert seen == [deep]


# ---------------------------------------------------------------------------
# The three levels are three spellings of one thing
# ---------------------------------------------------------------------------


def test_the_three_levels_agree_on_the_same_condition() -> None:
    """Same predicate, three levels, three ways to spell it, one answer.

    They cost different amounts, which is why they are separate. They are not
    supposed to disagree about the result, and this is what says so.
    """
    for data, expected in [(PNG, True), (GIF89A, False), (b"", False)]:

        def is_png(buffer: bytes) -> bool:
            return buffer.startswith(PNG)

        level_2 = match_types_custom(
            data, [MagicCustom("l2", rules=MatchRules.with_fn(is_png))], None
        )
        level_3 = match_dyn_types(data, [DynMagicCustom(is_png, "l3")])

        async def matcher(buffer: bytes) -> bool:
            return is_png(buffer)

        level_4 = run(match_async_dyn_types(data, [AsyncDynMagic(matcher, "l4")]))

        assert level_2 == ("l2" if expected else None)
        assert level_3 == ("l3" if expected else None)
        assert level_4 == ("l4" if expected else None)


def test_the_three_levels_agree_on_a_signature() -> None:
    """The same PNG signature, matched three ways, on the same bytes."""
    for data, expected in [
        (PNG, True),
        (b"\x89PNG\r\n\x1a", False),
        (b"header" + PNG, False),
    ]:
        level_2 = MagicCustom("l2", [PNG], [0]).matches(data)

        def is_png(buffer: bytes) -> bool:
            return buffer[: len(PNG)] == PNG

        level_3 = DynMagicCustom(is_png, "l3").matches(data)

        async def matcher(buffer: bytes) -> bool:
            return is_png(buffer)

        assert level_2 is expected
        assert level_3 is expected
        assert run(AsyncDynMagic(matcher, "l4").matches(data)) is expected


def test_a_level_2_rule_handles_what_level_1_cannot() -> None:
    """The point of the exercise: a rule for a format the table has not heard of.

    Level 1 knows 114 formats and no more. Anything else is a level 2 rule, and
    a made-up magic string is the honest way to test that, because no built-in
    signature can collide with it by accident.
    """
    assert detect_bytes(PNG) is FileKind.Png, "the built-in table does know PNG"

    rule = MagicCustom("shoujo", [b"MagicalGirl"], [0])
    assert match_types_custom(b"MagicalGirl\x00", [rule], None) == "shoujo"
    assert match_types_custom(PNG, [rule], None) is None
    assert detect_bytes(b"MagicalGirl\x00") is None, "level 1 has no such format"
