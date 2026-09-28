"""Type stub for the compiled extension module.

Written by hand because maturin only emits a stub when packaging a wheel, so
an editable install would leave ``_magical_rs`` untyped and pyright strict
would reject every reference to it. ``tests/test_stub.py`` asserts this file
agrees with the compiled module, so it cannot quietly fall out of date.
"""

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
