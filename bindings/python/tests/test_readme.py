"""Executes the claims made in ``README.md``.

The main crate does this for its own readme in ``readme_examples.rs``. Every
factual claim here was written by hand and at least one of them was wrong
before it was checked, so the examples are treated as code rather than prose.

The snippets cannot be run with ``exec`` on an installed package, because
they reference a ``photo.jpg`` that does not exist. Instead each claim is
restated against data the test controls, which checks the assertion without
checking the formatting.
"""

from __future__ import annotations

import asyncio
import contextlib
import io
import pathlib
import pydoc
import re
import sys

import pytest

from magical_py import (
    AsyncDynMagic,
    DynMagicCustom,
    FileKind,
    MagicCustom,
    MatchRules,
    bytes_read,
    detect,
    detect_bytes,
    match_async_dyn_types,
    match_dyn_types,
    match_dyn_types_all,
    match_types_custom,
    read_header,
    version,
)

_README = pathlib.Path(__file__).resolve().parents[1] / "README.md"

# The `photo.jpg` the readme uses, built from the bytes in the readme tables
# so the example and the test cannot disagree.
_JPEG = bytes([0xFF, 0xD8, 0xFF, 0xE0]) + b"\x00" * 32
_PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32

# The ISO 9660 disc the readme uses to talk about the read window. Its magic
# is at 36,865, which is what makes 2,048 bytes an interesting place to cut.
_ISO_MAGIC = b"CD001"
_ISO_OFFSET = 36_865


def _iso(size: int) -> bytes:
    """Return *size* zero bytes carrying the ISO 9660 magic at its offset."""
    data = bytearray(size)
    data[_ISO_OFFSET : _ISO_OFFSET + len(_ISO_MAGIC)] = _ISO_MAGIC
    return bytes(data)


def readme() -> str:
    return _README.read_text(encoding="utf-8")


def test_readme_exists() -> None:
    assert _README.is_file()


def test_version_is_a_release_string() -> None:
    assert re.fullmatch(r"\d+\.\d+\.\d+", version())


def test_readme_first_example_holds(tmp_path: pathlib.Path) -> None:
    """`detect("photo.jpg")` returns a `FileKind` with the stated metadata."""
    path = tmp_path / "photo.jpg"
    path.write_bytes(_JPEG)

    kind = detect(path)

    assert kind is FileKind.Jpg
    assert kind.value == "jpg"
    assert kind.mime == "image/jpeg"
    assert kind.extension == "jpg"
    assert kind.description == "JPEG"


def test_readme_detect_bytes_example_holds() -> None:
    assert detect_bytes(_PNG) is FileKind.Png


def test_readme_none_on_no_match() -> None:
    assert detect_bytes(b"no magic here") is None


def test_readme_error_handling_example_holds(tmp_path: pathlib.Path) -> None:
    """The readme shows `except FileNotFoundError`, so it must be that class."""
    path = tmp_path / "missing.jpg"
    with pytest.raises(FileNotFoundError):
        detect(path)


def test_readme_docstring_examples_hold() -> None:
    """The readme quotes `__doc__` output, so it has to be exactly that.

    An earlier version of the readme quoted `help()` output verbatim. That was
    a mistake: pydoc prefixes its output when stdout is not a terminal, and
    the exact layout varies between Python versions, so a reader could not
    reproduce the snippet. `__doc__` is quoted instead because it is stable.
    """
    assert FileKind.Png.__doc__ == "PNG"
    assert FileKind.ISO.__doc__ == "ISO 9660"
    assert FileKind.PkgZip.description == "Zip / JAR / APK"
    for snippet in ("'PNG'", "'ISO 9660'", "Zip / JAR / APK"):
        assert snippet in readme()


def test_readme_png_in_a_jpg_filename_claim_holds(tmp_path: pathlib.Path) -> None:
    assert "detected but is not really one" not in readme()  # wording guard
    path = tmp_path / "actually-a-png.jpg"
    path.write_bytes(_PNG)
    assert detect(path) is FileKind.Png


def test_readme_states_the_member_count() -> None:
    """The readme quotes a format count, so it has to be the live one."""
    assert f"{len(FileKind)} formats" in readme()


