"""The fluent spelling, and the claims made about it.

:class:`~magical_py.Detected` exists so a scan can be written with the file first.
The risks in a wrapper like this are not that it is wrong — every method is a call
to something the binding already had — but that it is *wrong in the ways a wrapper
can be*: reading the file twice, answering "is it this format?" with "is it
whatever came first?", and holding a cached answer that belongs to a window the
caller has since changed.

So most of what is here is those three, tested by breaking them.
"""

from __future__ import annotations

import io
import pathlib

import pytest

from magical_py import (
    DEFAULT_MAX_BYTES_READ,
    Detected,
    FileKind,
    bytes_read,
    detect_bytes,
    detected,
)

_PNG = b"\x89PNG\r\n\x1a\n"
_GIF = b"GIF89a"


#: The crate's own fixtures, which are real files rather than bytes in a variable.
#: A path and the same bytes have to agree, or the wrapper is only correct for one
#: of the two things a caller will pass it.
_TESTS = pathlib.Path(__file__).resolve().parents[3] / "crates" / "magical_rs" / "tests"


def test_a_path_and_the_same_bytes_agree() -> None:
    """The wrapper accepts both, and they are not two answers to one question."""
    path = _TESTS / "1.png"
    data = path.read_bytes()

    assert detected(path).kind is FileKind.Png
    assert detected(data).kind is FileKind.Png
    assert detected(path).kind is detected(data).kind


def test_it_reads_nothing_until_a_method_needs_an_answer() -> None:
    """A wrapper that read on construction would make ``detected()`` a read.

    The point of deferring is that building one is free, so a caller who only
    passes it along never touches the disk. A path that does not exist proves it
    without counting syscalls.
    """
    missing = _TESTS / "no-such-file-anywhere.png"
    assert not missing.exists()

    handle = detected(missing)  # must not raise
    assert isinstance(handle, Detected)

    with pytest.raises((FileNotFoundError, OSError)):
        handle.kind


def test_the_bytes_are_read_once_however_many_methods_are_called() -> None:
    """Two methods on one object, one read.

    Counted on a stream rather than on a path, because a stream is the case where
    a second read is not merely wasteful but wrong: a socket or a pipe has its
    bytes once.
    """
    stream = io.BytesIO(_PNG + b"\x00" * 4096)
    reads: list[int] = []
    inner = stream.read

    def counting_read(size: int = -1) -> bytes:
        reads.append(size)
        return inner(size)

    stream.read = counting_read  # type: ignore[method-assign]
    handle = detected(stream)

    assert handle.kind is FileKind.Png
    assert handle.matches(FileKind.Png)
    assert handle.mime == "image/png"
    assert handle.rule is not None

    assert len(reads) == 1, f"read {len(reads)} times: {reads}"


def test_matches_asks_about_the_named_format_not_the_first_one() -> None:
    """`matches` and `kind` are different questions, and KTX is the case that shows it.

    ``Ktx``'s declared signature is ``AB 4B 54 58 20`` — five bytes, and a prefix
    of ``Ktx2``'s twelve. So a KTX2 file *is* a KTX as far as any single rule is
    concerned, and is only ever reported as KTX2 because ``Ktx2`` sits earlier in
    the table and detection stops at the first match.

    That is the whole reason this method exists, and the reason it cannot be
    written ``kind is kind``: both answers are correct and they differ.
    """
    two = detected(_KTX2_HEADER)
    assert two.kind is FileKind.Ktx2
    assert two.matches(FileKind.Ktx2) is True
    # The prefix, and the reason detection cannot answer Ktx here.
    assert two.matches(FileKind.Ktx) is True

    one = detected(_KTX_HEADER)
    assert one.kind is FileKind.Ktx
    assert one.matches(FileKind.Ktx) is True
    # The other direction is false: five bytes do not contain a twelve-byte magic.
    assert one.matches(FileKind.Ktx2) is False


def test_matches_any_takes_varargs_or_one_iterable() -> None:
    """Both spellings, because a caller has one or the other and not both."""
    data = _PNG
    assert detected(data).matches_any(FileKind.GIF, FileKind.Png) is True
    assert detected(data).matches_any([FileKind.GIF, FileKind.Png]) is True
    assert detected(data).matches_any((FileKind.GIF, FileKind.Png)) is True
    assert detected(data).matches_any(FileKind.GIF, FileKind.Jpg) is False
    assert detected(data).matches_any([]) is False
    # A generator, consumed once rather than once per format.
    assert detected(data).matches_any(k for k in (FileKind.GIF, FileKind.Png)) is True


def test_within_changes_the_answer_and_says_which_window_it_is_in() -> None:
    """A window is a claim about what was read, and `within` re-classifies.

    ISO 9660 keeps its magic at offset 32,769 and declares a read size of 36,870,
    so a buffer holding it classified in a 2,048-byte window is a statement about
    the window rather than about the file.
    """
    iso = bytearray(bytes_read())
    iso[32_769:32_774] = b"CD001"

    handle = detected(bytes(iso))
    assert handle.kind is FileKind.ISO

    assert handle.window == bytes_read()
    assert handle.within(2048).kind is None
    assert handle.window == 2048
    # And back, because the bytes were read at the wider window.
    assert handle.within(bytes_read()).kind is FileKind.ISO


def test_within_returns_self_so_it_chains() -> None:
    """The chaining is the whole reason the method returns anything."""
    handle = detected(_PNG)
    assert handle.within(2048) is handle


