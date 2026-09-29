"""Type stub for the compiled extension module.

Written by hand because maturin only emits a stub when packaging a wheel, so
an editable install would leave ``_magical_rs`` untyped and pyright strict
would reject every reference to it. ``tests/test_stub.py`` asserts this file
agrees with the compiled module, so it cannot quietly fall out of date.
"""

from typing import Sequence

__version__: str

def detect_path(path: str, /) -> str | None:
    """Return the ``FileKind`` variant name for the file at *path*.

    Raises ``FileNotFoundError``, ``PermissionError`` or ``IsADirectoryError``
    on failure.
    """

def detect_bytes(data: bytes, /) -> str | None:
    """Return the ``FileKind`` variant name for a buffer."""

def all_kinds() -> list[str]:
    """Return every variant name the detection table can produce."""

def bytes_read() -> int:
    """Return the header size needed to classify any supported format."""

def signatures_match(
    data: bytes, signatures: Sequence[bytes], offsets: Sequence[int], /
) -> bool:
    """Report whether any signature appears at any of the offsets in ``data``.

    Mirrors the ``CustomMatchRules::Default`` arm of the crate's level 2
    matching, reimplemented here because ``MagicCustom`` holds ``'static``
    slices and a rule built from Python data cannot be one.

    Any sequence will do, not only a list: pyo3 extracts an owned ``Vec`` from
    a tuple, a list or any other sequence, and taking the argument by value is
    the only form it can. The addition of offset and signature length
    saturates, so a nonsensical offset reports no match instead of aborting
    the process, which is what the crate does with one.
    """