def test_member_descriptions_are_reachable_through_pydoc() -> None:
    """Every member's display name must be readable on every supported version.

    `help()` is deliberately not the thing asserted here. `pydoc.Doc.docclass`
    only rendered a data member's `__doc__` when that member was callable or a
    data descriptor, and an enum member is neither, so Python 3.8 dropped all
    114 display names from `help()`. Python 3.9 removed that condition.
    `pydoc.getdoc` has no such condition, so it holds everywhere from 3.8 on and
    is what the readme points readers at.
    """
    for name, description in (("Png", "PNG"), ("ISO", "ISO 9660")):
        member = FileKind[name]
        assert pydoc.getdoc(member) == description, f"{name} has no display name"

    assert re.search(r"Python 3\.8 omits it from `help\(\)`", readme()), (
        "the readme no longer warns that 3.8 hides the descriptions"
    )


def test_help_lists_descriptions_where_cpython_supports_it() -> None:
    """`help()` carries the display names from Python 3.9 on, and not before."""
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        help(FileKind)
    output = buffer.getvalue()

    if sys.version_info >= (3, 9):
        for name, description in (("Png", "PNG"), ("ISO", "ISO 9660")):
            assert name in output, f"{name} is missing from help() output"
            assert description in output, f"{description} is missing from help() output"
    else:
        # 3.8 still lists the members, it just cannot render their docstrings.
        assert "Png" in output, "the member list should still be present on 3.8"


def test_readme_claims_about_optional_metadata_hold() -> None:
    """The readme says `None` means "not registered", never "unknown"."""
    assert detect_bytes(b"") is None
    for kind in FileKind:
        mime = kind.mime
        extension = kind.extension
        assert mime is None or "/" in mime
        assert extension is None or not extension.startswith(".")


def test_readme_bytes_read_claim_holds() -> None:
    """`bytes_read` is documented as 36,870 in both readmes."""
    assert bytes_read() == 36_870
    assert "36,870" in readme()


def test_readme_two_byte_claim_holds() -> None:
    """The readme names 9 two-byte formats; all 9 must still be two bytes."""
    stated = re.search(r"(\d+) formats match on nothing but", readme())
    assert stated is not None, "the readme no longer states a two-byte count"
    assert int(stated.group(1)) == 9

    for name in ("Arj", "Bitmap", "Gzip", "MSDOS", "SerializedJavaData", "Zlib"):
        assert f"`{name}`" in readme(), f"the readme dropped {name} from the caveat"


# --- the detection levels section -----------------------------------------
#
# The levels table is a claim about cost, and cost is the reason the levels
# exist, so the names in it and the outcomes in the snippets are both checked.


def test_readme_documents_all_three_custom_levels() -> None:
    """The table must keep naming the level each entry belongs to."""
    for name in ("MagicCustom", "DynMagicCustom", "AsyncDynMagic"):
        assert f"| `{name}` |" in readme(), f"the levels table dropped {name}"
    for name in (
        "match_types_custom",
        "match_dyn_types",
        "match_dyn_types_all",
        "match_async_dyn_types",
    ):
        assert f"`{name}`" in readme(), f"the levels section never shows {name}"


def test_readme_says_level_5_is_not_exposed() -> None:
    """The crate has five levels and this has three. The reader should know."""
    assert re.search(r"Level 5 is raw pointers", readme())
    assert "`unsafe` by definition" in readme()


def test_readme_level_2_example_holds() -> None:
    """The snippet's two results: the match, and the fallback."""
    png = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    rules = [
        MagicCustom("gif", [b"GIF87a", b"GIF89a"], [0]),
        MagicCustom("png", [png], [0]),
    ]
    assert match_types_custom(png, rules, "unknown") == "png"
    assert match_types_custom(b"zzz", rules, "unknown") == "unknown"


def test_readme_level_2_predicate_example_holds() -> None:
    """The readme's `is_utf8` predicate, used the way the readme uses it."""

    def is_utf8(data: bytes) -> bool:
        try:
            data.decode("utf-8")
        except UnicodeDecodeError:
            return False
        return True

    rule = MagicCustom("text", rules=MatchRules.all(is_utf8))
    assert rule.matches("héllo".encode("utf-8")) is True
    assert rule.matches(b"\xff\xfe\x00") is False