def test_a_changed_window_is_not_answered_from_the_cache() -> None:
    """The bug this exists to catch: a cached kind outliving its window.

    Reading `kind`, narrowing the window, and reading `kind` again has to give two
    different answers. A cache keyed on nothing would give the first one twice.
    """
    iso = bytearray(bytes_read())
    iso[32_769:32_774] = b"CD001"

    handle = detected(bytes(iso))
    first = handle.kind
    assert first is FileKind.ISO

    assert handle.within(2048).kind is None
    assert handle.kind is None, "the cached ISO survived a narrower window"

    assert handle.within(bytes_read()).kind is FileKind.ISO


def test_the_repeated_window_is_the_same_object() -> None:
    """`within` with no change must not throw the cache away for nothing."""
    handle = detected(_PNG)
    assert handle.within(bytes_read()).kind is FileKind.Png
    assert handle.kind is FileKind.Png


def test_a_negative_window_is_rejected_at_the_front_door() -> None:
    """The message matches the rest of the package, so one wording is learned once."""
    with pytest.raises(ValueError, match="max_bytes_read cannot be negative"):
        detected(_PNG, max_bytes_read=-1)
    with pytest.raises(ValueError, match="max_bytes_read cannot be negative"):
        detected(_PNG).within(-1)


def test_the_metadata_methods_answer_for_no_match_too() -> None:
    """`None` where there is a value and a string where there is always one.

    Every kind has a display name and most have a MIME and an extension, so the
    three properties answer differently when nothing matched. A caller printing a
    result should not have to branch four times.
    """
    junk = detected(b"." * 64)

    assert junk.kind is None
    assert bool(junk) is False
    assert junk.mime is None
    assert junk.extension is None
    assert junk.rule is None
    assert junk.description == "no match"
    assert repr(junk) == "<Detected no match>"


def test_the_metadata_methods_agree_with_the_kind_they_describe() -> None:
    """No answer is recomputed; each one is the enum's own."""
    handle = detected(_PNG)
    kind = handle.kind
    assert kind is not None

    assert handle.mime == kind.mime
    assert handle.extension == kind.extension
    assert handle.description == kind.description
    assert handle.rule == kind.rule


def test_it_answers_the_same_as_detect_on_every_fixture() -> None:
    """The wrapper is a spelling, so it cannot answer anything else.

    Every file the crate ships, through both entry points, plus the same bytes
    with no path at all. A wrapper that drifts from the function it wraps is worse
    than no wrapper, because it is trusted to be the same.
    """
    fixtures = sorted(_TESTS.glob("*.*"))
    assert len(fixtures) >= 4, f"only found {fixtures}"

    for fixture in fixtures:
        data = fixture.read_bytes()
        expected = detect_bytes(data)
        assert detected(data).kind is expected, fixture.name
        assert detected(fixture).kind is expected, fixture.name


def test_it_agrees_with_detect_on_buffers_of_every_length() -> None:
    """A short buffer is where a window is most likely to be misread.

    From empty to past the ISO magic, every length, because a wrapper that reads
    one byte too few or one byte too many differs from `detect` only at the ends.
    """
    iso = bytearray(bytes_read())
    iso[32_769:32_774] = b"CD001"
    assert detected(bytes(iso)).kind is FileKind.ISO

    for length in (0, 1, 7, 8, 9, 2_048, 32_774, 36_869, 36_870, 40_000):
        truncated = bytes(iso[:length])
        assert detected(truncated).kind is detect_bytes(truncated), f"{length} bytes"


def test_it_matches_detect_for_noise() -> None:
    """Unrecognised input, where the answer is `None` and the wrapper must not guess."""
    for seed in range(64):
        data = bytes(((seed * 37 + n * 11) & 0x7F) | 0x80 for n in range(512))
        assert detected(data).kind is detect_bytes(data), seed


def test_something_that_is_not_readable_is_rejected_with_a_reason() -> None:
    """The error names the three shapes that are accepted."""
    with pytest.raises(TypeError, match="takes a path, a readable object or bytes"):
        _ = detected(42).kind  # type: ignore[arg-type]


def test_it_rejects_a_readable_object_that_returns_nothing() -> None:
    """A stream that yields `None` is broken, and `bytes(None)` says so.

    Worth pinning because the alternative is a `TypeError` from deep inside the
    conversion with nothing about which of the three source shapes was wrong.
    """

    class Broken:
        def read(self, size: int = -1) -> bytes:
            return None  # type: ignore[return-value]

    with pytest.raises(TypeError):
        _ = detected(Broken()).kind


def test_the_default_window_is_not_the_crates_default_window() -> None:
    """Two numbers that are both called "default" and are 36,870 and 2,048.

    `window` reports which is in force so a caller does not have to remember, and
    this test fails if the two ever collapse into one name.
    """
    assert detected(_PNG).window == bytes_read()
    assert bytes_read() == 36_870
    assert DEFAULT_MAX_BYTES_READ == 2_048
    assert detected(_PNG, max_bytes_read=DEFAULT_MAX_BYTES_READ).window == 2_048


def test_it_is_exported_and_is_the_documented_class() -> None:
    """The name in `__all__` is the name a caller types."""
    import magical_py

    assert "detected" in magical_py.__all__
    assert "Detected" in magical_py.__all__
    assert magical_py.detected is detected
    assert isinstance(detected(_PNG), Detected)


# KTX and KTX2 share a magic prefix, which is the only case in the table where
# `is` and `kind` can disagree about a file that really is the smaller format.
_KTX_HEADER = b"\xabKTX "
_KTX2_HEADER = b"\xabKTX 20\xbb\r\n\x1a\n"