"""Tests for :func:`magical_py.detect` and :func:`magical_py.detect_bytes`.

The fixtures here are written to ``tmp_path`` from magic bytes rather than
checked in as binaries. A committed ``1.png`` in the repository would be a
second copy of the truth that can drift, and it would bloat the wheel's test
suite. Building a header from the documented bytes also makes the test state
exactly which signature it is exercising.
"""

from __future__ import annotations

import pathlib

import pytest

from magical_py import FileKind, bytes_read, detect, detect_bytes

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
