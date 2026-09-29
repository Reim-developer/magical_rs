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

:mod:`magical_py._levels` documents what each level costs and where it
deliberately differs from the crate.
"""

from __future__ import annotations

import os
from typing import Final

from . import _magical_rs
from ._kinds import FileKind
from ._levels import (
    AsyncDynMagic,
    DynMagicCustom,
    MagicCustom,
    MatchRules,
    match_async_dyn_types,
    match_async_dyn_types_all,
    match_dyn_types,
    match_dyn_types_all,
    match_types_custom,
    match_types_custom_all,
)

__all__ = [
    "AsyncDynMagic",
    "DynMagicCustom",
    "FileKind",
    "MagicCustom",
    "MatchRules",
    "bytes_read",
    "detect",
    "detect_bytes",
    "match_async_dyn_types",
    "match_async_dyn_types_all",
    "match_dyn_types",
    "match_dyn_types_all",
    "match_types_custom",
    "match_types_custom_all",
    "version",
]

__version__: Final[str] = _magical_rs.__version__


def version() -> str:
    """Return the version of the installed ``magical_py`` package."""
    return __version__


def bytes_read() -> int:
    """Return how many bytes must be read to classify any supported format.

    The furthest signature in the table sits at offset 36,865, so a header
    shorter than the returned value cannot be identified reliably. This is
    only useful if you are reading file headers yourself; :func:`detect`
    already sizes its own read.

    :returns: The number of bytes required, currently 36,870.
    """
    return _magical_rs.bytes_read()


def detect(path: str | os.PathLike[str]) -> FileKind | None:
    """Identify the format of the file at *path*.

    :param path: A path to an existing file. :class:`pathlib.Path` is
        accepted.
    :returns: The detected :class:`FileKind`, or ``None`` if no signature
        matched.
    :raises FileNotFoundError: If *path* does not exist.
    :raises PermissionError: If *path* cannot be read. A directory also
        lands here on Windows, where opening one is denied outright; on other
        platforms it raises :exc:`IsADirectoryError` instead.
    """
    return _lookup(_magical_rs.detect_path(os.fspath(path)))


def detect_bytes(data: bytes) -> FileKind | None:
    """Identify the format of an in-memory buffer.

    Only the leading bytes are inspected, so passing a whole large file is
    wasteful; :func:`bytes_read` bytes is always enough.

    :param data: The bytes to inspect.
    :returns: The detected :class:`FileKind`, or ``None`` if no signature
        matched.
    """
    return _lookup(_magical_rs.detect_bytes(data))


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
