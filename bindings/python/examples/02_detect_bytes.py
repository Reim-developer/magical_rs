"""Detect from memory, and read no more of a file than detection needs.

``bytes_read()`` is 36,870 because the furthest signature in the table, ISO
9660, stores its magic at offset 36,865. A header even one byte too short
misses it and comes back as no match, which is why the figure is a function to
ask rather than a constant to guess at.

That is also why a no match needs a little care. A 2,048-byte header holding
a 64 MiB disc's magic is not a file of an unknown type; it is a window too
small to hold the answer, and without more to go on the two look the same.
``max_bytes_read`` supplies the missing half: name the window and the no match
becomes a statement about the window, which a caller can do something about.

Run it with::

    python 02_detect_bytes.py
"""

from __future__ import annotations

import pathlib
import tempfile

from magical_py import bytes_read, detect, detect_bytes, read_header

ISO_MAGIC = b"CD001"
ISO_OFFSET = 36_865
CHEAP_READ = 2_048


def build_iso(size: int) -> bytes:
    """Return *size* zero bytes carrying the ISO 9660 magic at its offset."""
    data = bytearray(size)
    data[ISO_OFFSET : ISO_OFFSET + len(ISO_MAGIC)] = ISO_MAGIC
    return bytes(data)


def main() -> None:
    window = bytes_read()
    print(f"bytes_read() is {window}. The ISO 9660 magic sits at offset {ISO_OFFSET},")
    print(
        f"so it needs those bytes plus {len(ISO_MAGIC)}, which lands exactly on the"
        " end of the window."
    )

    header = build_iso(window)
    print()
    print(f"  {window} bytes of header -> {detect_bytes(header)}")
    for short in (window - 1, ISO_OFFSET, CHEAP_READ):
        print(f"  {short} bytes of header -> {detect_bytes(header[:short])}")

    # The last of those three is not the same answer as the other two. It
    # missed a format rather than rejecting one, and naming the window is what
    # says so.
    print()
    print(f"  The last one, with the window named as max_bytes_read={CHEAP_READ}:")
    print(f"    -> {detect_bytes(header[:CHEAP_READ], max_bytes_read=CHEAP_READ)}")
    print(f"    which is about the window, not the file: ISO 9660 needs {window}")

    # The read is a fixed size whatever the file is, so classifying a 64 MiB
    # disc costs the same 36,870 bytes as classifying a 40-byte header.
    with tempfile.TemporaryDirectory() as directory:
        disc = pathlib.Path(directory) / "disc.iso"
        with disc.open("wb") as handle:
            handle.write(header)
            handle.truncate(64 * 1024 * 1024)
        print()
        print(f"  {disc.stat().st_size:,} byte file, read {window:,} bytes")
        print(f"    -> {detect_bytes(read_header(disc))}")

        # The same file, detected from the open file rather than its path.
        # Anything with a read method works, so a socket or an archive entry
        # is no harder.
        with disc.open("rb") as handle:
            print(f"    -> {detect(handle)}, from the open file")


if __name__ == "__main__":
    main()
