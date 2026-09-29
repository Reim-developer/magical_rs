"""Tests for :func:`magical_py.detect`, :func:`magical_py.detect_bytes` and
:func:`magical_py.read_header`.

The fixtures here are written to ``tmp_path`` from magic bytes rather than
checked in as binaries. A committed ``1.png`` in the repository would be a
second copy of the truth that can drift, and it would bloat the wheel's test
suite. Building a header from the documented bytes also makes the test state
exactly which signature it is exercising.
"""

from __future__ import annotations

import io
import pathlib
import zipfile

import pytest

from magical_py import FileKind, bytes_read, detect, detect_bytes, read_header

# Signatures copied from the format tables in readme.md, which
# tests/readme_coverage.rs keeps in step with the Rust table.
HEADERS: dict[FileKind, bytes] = {
    FileKind.Png: bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
    FileKind.Jpg: bytes([0xFF, 0xD8, 0xFF, 0xE0]),
    FileKind.GIF: b"GIF87a",
    FileKind.PDF: b"%PDF-",
    FileKind.PkgZip: bytes([0x50, 0x4B, 0x03, 0x04]),
    FileKind.Gzip: bytes([0x1F, 0x8B]),
    FileKind.Bzip: b"BZh",
    FileKind.SevenZip: bytes([0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
    FileKind.Zstd: bytes([0x28, 0xB5, 0x2F, 0xFD]),
    FileKind.ELF: bytes([0x7F, 0x45, 0x4C, 0x46]),
    FileKind.WASM: bytes([0x00, 0x61, 0x73, 0x6D]),
    FileKind.Dalvik: b"dex\n",
    FileKind.SQLite: b"SQLite format 3\x00",
    FileKind.XML: b"<?xml ",
}

# Only headers long enough to be unambiguous on their own. Gzip, for
# instance, is two bytes, so a test built from it alone would not prove
# much.
CLEAR_HEADERS = {k: v for k, v in HEADERS.items() if len(v) >= 4}

PNG = HEADERS[FileKind.Png]
ISO_MAGIC = b"CD001"
ISO_OFFSET = 36_865
ISO_MAGIC_OFFSETS = (32_769, 34_817, 36_865)


def build_iso(size: int) -> bytes:
    """Return *size* zero bytes carrying the ISO 9660 magic at its offset."""
    data = bytearray(size)
    data[ISO_OFFSET : ISO_OFFSET + len(ISO_MAGIC)] = ISO_MAGIC
    return bytes(data)


@pytest.mark.parametrize(("kind", "header"), list(CLEAR_HEADERS.items()))
def test_detect_bytes_recognises_each_header(kind: FileKind, header: bytes) -> None:
    assert detect_bytes(header) is kind


@pytest.mark.parametrize(("kind", "header"), list(CLEAR_HEADERS.items()))
def test_detect_path_recognises_each_header(
    kind: FileKind, header: bytes, tmp_path: pathlib.Path
) -> None:
    path = tmp_path / f"sample-{kind.value}.bin"
    path.write_bytes(header + b"\x00" * 64)
    assert detect(path) is kind


def test_detect_bytes_ignores_trailing_content() -> None:
    header = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    assert detect_bytes(header + b"trailing bytes") is FileKind.Png


def test_detect_bytes_returns_none_for_unknown_content() -> None:
    assert detect_bytes(b"just some text, nothing magic about it") is None


def test_detect_bytes_returns_none_for_empty_input() -> None:
    assert detect_bytes(b"") is None


def test_detect_path_accepts_str_and_pathlike(tmp_path: pathlib.Path) -> None:
    path = tmp_path / "image.png"
    path.write_bytes(bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32)
    assert detect(str(path)) is FileKind.Png
    assert detect(path) is FileKind.Png


def test_detect_raises_file_not_found(tmp_path: pathlib.Path) -> None:
    with pytest.raises(FileNotFoundError):
        detect(tmp_path / "absent.png")


def test_detect_propagates_os_errors(tmp_path: pathlib.Path) -> None:
    """A directory is an error, not an unknown format.

    Returning ``None`` for a directory would be indistinguishable from a
    file whose signature does not match, and the caller could not tell a
    mistake in the path from a genuine negative result.
    """
    with pytest.raises(OSError):
        detect(tmp_path)


def test_short_file_does_not_crash(tmp_path: pathlib.Path) -> None:
    """A file shorter than its signature must not read out of bounds."""
    path = tmp_path / "truncated.png"
    path.write_bytes(bytes([0x89, 0x50]))
    assert detect(path) is None


def test_bytes_read_matches_the_furthest_signature() -> None:
    """The advertised read size must cover every offset in the table.

    ISO 9660 is detected at offset 36,865, so a caller that reads less than
    that will silently miss it. This pins the value the docstring quotes.
    """
    assert bytes_read() == 36_870


def test_high_offset_format_is_found(tmp_path: pathlib.Path) -> None:
    """A signature far into the file is only reachable with a big read.

    This is the end-to-end check that ``detect`` really does read
    ``bytes_read()`` bytes, rather than some smaller default.
    """
    header = bytearray(36_865)
    header[36_865:36_870] = b"CD001"
    path = tmp_path / "disc.iso"
    path.write_bytes(bytes(header))
    assert detect(path) is FileKind.ISO


def test_detection_does_not_consult_the_file_name(tmp_path: pathlib.Path) -> None:
    """A misleading extension must not change the answer.

    Detection matches magic bytes only. If the file name were consulted, a
    PNG named ``.jpg`` would come back as JPEG, which is the exact failure
    this library exists to avoid.
    """
    path = tmp_path / "actually-a-png.jpg"
    path.write_bytes(bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32)
    assert detect(path) is FileKind.Png


def test_shebang_requires_a_path_separator() -> None:
    """A script header must have a path separator on the first line.

    ``#!AMR`` is not a script, it is an AMR audio file: that is the literal
    magic of the format. Before the shebang rule was narrowed, the script
    rule claimed it first and AMR audio became undetectable.
    """
    assert detect_bytes(b"#!/bin/sh\necho hi\n") is FileKind.ScriptExecute
    assert detect_bytes(b"#!AMR\n") is FileKind.Amr


def test_hash_bang_without_a_slash_is_not_a_script() -> None:
    """A bare ``#!X`` is not a script and matches no other format."""
    assert detect_bytes(b"#!X\n") is None


# --- max_bytes_read -----------------------------------------------------------
#
# Naming the window is what separates "not that format" from "not within the
# window you named", so the tests below are about which of the two a `None`
# means rather than about detection working at all.


def test_a_short_buffer_is_not_a_wrong_answer() -> None:
    """A 4 KB buffer holding ISO magic is a window too small, not a no-match.

    ISO 9660 stores its magic at offset 36,865. Handed a buffer that stops
    before it, the search finds nothing, and a bare `None` would read as "this
    is not an ISO" when the truth is that the magic was never in the buffer.
    Naming the window is what makes the answer interpretable.
    """
    short = build_iso(4_096)
    assert detect_bytes(short) is None
    assert detect_bytes(short, max_bytes_read=4_096) is None
    assert detect_bytes(build_iso(bytes_read()), max_bytes_read=bytes_read()) is FileKind.ISO


def test_a_named_window_can_exclude_a_format_whose_magic_fits() -> None:
    """A limit below a rule's own read size drops the rule, by design.

    Every rule in the table declares a read size, the smallest being 2,048, so
    a window under that excludes PNG along with everything else. This is the
    crate's `FileKind::match_with_max_read_rule` semantics and it is the price
    of a cheap read: a smaller window means fewer formats reachable, not a
    coarser judgement about the same ones.
    """
    assert detect_bytes(PNG, max_bytes_read=2_048) is FileKind.Png
    assert detect_bytes(PNG, max_bytes_read=2_047) is None
    assert detect_bytes(PNG, max_bytes_read=0) is None


def test_the_default_window_reaches_everything() -> None:
    """Omitting the limit must not narrow the table.

    This is the backwards-compatibility half: the argument is new, so the
    behaviour without it has to be exactly what it was.
    """
    for kind, header in CLEAR_HEADERS.items():
        assert detect_bytes(header) is kind
        assert detect_bytes(header, max_bytes_read=bytes_read()) is kind


def test_the_limit_is_keyword_only() -> None:
    """Passing it positionally would silently change what a caller means.

    `detect_bytes` takes one positional argument, so a second positional one
    is a mistake rather than a request, and it should not be honoured.
    """
    with pytest.raises(TypeError):
        detect_bytes(PNG, 2_048)  # type: ignore[misc]


@pytest.mark.parametrize("limit", [-1, -2_048])
def test_a_negative_limit_is_refused(limit: int) -> None:
    """Reading a negative number of bytes is a caller error, not an empty read.

    The level 2 and 3 constructors raise `ValueError` for the same mistake, so
    the wording is shared: a caller moving between levels should not have to
    learn two error messages for one fault.
    """
    with pytest.raises(ValueError, match="max_bytes_read cannot be negative"):
        detect_bytes(PNG, max_bytes_read=limit)


# --- read_header -------------------------------------------------------------


def test_read_header_reads_the_window_by_default(tmp_path: pathlib.Path) -> None:
    target = tmp_path / "disc.iso"
    target.write_bytes(build_iso(bytes_read()))
    assert len(read_header(target)) == bytes_read()


def test_read_header_honours_an_explicit_size(tmp_path: pathlib.Path) -> None:
    target = tmp_path / "disc.iso"
    target.write_bytes(build_iso(bytes_read()))
    assert len(read_header(target, max_bytes=2_048)) == 2_048


def test_read_header_returns_the_files_own_bytes(tmp_path: pathlib.Path) -> None:
    """A short file comes back short, not padded to the requested size.

    The header is allocated at the size asked for, so without truncating it a
    small file came back full of invented zeros. That was harmless while the
    bytes went only into a detection, and it stopped being harmless the moment
    they were handed to a caller: 36,870 bytes of which 36,866 were not the
    file's.
    """
    target = tmp_path / "short.bin"
    target.write_bytes(b"II*")
    assert read_header(target) == b"II*"
    assert len(read_header(target, max_bytes=bytes_read())) == 3


def test_read_header_never_reads_past_the_end(tmp_path: pathlib.Path) -> None:
    """Asking for more than the file holds is ordinary, not an error."""
    target = tmp_path / "short.bin"
    target.write_bytes(PNG)
    assert read_header(target, max_bytes=1_000_000) == PNG


def test_read_header_raises_the_same_errors_as_detect(tmp_path: pathlib.Path) -> None:
    with pytest.raises(FileNotFoundError):
        read_header(tmp_path / "absent.png")
    with pytest.raises(OSError):
        read_header(tmp_path)


def test_read_header_refuses_a_negative_size(tmp_path: pathlib.Path) -> None:
    with pytest.raises(ValueError, match="max_bytes cannot be negative"):
        read_header(tmp_path, max_bytes=-1)


def test_read_header_feeds_detect_bytes(tmp_path: pathlib.Path) -> None:
    """The two compose, which is the only reason either is worth having."""
    target = tmp_path / "disc.iso"
    target.write_bytes(build_iso(bytes_read()))
    window = 2_048
    assert detect_bytes(read_header(target, max_bytes=window), max_bytes_read=window) is None


def test_a_file_and_its_own_bytes_agree(tmp_path: pathlib.Path) -> None:
    """`detect` and `detect_bytes` must not disagree about the same data.

    Detection that answers differently depending on whether the caller had a
    path or a buffer is worse than either answer. It happened: the read was
    zero-filled to its capacity, and two formats end their magic with a zero —
    PCX is `[0x0A, 0x00]`, TIFF is `II*\0` — so a one-byte file holding a
    newline was a PCX by path and nothing at all by buffer.
    """
    for content in (b"\x0a", b"II*", b"Rar!\x1a\x07", b"\x00\x00\x01\x00"):
        target = tmp_path / f"tiny-{len(content)}.bin"
        target.write_bytes(content)
        assert detect(target) is detect_bytes(content), f"{content!r} disagrees"


# --- sources other than a path -----------------------------------------------


def test_detect_reads_an_open_file(tmp_path: pathlib.Path) -> None:
    target = tmp_path / "image.png"
    target.write_bytes(PNG + b"\x00" * 32)
    with target.open("rb") as handle:
        assert detect(handle) is FileKind.Png


def test_detect_reads_an_in_memory_stream() -> None:
    assert detect(io.BytesIO(PNG)) is FileKind.Png


def test_detect_reads_an_archive_entry(tmp_path: pathlib.Path) -> None:
    """A ``ZipExtFile`` is the case that has no path to hand over.

    It is a stream, so detection has to work from the object rather than the
    name, and the entry is compressed data until it is read.
    """
    archive = tmp_path / "bundle.zip"
    with zipfile.ZipFile(archive, "w") as zf:
        zf.writestr("inner.png", PNG)
    with zipfile.ZipFile(archive) as zf:
        with zf.open("inner.png") as entry:
            assert detect(entry) is FileKind.Png


def test_detect_on_a_stream_honours_the_limit() -> None:
    """A stream is read no further than the window, exactly as a path is.

    The full window reaches the ISO magic; 2 KB of the same data cannot, and
    neither can a 2 KB limit over the whole of it.
    """
    assert detect(io.BytesIO(build_iso(bytes_read()))) is FileKind.ISO
    assert detect(io.BytesIO(build_iso(2_048))) is None
    assert detect(io.BytesIO(build_iso(bytes_read())), max_bytes_read=2_048) is None


def test_a_path_is_still_a_path() -> None:
    """The union must not have cost the original call its behaviour."""
    import tempfile

    with tempfile.TemporaryDirectory() as directory:
        target = pathlib.Path(directory) / "image.png"
        target.write_bytes(PNG)
        assert detect(target) is FileKind.Png
        assert detect(str(target)) is FileKind.Png
        assert detect(pathlib.Path(target)) is FileKind.Png


def test_the_read_size_and_the_classification_agree(tmp_path: pathlib.Path) -> None:
    """`detect(path, n)` must be `detect_bytes(read_header(path, n), n)`.

    Two code paths that mean the same thing have to mean it for the same
    values, or a caller who picks one over the other is choosing a different
    answer rather than a different spelling.
    """
    target = tmp_path / "image.png"
    target.write_bytes(PNG + b"\x00" * 5_000)
    for window in (0, 1, 16, 2_048, 4_096, bytes_read(), 100_000):
        assert detect(target, max_bytes_read=window) is detect_bytes(
            read_header(target, max_bytes=window), max_bytes_read=window
        ), f"the two paths disagree at a window of {window}"


def test_the_limit_reaches_the_iso_offsets(tmp_path: pathlib.Path) -> None:
    """The three ISO signatures are 2 KB apart, so the window has to reach all.

    Not a detection claim — the read size is already pinned above — but a
    statement of which offsets the table actually uses, so a change to them
    shows up here rather than as a mystery in someone else's short buffer.
    """
    assert bytes_read() == max(ISO_MAGIC_OFFSETS) + len(ISO_MAGIC)

