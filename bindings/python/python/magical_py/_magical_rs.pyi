"""Type stub for the compiled extension module.

Written by hand because maturin only emits a stub when packaging a wheel, so
an editable install would leave ``_magical_rs`` untyped and pyright strict
would reject every reference to it. ``tests/test_stub.py`` asserts this file
agrees with the compiled module, so it cannot quietly fall out of date.
"""

from typing import Sequence

__version__: str

def detect_path(path: str, /, *, max_bytes_read: int | None = None) -> str | None:
    """Return the ``FileKind`` variant name for the leading bytes of a file.

    Reads ``max_bytes_read`` of them, or the size ``bytes_read()`` reports if
    no limit is given.

    Raises ``FileNotFoundError``, ``PermissionError`` or ``IsADirectoryError``
    on failure.
    """

def detect_bytes(data: bytes, /, *, max_bytes_read: int | None = None) -> str | None:
    """Return the ``FileKind`` variant name for a buffer.

    ``max_bytes_read`` narrows the table to the rules whose own read size fits
    inside that window, which is ``FileKind::match_with_max_read_rule``. The
    crate compiles that under ``not(feature = "std")`` only, so it is
    reproduced over the public ``SIGNATURE_KIND`` here.
    """

def read_header(path: str, /, *, max_bytes: int | None = None) -> bytes:
    """Read up to ``max_bytes`` bytes from a file, detecting nothing.

    The crate's ``read_file_header``. ``max_bytes`` defaults to what
    ``bytes_read()`` reports.
    """

def all_kinds() -> list[str]:
    """Return every variant name the detection table can produce."""

def bytes_read() -> int:
    """Return the header size needed to classify any supported format."""

def read_limits() -> tuple[int, int, list[int], int, list[int], int]:
    """Return the crate's read sizes and offsets, in one tuple.

    ``(default_max_bytes_read, default_offset, iso_offsets, iso_max_bytes_read,
    tar_offsets, tar_max_bytes_read)``. A tuple rather than a dict because the
    values differ in type and a ``dict[str, object]`` would cost every caller
    six casts; the Python layer names them in a frozen dataclass.
    """

def max_bytes_read_for(offsets: Sequence[int], signature_len: int, /) -> int:
    """Return the read size ``offsets`` and a signature of that length require.

    The crate's ``max_bytes`` is a ``const fn`` over slices, so it cannot be
    called from Python. Only the signature's length is used, so the caller
    passes the length rather than the bytes.
    """

def signature_of(
    kind: str, /
) -> tuple[str, list[bytes], list[int], int, bool] | None:
    """Return one table entry as ``(kind, signatures, offsets, max_bytes_read, uses_predicate)``.

    ``None`` when no signature produces ``kind``. A predicate entry reports
    empty ``signatures`` and ``uses_predicate`` true, because there are no
    bytes to report.
    """

def signature_table() -> list[tuple[str, list[bytes], list[int], int, bool]]:
    """Return every table entry, in the order detection tries them."""

def kind_matches(kind: str, data: bytes, /) -> bool:
    """Report whether one named format's rule matches ``data``, ignoring the table.

    Raises ``ValueError`` for a name the table does not contain, since "no such
    format" and "did not match" are different answers.
    """

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
    the process, as the crate's does since ``0.6.2``.
    """
