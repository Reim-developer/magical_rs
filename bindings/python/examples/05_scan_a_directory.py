"""Classify a whole directory tree and summarise what is in it.

Walking a tree is the case where returning a member rather than a string earns
its keep: ``FileKind`` groups and sorts on its own, so a tally needs no parsing
and no dictionary keyed by media type.

Run it with::

    python 05_scan_a_directory.py
"""

from __future__ import annotations

import pathlib
import tempfile

from magical_py import FileKind, detect

# Relative path -> bytes, built here rather than committed as fixtures so the
# example is self-contained.
TREE: dict[str, bytes] = {
    "photo.png": bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32,
    "holiday.jpg": bytes([0xFF, 0xD8, 0xFF, 0xE0]) + b"\x00" * 32,
    "archive.zip": bytes([0x50, 0x4B, 0x03, 0x04]) + b"\x00" * 32,
    "notes.txt": b"Just some text, with no magic bytes anywhere in it.\n",
    "nested/report.pdf": b"%PDF-1.7\n" + b"\x00" * 32,
    "nested/deep/animated.gif": b"GIF89a" + b"\x00" * 32,
}


def build_tree(root: pathlib.Path) -> None:
    """Write :data:`TREE` under *root*."""
    for name, payload in TREE.items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)


def scan(root: pathlib.Path) -> tuple[dict[FileKind, int], list[tuple[str, int]]]:
    """Tally every file under *root* by kind, and collect what did not match."""
    counts: dict[FileKind, int] = {}
    unknown: list[tuple[str, int]] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        kind = detect(path)
        if kind is None:
            unknown.append((path.relative_to(root).as_posix(), path.stat().st_size))
        else:
            counts[kind] = counts.get(kind, 0) + 1
    return counts, unknown


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        build_tree(root)
        counts, unknown = scan(root)

        print(f"{len(counts) + len(unknown)} files under the tree, tallied by kind:")
        ordered = sorted(counts.items(), key=lambda item: (-item[1], item[0].name))
        for kind, count in ordered:
            print(f"  {kind.__doc__} x {count} ({kind.mime})")

        print()
        print(f"{len(unknown)} of them matched no signature:")
        for name, size in unknown:
            print(f"  {name} ({size} bytes)")


if __name__ == "__main__":
    main()