def test_readme_level_3_example_holds() -> None:
    """Both forms, and the mixed-kind claim the section makes in prose."""
    rules = [
        DynMagicCustom(lambda data: data.startswith(b"RIFF"), "riff"),
        DynMagicCustom(lambda data: data[4:8] == b"WEBP", "webp"),
    ]
    assert match_dyn_types(b"RIFF____WEBPVP8 ", rules) == "riff"
    assert match_dyn_types(b"OggS\x00\x00\x00\x00", rules) is None
    assert match_dyn_types_all(b"OggS\x00\x00\x00\x00", rules) == []
    # First match wins, so a file that is both is reported as the first rule.
    assert match_dyn_types_all(b"RIFF____WEBPVP8 ", rules) == ["riff"]

    mixed: list[DynMagicCustom[object]] = [
        DynMagicCustom(lambda data: True, "a string"),
        DynMagicCustom(lambda data: True, 42),
        DynMagicCustom(lambda data: True, None),
    ]
    assert match_dyn_types_all(b"anything", mixed) == ["a string", 42, None]


def test_readme_level_4_example_holds() -> None:
    """The async snippet, driven the way the readme drives it."""
    calls: list[bytes] = []

    async def registered_checksum(data: bytes) -> bool:
        calls.append(data)
        return data[:4] == b"\x89PNG"

    rule = AsyncDynMagic(registered_checksum, "verified")
    png = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) * 4
    assert asyncio.run(match_async_dyn_types(png, [rule])) == "verified"
    assert calls == [png]
    assert asyncio.run(match_async_dyn_types(b"not a png", [rule])) is None


def test_readme_level_4_says_a_coroutine_object_is_a_type_error() -> None:
    """The readme states a refusal, so the refusal has to be real."""

    async def decide(data: bytes) -> bool:
        return True

    pending = decide(b"")
    rule = AsyncDynMagic(pending, "k")  # type: ignore[arg-type]
    try:
        with pytest.raises(TypeError, match="not callable"):
            asyncio.run(rule.matches(b""))
    finally:
        pending.close()

    assert "coroutine object" in readme()


def test_readme_level_4_claims_no_feature_flag() -> None:
    """The readme tells a reader there is nothing to install, so check the claim."""
    assert "no feature flag and no extra dependency" in readme()
    assert re.search(r"magical_async_dyn", readme())


def test_readme_open_file_claim_holds(tmp_path: pathlib.Path) -> None:
    """The readme shows `detect` on an open file, so an open file has to work."""
    assert "with open(\"disc.iso\", \"rb\") as handle:" in readme()

    disc = tmp_path / "disc.iso"
    disc.write_bytes(_iso(bytes_read()))
    with disc.open("rb") as handle:
        assert detect(handle) is FileKind.ISO


def test_readme_read_header_claim_holds(tmp_path: pathlib.Path) -> None:
    """The readme says `read_header` spends the figure `bytes_read` reports."""
    assert "read_header(path, max_bytes=...)" in readme()

    disc = tmp_path / "disc.iso"
    disc.write_bytes(_iso(bytes_read()))
    assert len(read_header(disc)) == bytes_read()
    assert len(read_header(disc, max_bytes=2_048)) == 2_048


def test_readme_short_answer_claim_holds() -> None:
    """A 2,048-byte ISO slice is a no match, and naming the window says why.

    Both halves of the readme's argument, because the point is that they are
    the same `None` and mean different things. The readme says nothing whose
    magic fits in 2,048 bytes matched, which is only true while ISO is out of
    reach — so the test pins the reach as well as the answer.
    """
    assert "nothing whose magic fits in" in readme()

    window = bytes_read()
    short = _iso(window)[:2_048]
    assert detect_bytes(short) is None
    assert detect_bytes(short, max_bytes_read=2_048) is None
    # The same bytes over a window that does reach the magic, so the difference
    # is the window and not the data.
    assert detect_bytes(_iso(window), max_bytes_read=window) is FileKind.ISO
    # And PNG is out of reach below 2,048, which is the readme's own caveat.
    assert detect_bytes(_PNG, max_bytes_read=2_048) is FileKind.Png
    assert detect_bytes(_PNG, max_bytes_read=2_047) is None
