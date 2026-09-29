"""Native Python bindings for `magical_rs`, a zero-dependency Rust library
that identifies file formats by matching magic bytes at exact offsets.

This package is not a drop-in replacement for ``python-magic`` and does not
try to be one. ``python-magic`` returns the string ``"image/png"`` and leaves
the caller to parse it; :func:`detect` returns a :class:`FileKind` member
that carries its media type and conventional extension with it.

    >>> from magical_py import FileKind, detect
    >>> kind = detect("photo.jpg")
    >>> kind is FileKind.Jpg
    True
    >>> kind.mime
    'image/jpeg'
    >>> kind.extension
    'jpg'

Detection reads magic bytes only. The file name and its extension are never
consulted, so a ``.jpg`` holding PNG data is reported as
:attr:`FileKind.Png`.

:func:`detect` takes a path or anything open for binary reading, so a socket, a
pipe and a :class:`zipfile.ZipExtFile` work as well as a file on disk. Both
read only as far as they need, and ``max_bytes_read`` says how far that is,
which is what tells a ``None`` meaning "not that" apart from one meaning "not
within the window you named".

The crate has five detection levels. :func:`detect` and :func:`detect_bytes`
are level 1, the built-in table. The other four are custom rules, and three of
them have a Python counterpart:

* Level 2, :class:`MagicCustom` with :func:`match_types_custom`, describes a
  format as signatures at offsets. A match costs one Rust call.
* Level 3, :class:`DynMagicCustom` with :func:`match_dyn_types`, hands the
  decision to a Python callable. Kinds may differ between rules.
* Level 4, :class:`AsyncDynMagic` with :func:`match_async_dyn_types`, is level 3
  with a matcher that may await.
* Level 5, raw pointers, is ``unsafe`` by definition and is not exposed.

:func:`detect` answers one question, what a file is, and answers it with the
first table entry that matched. Three more ask about the table itself:
:func:`describe` and :attr:`FileKind.rule` say what an entry compares,
:meth:`FileKind.matches` asks whether one format would match regardless of the
order, and :func:`read_limits` reports the read sizes that
``max_bytes_read`` is measured against.

:mod:`magical_py._levels` documents what each level costs and where it
deliberately differs from the crate.
"""

from __future__ import annotations

import os
from typing import Final, Protocol

from . import _magical_rs
from ._kinds import FileKind, Signature
from ._levels import (
    AsyncDynMagic,
    AsyncPredicate,
    DynMagicCustom,
    MagicCustom,
    MatchRules,
    Predicate,
    match_async_dyn_types,
    match_async_dyn_types_all,
    match_dyn_types,
    match_dyn_types_all,
    match_types_custom,
    match_types_custom_all,
)
from ._signatures import (
    DEFAULT_MAX_BYTES_READ,
    ReadLimits,
    describe,
    read_limits,
    signature_table,
)

__all__ = [
    "DEFAULT_MAX_BYTES_READ",
    "AsyncDynMagic",
    "AsyncPredicate",
    "DynMagicCustom",
    "FileKind",
    "MagicCustom",
    "MatchRules",
    "Predicate",
    "ReadLimits",
    "Signature",
    "bytes_read",
    "describe",
    "detect",
    "detect_bytes",
    "match_async_dyn_types",
    "match_async_dyn_types_all",
    "match_dyn_types",
    "match_dyn_types_all",
    "match_types_custom",
    "match_types_custom_all",
    "read_header",
    "read_limits",
    "signature_table",
    "version",
]

__version__: Final[str] = _magical_rs.__version__


class _Reader(Protocol):
    """Anything open for binary reading, which is what a stream is.

    Structural rather than :class:`io.BufferedReader`, because the sources worth
    detecting are not all files: a socket's ``makefile``, a pipe, a
    :class:`zipfile.ZipExtFile` and an :class:`io.BytesIO` all satisfy it and
    none of them shares a base class worth naming. Only ``read`` is required,
    which is also the only one that is used.

    The union of this and a path is spelled out at each use rather than
    aliased: a type alias would have to be written with PEP 604 syntax to be
    read as one, and the floor is 3.8, where that syntax is a runtime error in
    a position that is evaluated when the alias is defined.
    """

    def read(self, size: int = ..., /) -> bytes: ...


def version() -> str:
    """Return the version of the installed ``magical_py`` package."""
    return __version__


def bytes_read() -> int:
    """Return how many bytes must be read to classify any supported format.

    The furthest signature in the table sits at offset 36,865, so a header
    shorter than the returned value cannot be identified reliably.
    :func:`detect` already sizes its own read; this is the figure to pass to
    :func:`read_header` when sizing your own, or as ``max_bytes_read`` when
    bounding a classification.

    :returns: The number of bytes required, currently 36,870.
    """
    return _magical_rs.bytes_read()


