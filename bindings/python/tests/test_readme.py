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

import contextlib
import io
import pathlib
import pydoc
import re
import sys

import pytest

from magical_py import FileKind, bytes_read, detect, detect_bytes, version

_README = pathlib.Path(__file__).resolve().parents[1] / "README.md"

# The `photo.jpg` the readme uses, built from the bytes in the readme tables
# so the example and the test cannot disagree.
_JPEG = bytes([0xFF, 0xD8, 0xFF, 0xE0]) + b"\x00" * 32
_PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32


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
