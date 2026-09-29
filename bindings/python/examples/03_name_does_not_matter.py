"""The file name is never read.

Detection matches magic bytes at fixed offsets, so a ``.jpg`` holding PNG data
is reported as PNG. The extension on a result describes the format that was
found; it is not a claim about the file it was found in, and no amount of
renaming changes the answer.

Run it with::

    python 03_name_does_not_matter.py
"""

from __future__ import annotations

import pathlib
import tempfile

from magical_py import FileKind, detect

PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32
TEXT = b"Ordinary text, with no magic bytes in it.\n"


def report(path: pathlib.Path) -> None:
    """Print what *path* is called against what it turned out to be."""
    kind = detect(path)
    answer = kind.name if kind is not None else "no signature matched"
    print(f"  {path.name} -> {answer}")


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        (root / "photo.jpg").write_bytes(PNG)
        (root / "notes.png").write_bytes(TEXT)
        (root / "logo").write_bytes(PNG)

        print("Three files, three names, one of them a lie:")
        report(root / "photo.jpg")
        report(root / "notes.png")
        report(root / "logo")

    print()
    print(
        f"FileKind.Png.extension is {FileKind.Png.extension!r}, which describes the"
        " format and not the file it was found in."
    )


if __name__ == "__main__":
    main()
