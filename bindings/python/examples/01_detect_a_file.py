"""Identify one file, given as a path string or a ``pathlib.Path``.

The result is not a media type string to parse afterwards. It is a
:class:`FileKind` member, and the display name, the media type and the
conventional extension travel with it.

Run it with::

    python 01_detect_a_file.py

The files it reads are built from the magic bytes in the readme's format
tables and written to a temporary directory, so the example needs nothing from
this repository and runs anywhere ``magical_py`` is installed.
"""

from __future__ import annotations

import pathlib
import tempfile

from magical_py import FileKind, detect

JPEG = bytes([0xFF, 0xD8, 0xFF, 0xE0]) + b"\x00" * 32
TEXT = b"Ordinary text, with no magic bytes in it.\n"


def describe(kind: FileKind | None) -> str:
    """Render a detection result the way the example prints it."""
    if kind is None:
        return "  no signature matched"
    fields: list[tuple[str, str | None]] = [
        ("member", kind.name),
        ("value", kind.value),
        ("display", kind.__doc__),
        ("media type", kind.mime),
        ("extension", kind.extension),
    ]
    return "\n".join(f"  {label}: {value}" for label, value in fields)


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        image = root / "photo.jpg"
        image.write_bytes(JPEG)
        plain = root / "notes.txt"
        plain.write_bytes(TEXT)

        print(f"detect() on {image.name}, {image.stat().st_size} bytes of JPEG header:")
        print(describe(detect(image)))
        print()
        print(f"detect() on {plain.name}, whose bytes match no signature:")
        print(describe(detect(plain)))
        print()
        print("A str path gives the same answer, so os.PathLike is enough:")
        print(describe(detect(str(image))))


if __name__ == "__main__":
    main()
