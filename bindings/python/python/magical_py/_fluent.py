"""``detected(path).is(FileKind.Png)`` — detection with the data first.

:func:`magical_py.detect` puts the function first and the file second, which is
the right shape for a question asked once and thrown away:

    >>> from magical_py import FileKind, detect
    >>> detect("photo.jpg") is FileKind.Jpg
    True

It is the wrong shape for the case this module exists for: a scan that asks
about every file in a directory, where the answer is usually *no*. There the
readable form is the one Rust's ``magical_fluent`` feature gives, with the subject
written first and the question trailing it:

    >>> from magical_py import FileKind, detected
    >>> detected("photo.jpg").matches(FileKind.Jpg)
    True

# Why not a method on ``bytes``

Because Python will not let it. ``bytes`` is a built-in type, so the spelling that
would read best here — ``b"...".detect()`` — is the one this language cannot offer,
and :class:`Detected` is the wrapper standing in for it.

# Why not ``is`` and ``is_any``

Because ``is`` is a keyword, so ``handle.is(FileKind.Png)`` is a ``SyntaxError``
rather than a call. Rust's ``Detect::is`` is :meth:`Detected.matches` here, and
:meth:`Detected.matches_any` is its ``is_any``. The rename is the language's, not a
preference, and the new name is better anyway: it is the inverse of the
:meth:`magical_py.FileKind.matches` that already existed, so a caller asking the
question one way or the other reaches for the same word.

# What this is not

It adds no matching. Every method below is a named call to something the binding
already had — :func:`magical_py.detect_bytes`,
:meth:`~magical_py.FileKind.matches`, :attr:`~magical_py.FileKind.rule` — and
the answers are the ones those give. This is a spelling, not a detection level.

# What it costs

The bytes are read once, on first use, and kept. A path is opened exactly once
however many methods are called on the result, and a stream is read exactly once,
which matters because a socket or a pipe has its bytes only once.

That is also the limit of :meth:`Detected.within`. It re-classifies what was
read; it cannot reach bytes that were never read. Narrowing the window always
works, because the bytes are a superset of the narrower one. Widening it works
too, but only as far as the read went — which is stated on the method rather than
left to be discovered by a caller who widened a window and got the same answer.
"""

from __future__ import annotations

import os
from typing import Iterable, Protocol

from ._kinds import FileKind, Signature

# Imported from the submodule rather than as `from . import _magical_rs`, for the
# reason `_signatures` gives: the latter names the package, and the package is in
# the middle of importing this module.
from ._magical_rs import bytes_read as _crate_bytes_read
from ._magical_rs import detect_bytes as _crate_detect_bytes
from ._magical_rs import read_header as _crate_read_header

__all__ = [
    "Detected",
    "detected",
]


class _Reader(Protocol):
    """Anything open for binary reading, which is what a stream is.

    Structural rather than :class:`io.BufferedReader`, for the reason the package
    ``__init__`` gives: the sources worth detecting are a socket's ``makefile``, a
    pipe, a :class:`zipfile.ZipExtFile` and an :class:`io.BytesIO`, and they share
    no base class worth naming. Only ``read`` is required, and it is the only one
    that is used.

    Declared rather than elided with ``hasattr`` so that the stream branch is
    reachable as far as a type checker is concerned. Without it, the union of the
    other three source types is exhaustive, ``reportUnreachable`` fires on the
    branch that actually handles streams, and the error points at working code.
    """

    def read(self, size: int = ..., /) -> bytes: ...



def detected(
    source: str | os.PathLike[str] | bytes | bytearray | memoryview | _Reader,
    *,
    max_bytes_read: int | None = None,
) -> Detected:
    """Return a :class:`Detected` over *source*.

    Nothing is read here: the read happens on the first method that needs an
    answer, so building one is free and a caller who only wants to pass it on
    never touches the disk.

    :param source: A path, anything open for binary reading, or the bytes
        themselves — the three shapes :func:`magical_py.detect` and
        :func:`magical_py.detect_bytes` accept between them, so that
        ``detected(...)`` is a spelling rather than a decision about where the
        bytes come from.
    :param max_bytes_read: The window to classify in. Defaults to
        :func:`magical_py.bytes_read`, which reaches every format in the table.
    :raises ValueError: If *max_bytes_read* is negative.
    :returns: A :class:`Detected`, not yet read.
    """
    if max_bytes_read is not None and max_bytes_read < 0:
        raise ValueError(f"max_bytes_read cannot be negative, got {max_bytes_read}")
    return Detected(source, max_bytes_read=max_bytes_read)