def read_header(
    source: str | os.PathLike[str] | _Reader, *, max_bytes: int | None = None
) -> bytes:
    """Read the leading bytes of a file or stream, detecting nothing.

    :func:`detect` does this for a path, but it does it on its own terms: it
    reads :func:`bytes_read` bytes and hands back a :class:`FileKind`. This is
    for a source :func:`detect` cannot take a path to — a file already open, a
    socket, a pipe, an entry inside a :class:`zipfile.ZipFile` — and for a
    caller that wants the bytes themselves rather than a verdict about them.

    The result is what :func:`detect_bytes` expects, so the two compose, and
    naming the window on both sides is what makes a short answer mean
    something::

        with open("disc.iso", "rb") as handle:
            kind = detect_bytes(read_header(handle, max_bytes=2048), max_bytes_read=2048)

    :param source: A path, or an object open for binary reading.
    :param max_bytes: How many bytes to read. Defaults to :func:`bytes_read`,
        which is enough to classify anything in the table.
    :returns: Up to *max_bytes* bytes, or fewer if the source is shorter. A
        file shorter than the window is not an error, and neither is a stream
        that has less to give than was asked for.
    :raises ValueError: If *max_bytes* is negative.
    :raises FileNotFoundError: If *source* is a path that does not exist.
    :raises PermissionError: If *source* cannot be read.
    """
    _check_size(max_bytes, "max_bytes")
    return _read(source, _window(max_bytes))


def detect(
    source: str | os.PathLike[str] | _Reader, *, max_bytes_read: int | None = None
) -> FileKind | None:
    """Identify the format of a file, a path or an open binary stream.

    :param source: A path, or an object open for binary reading. Anything with
        a ``read`` method will do, so a file already open, a socket, a pipe and
        a :class:`zipfile.ZipExtFile` all work. A path is read by the
        extension, which is where the :exc:`OSError` subclasses come from; a
        stream is read here, since it has no path to hand over.
    :param max_bytes_read: How many bytes are worth reading. Defaults to
        :func:`bytes_read`, which reaches every format in the table. Lowering
        it makes the read itself smaller, and any format whose own magic sits
        past that point then cannot match — which is a statement about the
        window, not about the file. See :func:`detect_bytes`.
    :returns: The detected :class:`FileKind`, or ``None`` if no signature
        matched.
    :raises ValueError: If *max_bytes_read* is negative.
    :raises FileNotFoundError: If *source* is a path that does not exist.
    :raises PermissionError: If *source* cannot be read. A directory also
        lands here on Windows, where opening one is denied outright; on other
        platforms it raises :exc:`IsADirectoryError` instead.
    """
    _check_size(max_bytes_read, "max_bytes_read")
    if isinstance(source, (str, os.PathLike)):
        return _lookup(_magical_rs.detect_path(os.fspath(source), max_bytes_read=max_bytes_read))
    return _lookup(
        _magical_rs.detect_bytes(
            source.read(_window(max_bytes_read)), max_bytes_read=max_bytes_read
        )
    )


def detect_bytes(data: bytes, *, max_bytes_read: int | None = None) -> FileKind | None:
    """Identify the format of an in-memory buffer.

    Only the leading bytes are inspected, so passing a whole large file is
    wasteful; :func:`bytes_read` bytes is always enough.

    :param data: The bytes to inspect.
    :param max_bytes_read: The size of the window *data* is claimed to have
        come from. Naming it is what tells a short buffer apart from a wrong
        one. Left unset, a 4 KB buffer holding ISO 9660 magic at offset 32,769
        comes back ``None``, which reads as "this is not an ISO" when the
        truth is that the magic was never inside the buffer. Set to 4,096, the
        same call comes back ``None`` and means "no format whose magic fits in
        4 KB matched" — a statement about the window rather than about the
        file, and one the caller can act on.
    :returns: The detected :class:`FileKind`, or ``None`` if no signature
        matched.
    :raises ValueError: If *max_bytes_read* is negative.
    """
    _check_size(max_bytes_read, "max_bytes_read")
    return _lookup(_magical_rs.detect_bytes(data, max_bytes_read=max_bytes_read))


def _read(source: str | os.PathLike[str] | _Reader, limit: int) -> bytes:
    """Read up to *limit* bytes from a path or from an open binary file.

    A path goes to the extension, so that the crate's own `read_file_header`
    does the reading and the `OSError` subclasses come from where the rest of
    them do. Anything else is read here: a stream has no path, and the crate
    takes one.
    """
    if isinstance(source, (str, os.PathLike)):
        return _magical_rs.read_header(os.fspath(source), max_bytes=limit)
    return source.read(limit)


def _window(value: int | None) -> int:
    """Return the read size to use: the caller's, or the one the table needs.

    Assumes *value* has been through :func:`_check_size`, which both callers do
    before reaching this, so that a negative limit is reported the same way
    whichever of the two it arrived through.
    """
    return bytes_read() if value is None else value


def _check_size(value: int | None, name: str) -> None:
    """Reject a negative read size before it reaches the extension.

    Extracting a negative Python int as `usize` would raise `OverflowError`,
    which is a detail of pyo3 rather than a promise of this API. The wording
    matches the level 2 and 3 constructors, so a caller moving between levels
    only has to learn one message.
    """
    if value is not None and value < 0:
        raise ValueError(f"{name} cannot be negative, got {value}")


def _lookup(name: str | None) -> FileKind | None:
    """Turn a Rust variant name into a :class:`FileKind` member.

    The names cross the language boundary unchanged, so subscripting by name
    is exact rather than a translation. A name the extension produced but
    the enum lacks means the two tables have drifted apart, which
    ``tests/test_drift.py`` is there to catch.
    """
    if name is None:
        return None
    return FileKind[name]
