"""Three different outcomes: a kind, ``None``, or an exception.

``detect`` returning ``None`` means the bytes matched no signature. An
exception means the file could not be read at all. Keeping those apart matters:
a caller that folds a permission error into "unknown format" silently drops
files and never finds out which ones.

Run it with::

    python 06_handle_errors.py
"""

from __future__ import annotations

import pathlib
import tempfile

from magical_py import detect

PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32
TEXT = b"No magic bytes in this file at all.\n"


def attempt(label: str, path: pathlib.Path) -> None:
    """Run *detect* on *path* and print which of the three outcomes it was."""
    try:
        kind = detect(path)
    except (IsADirectoryError, PermissionError) as error:
        # Which of the two arrives depends on the platform: Windows denies
        # opening a directory outright, while POSIX reports that it is one.
        print(f"  {label}: raised {type(error).__name__}")
    except OSError as error:
        print(f"  {label}: raised {type(error).__name__}: {error}")
    else:
        answer = kind.name if kind is not None else "None, no signature matched"
        print(f"  {label}: returned {answer}")


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        image = root / "image.png"
        image.write_bytes(PNG)
        plain = root / "notes.txt"
        plain.write_bytes(TEXT)
        folder = root / "folder"
        folder.mkdir()

        print("One function, four calls, three outcomes:")
        attempt("an existing PNG", image)
        attempt("bytes with no magic", plain)
        attempt("a path that is absent", root / "missing.png")
        attempt("a directory", folder)

    print()
    print("Only the second line is a None. The last two are failures to read the")
    print("file at all, and are worth handling differently.")


if __name__ == "__main__":
    main()