class Detected:
    """A file's format, and the questions about it, with the file written first.

    Not a value object: it holds the bytes, so it holds memory, and one of these
    per file across a large scan adds up. It is meant to answer a few questions
    about one file and then be dropped.

    .. code-block:: python

        from magical_py import FileKind, detected

        for path in directory.iterdir():
            if detected(path).matches_any(FileKind.Png, FileKind.Gif, FileKind.Jpg):
                ...

    :param source: A path, an open binary stream, or bytes.
    :param max_bytes_read: The window to classify in. ``None`` means
        :func:`magical_py.bytes_read`.
    """

    __slots__ = ("_data", "_kind", "_kind_window", "_source", "_window")

    def __init__(
        self,
        source: str | os.PathLike[str] | bytes | bytearray | memoryview | _Reader,
        *,
        max_bytes_read: int | None = None,
    ) -> None:
        self._source = source
        self._window = _crate_bytes_read() if max_bytes_read is None else max_bytes_read
        self._data: bytes | None = None
        # `_kind_window` is the window `_kind` was computed in, so a window change
        # asks again instead of answering from the wrong one.
        self._kind: FileKind | None = None
        self._kind_window: int | None = None

    def __repr__(self) -> str:
        """Say what it found, which means reading the file.

        A ``repr`` that did not read would print the same thing for every input,
        which is the one thing a repr must not do. The read is deferred to
        ``kind`` rather than done here, so building the object stays free.
        """
        kind = self.kind
        return f"<Detected {kind.name if kind is not None else 'no match'}>"

    def __bool__(self) -> bool:
        """True when a format matched, so ``if detected(path):`` reads as it says.

        The one place a ``None`` is treated as false, and here it means exactly
        one thing: no signature matched.
        """
        return self.kind is not None

    @property
    def kind(self) -> FileKind | None:
        """The format, or ``None`` if the table does not recognise these bytes.

        The same answer as :func:`magical_py.detect_bytes`: first match in table
        order over the whole table, filtered to rules whose own read size fits the
        window. Cached, so asking twice reads nothing twice.
        """
        if self._kind_window != self._window:
            self._kind = _classify(self._data_bytes(), self._window)
            self._kind_window = self._window
        return self._kind

    def matches(self, kind: FileKind) -> bool:
        """Whether these bytes are *kind*, ignoring every other rule.

        Named ``matches`` and not ``is``, because ``is`` is a keyword in Python
        and ``handle.is(kind)`` does not parse. It is the inverse of
        :meth:`magical_py.FileKind.matches`, and the two read as a pair —
        ``kind.matches(data)`` and ``data.matches(kind)`` — which is what a
        caller reaching for one of them is usually doing.

        Not the same question as ``.kind is kind``, and the difference is worth
        naming. This asks whether that one format's own rule matches, so a file
        that is both ``Ktx`` and — by its first bytes — an earlier entry in the
        table is the earlier entry as far as :attr:`kind` is concerned, and is
        still "yes, it is Ktx" as far as this is concerned.

        ``Ktx`` is the case that matters: its magic is a prefix of ``Ktx2``'s, so a
        KTX2 file is never *reported* as KTX, and this is the only way to ask.

        :param kind: The format to test for.
        """
        return kind.matches(self._data_bytes())

    def matches_any(self, *kinds: FileKind | Iterable[FileKind]) -> bool:
        """Whether these bytes are any of *kinds*, one lookup per format.

        Accepts the varargs of this form or a single iterable, so that both
        ``matches_any(FileKind.Png, FileKind.Gif)`` and
        ``matches_any([png, gif])`` are correct. That is not cleverness: a caller
        holding a list should not have to splat it, and a caller holding two
        literals should not have to bracket them.

        The formats are tried in the order given and the answer is the first that
        matches, so this is a ``bool`` rather than which one matched.

        An empty argument list is ``False``, since a file cannot be any of
        nothing.
        """
        return any(self.matches(kind) for kind in _flatten(kinds))

    def within(self, max_bytes_read: int) -> Detected:
        """Classify the same bytes in a different window, and return ``self``.

        Named after the Rust ``detect_within``. A window is a claim about how much
        of the file was read, so lowering it makes every format whose own magic
        sits past that point unable to match — a statement about the window and
        not about the file.

        This does not read again, because it does not have to: the bytes already
        in hand are a superset of a narrower window, and
        :func:`magical_py.detect_bytes` filters by the window whatever it is
        handed. Widening the window beyond the original read cannot recover bytes
        that were never read, and :attr:`window` reports what it actually is so a
        caller can see that rather than infer it.

        :param max_bytes_read: The new window, in bytes.
        :raises ValueError: If *max_bytes_read* is negative.
        :returns: ``self``, so it can be chained onto the constructor.
        """
        if max_bytes_read < 0:
            raise ValueError(f"max_bytes_read cannot be negative, got {max_bytes_read}")
        self._window = max_bytes_read
        # Dropped rather than trusted: the cache is keyed on the window.
        self._kind_window = None
        return self

    @property
    def window(self) -> int:
        """The window in force, in bytes.

        Defaults to :func:`magical_py.bytes_read` — 36,870 — and is *not*
        :attr:`magical_py.DEFAULT_MAX_BYTES_READ`, which is the crate's 2,048. Both
        are worth having and they are different numbers, so this says which is in
        play rather than leaving a caller to remember.
        """
        return self._window

    @property
    def rule(self) -> Signature | None:
        """What :attr:`kind` compares, or ``None`` when nothing matched.

        The same value as :attr:`magical_py.FileKind.rule`, reached without naming
        the kind first. It is ``None`` here rather than raising as that property
        does, because "nothing matched" is a normal answer for this object and not
        an exceptional one.
        """
        kind = self.kind
        return None if kind is None else kind.rule

    @property
    def mime(self) -> str | None:
        """The media type of :attr:`kind`, or ``None`` if there is none."""
        kind = self.kind
        return None if kind is None else kind.mime

    @property
    def extension(self) -> str | None:
        """The conventional extension of :attr:`kind`, or ``None``."""
        kind = self.kind
        return None if kind is None else kind.extension

    @property
    def description(self) -> str:
        """The human-readable name of :attr:`kind`, or ``"no match"``.

        Every :class:`~magical_py.FileKind` has a display name, so this is a
        string in both cases — which is what a caller printing a result wants, and
        why it is not ``None`` the way :attr:`mime` is.
        """
        kind = self.kind
        return "no match" if kind is None else kind.description

    def _data_bytes(self) -> bytes:
        """The bytes of the source, read once and kept.

        Private because it is an implementation detail of the cache, and public
        because ``is`` needs the bytes rather than the kind: asking whether one
        format matches must not be answered by whatever the table happened to
        reach first.
        """
        if self._data is None:
            self._data = _read_source(self._source, self._window)
        return self._data


def _flatten(kinds: tuple[FileKind | Iterable[FileKind], ...]) -> tuple[FileKind, ...]:
    """Accept both ``is_any(a, b)`` and ``is_any([a, b])``.

    One argument that is a list is the caller's list; two or more are the varargs.
    A single :class:`~magical_py.FileKind` is not iterable, so the one-argument
    case needs no try/except to tell a kind from a collection.

    A tuple out, so a caller passing a generator has it consumed once here rather
    than once per format.
    """
    if len(kinds) == 1:
        only = kinds[0]
        if isinstance(only, FileKind):
            return (only,)
        return tuple(only)
    return tuple(kinds)  # type: ignore[arg-type]


def _read_source(
    source: str | os.PathLike[str] | bytes | bytearray | memoryview | _Reader, window: int
) -> bytes:
    """Return *source* as bytes, reading a path or a stream if it is one.

    The shapes are told apart here rather than in each method, because they are the
    same decision several times over and getting one wrong shows up only for that
    kind of input.

    A path goes through the extension's own reader rather than a call back into
    ``magical_py._read``: importing the package from inside this module would make
    the two import each other, and pyright reports that as a cycle.
    """
    if isinstance(source, bytes):
        return source
    if isinstance(source, (bytearray, memoryview)):
        return bytes(source)
    if isinstance(source, (str, os.PathLike)):
        return _crate_read_header(os.fspath(source), max_bytes=window)

    # `getattr` rather than a bare `source.read(window)`, so that the error below
    # is a sentence about the three shapes that are accepted instead of an
    # `AttributeError` from whichever attribute happened to be missing. It also
    # keeps this branch reachable as far as a type checker is concerned: after the
    # three `isinstance` checks the declared type is already narrowed to `_Reader`,
    # so calling `.read` directly is reported as unreachable even though it runs.
    reader = getattr(source, "read", None)
    if reader is None:
        raise TypeError(
            "detected() takes a path, a readable object or bytes, "
            f"not {type(source).__name__}"
        )
    return reader(window)



def _classify(data: bytes, window: int) -> FileKind | None:
    """Return the format of *data* within *window*.

    Takes bytes rather than a source because every caller has already read, and
    routing a path through here would open it a second time.
    """
    name = _crate_detect_bytes(data, max_bytes_read=window)
    return None if name is None else FileKind[name]