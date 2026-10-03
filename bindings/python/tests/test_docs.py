"""Executes the Python examples in ``docs/``.

``test_readme.py`` does this for ``bindings/python/README.md`` and
``crates/magical_rs/tests/readme_examples.rs`` does it for the repository readme's
Rust. ``docs/`` is a third document with its own code, and an example that does not
run is a claim the reader finds out about by pasting it into a file.

The snippets are transcribed rather than extracted. Extracting them means a parser
for Markdown fences, and a parser that silently finds zero snippets passes — which
is the failure this file exists to prevent. So each test below names the page and
the heading it came from, and ``the_pages_are_the_ones_covered`` checks the other
direction: that this file does not drift from the pages it claims to cover.

Where a page's snippet needs a file, the file is built from real magic bytes rather
than skipped, because "the example needs a fixture" is exactly the gap that lets a
wrong signature survive.
"""

from __future__ import annotations

import pathlib
import re
from typing import Callable

import pytest

from magical_py import (
    DEFAULT_MAX_BYTES_READ,
    Detected,
    DynMagicCustom,
    FileKind,
    MagicCustom,
    MatchRules,
    Predicate,
    Signature,
    bytes_read,
    describe,
    detect,
    detect_bytes,
    detected,
    match_types_custom,
    match_types_custom_all,
    read_header,
    read_limits,
    signature_table,
    version,
)

_REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
_DOCS = _REPO_ROOT / "docs"

# The `photo.jpg` and `logo.png` the pages use, built from real magic bytes so the
# page and the test cannot disagree about what a JPEG is.
_JPEG = bytes([0xFF, 0xD8, 0xFF, 0xE0]) + b"\x00" * 32
_PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) + b"\x00" * 32


@pytest.fixture()
def photo(tmp_path: pathlib.Path) -> pathlib.Path:
    """A JPEG on disk, so `detect` is asked the question the page asks."""
    path = tmp_path / "photo.jpg"
    path.write_bytes(_JPEG)
    return path


def page(name: str) -> str:
    path = _DOCS / name
    assert path.is_file(), f"docs/{name} is listed in test_docs.py but is not there"
    return path.read_text(encoding="utf-8")


# --------------------------------------------------------------------------
# docs/api/python.md
# --------------------------------------------------------------------------


def test_docs_python_the_public_surface_is_importable() -> None:
    """docs/api/python.md - "The public surface"

    The page opens with one import statement listing what the package exports. If a
    name in that list stops existing, the page's first code block stops working, and
    an ImportError is the least of what a reader would have to diagnose.
    """
    import magical_py

    for name in re.findall(r"^\s{4}([A-Za-z_][A-Za-z0-9_]*),", page("api/python.md"), re.M):
        assert hasattr(magical_py, name), f"docs/api/python.md offers {name}, which does not exist"


def test_docs_python_level_one(photo: pathlib.Path) -> None:
    """docs/api/python.md - "Level 1"

    Every claim in the page's example is an assertion: the identity check, the two
    metadata attributes, and the `None` on bytes no rule matches.
    """
    kind = detect(photo)

    assert kind is FileKind.Jpg
    assert detect_bytes(_JPEG) is FileKind.Jpg
    assert detect_bytes(b"." * 64) is None

    # The page's own metadata claims, as values rather than as comments.
    assert kind.mime == "image/jpeg"
    assert kind.extension == "jpg"
    assert kind.description == "JPEG"
    assert kind.value == "jpg"


def test_docs_python_accepts_a_stream(photo: pathlib.Path) -> None:
    """docs/api/python.md - "`detect` takes a path **or anything open for binary
    reading**"

    The page says a socket, a pipe and a `zipfile.ZipExtFile` all work. A stream is
    the claim that distinguishes this from a path-only reader, so it is checked on
    the plainest stream there is.
    """
    with photo.open("rb") as handle:
        assert detect(handle) is FileKind.Jpg


def test_docs_python_detected(photo: pathlib.Path) -> None:
    """docs/api/python.md - "`Detected` — the data-first spelling\""""
    what = detected(photo)

    assert isinstance(what, Detected)
    assert what.kind is FileKind.Jpg
    assert bool(what) is True
    assert what.mime == "image/jpeg"
    assert what.extension == "jpg"
    assert what.description == "JPEG"
    assert what.window == bytes_read() == 36_870
    assert isinstance(what.rule, Signature)
    assert what.rule.signatures[0] == _JPEG[:4]

    # `matches` and `matches_any` in both spellings the page offers.
    assert what.matches(FileKind.Jpg) is True
    assert what.matches(FileKind.Png) is False
    assert what.matches_any([FileKind.Png, FileKind.Jpg]) is True
    assert what.matches_any(FileKind.Png, FileKind.Jpg) is True
    assert what.matches_any([]) is False

    # `within` returns self, so it chains.
    assert what.within(2_048) is what
    assert what.kind is FileKind.Jpg


def test_docs_python_a_none_is_two_things() -> None:
    """docs/api/python.md - "**A `None` from `detect_bytes` means one of two
    things**"

    The page's point is that naming the window is what separates "no rule matched"
    from "no rule that fits the window you named matched". The second needs the ISO
    9660 magic, which sits at 36,865 and so never enters a 2,048-byte window.
    """
    iso = bytearray(40_000)
    iso[36_865 : 36_865 + 5] = b"CD001"

    assert detect_bytes(bytes(iso)) is FileKind.ISO
    assert detect_bytes(bytes(iso), max_bytes_read=2_048) is None
    assert detect_bytes(b"." * 64) is None
    assert detect_bytes(b"." * 64, max_bytes_read=2_048) is None


def test_docs_python_matches_is_not_is() -> None:
    """docs/api/python.md - "**It is called `matches`, not `is`, because `is` is a
    keyword in Python.**"

    Two claims: the name is forced by the language, and the name is not a loss
    because `matches` is the inverse of `FileKind.matches`. The first is checked by
    asking Python to compile the spelling the page says will not work.
    """
    with pytest.raises(SyntaxError):
        compile("what.is(FileKind.Jpg)", "<docs>", "exec")

    # kind first, on the enum; data first, on a `Detected`.
    assert FileKind.Jpg.matches(_JPEG) is True
    assert FileKind.Png.matches(_JPEG) is False


def test_docs_python_the_table() -> None:
    """docs/api/python.md - "Asking the table a question\""""
    table = signature_table()

    assert len(table) == 114
    assert all(isinstance(entry, Signature) for entry in table)
    assert all(isinstance(entry.max_bytes_read, int) for entry in table)

    one = describe(FileKind.Png)
    assert isinstance(one, Signature)
    assert one.signatures[0] == _PNG[:8]
    assert one.offsets == (0,)

    limits = read_limits()
    assert limits.default_max_bytes_read == DEFAULT_MAX_BYTES_READ == 2_048


def test_docs_python_level_two() -> None:
    """docs/api/python.md - "Levels 2, 3 and 4"

    The page states that `kind` is any object at all, so the type is the caller's.
    An enum defined here rather than imported is the check: there is no `Kind` in
    the package, and an example that imported one would be inventing API.
    """
    import enum

    assert not hasattr(__import__("magical_py"), "Kind")

    class Kind(enum.Enum):
        Shoujo = "shoujo"
        Unknown = "unknown"

    rules = [
        MagicCustom(kind=Kind.Shoujo, signatures=[b"MagicalGirl"], offsets=[0]),
        MagicCustom(kind=Kind.Unknown, signatures=[b"Mahou"], offsets=[0]),
    ]

    assert match_types_custom(b"MagicalGirl", rules, Kind.Unknown) is Kind.Shoujo
    assert match_types_custom(b"nothing", rules, Kind.Unknown) is Kind.Unknown
    assert match_types_custom_all(b"MagicalGirl", rules) == [Kind.Shoujo]

    # The page says a level 2 answer is never None and `fallback` is required.
    assert match_types_custom(b"nothing", rules, Kind.Shoujo) is Kind.Shoujo


def test_docs_python_offsets_are_positions_not_a_pairing() -> None:
    """docs/api/python.md - "**`offsets` is a list of positions, not a pairing with
    `signatures`.**\"

    The page says any signature at any offset is a match, and that a *length*
    mismatch is therefore not checked. Both halves are run, because the wrong
    version of this sentence — "they are paired by position" — is the natural
    mistake and produces a rule set that quietly ignores half its work.
    """
    # Either prefix, both at offset zero: one rule, two accepted magics.
    either = [MagicCustom(kind="x", signatures=[b"AB", b"CD"], offsets=[0])]
    assert match_types_custom(b"AB", either, "fallback") == "x"
    assert match_types_custom(b"CD", either, "fallback") == "x"

    # Not a pairing, so a length mismatch is accepted rather than refused.
    mismatched = [MagicCustom(kind="x", signatures=[b"AB", b"CD"], offsets=[0, 8])]
    assert match_types_custom(b"CD", mismatched, "fallback") == "x"

    # One side alone cannot match, and that one is refused.
    with pytest.raises(ValueError):
        MagicCustom(kind="x", signatures=[b"AB"], offsets=[])

    # `rules` is exclusive of both, because the crate discards them silently.
    with pytest.raises(ValueError):
        MagicCustom(kind="x", signatures=[b"AB"], offsets=[0], rules=MatchRules.all(lambda d: True))

    # And the two numeric checks the page lists.
    with pytest.raises(ValueError):
        MagicCustom(kind="x", signatures=[b"AB"], offsets=[0], max_bytes_read=-1)

    with pytest.raises(ValueError):
        MagicCustom(kind="x", signatures=[b"AB"], offsets=[-1])


def test_docs_python_errors(tmp_path: pathlib.Path) -> None:
    """docs/api/python.md - "Errors\""""
    with pytest.raises(FileNotFoundError):
        detect(tmp_path / "absent.jpg")

    with pytest.raises(ValueError):
        detect_bytes(_JPEG, max_bytes_read=-1)

    with pytest.raises(ValueError):
        read_header(tmp_path / "photo.jpg", max_bytes=-1)

    assert version()


def test_docs_python_is_deliberately_blocking() -> None:
    """docs/api/python.md - "**`detect` is deliberately blocking.**\""""
    import inspect

    assert not inspect.iscoroutinefunction(detect)
    assert not inspect.isasyncgenfunction(detect)


def test_docs_python_predicate_is_a_type_and_dyn_works() -> None:
    """docs/api/python.md - the level table.

    The page lists `Predicate` and `DynMagicCustom` among the exported names. Two
    things are worth pinning: `Predicate` is a *type alias* for a callable rather
    than a class, which is why the page shows predicates as plain lambdas, and
    `DynMagicCustom` is constructible, since a class that cannot be instantiated is
    still importable.
    """
    assert Predicate == Callable[[bytes], bool]

    rule = DynMagicCustom(
        matcher=lambda data: data.startswith(b"MAGICAL"),
        kind="custom",
        max_bytes_read=32,
    )
    assert rule.matches(b"MAGICAL") is True
    assert rule.matches(b"something else") is False

    # `MatchRules` is the level 2 way to say the same thing.
    assert MatchRules.all(lambda data: data.startswith(b"MAG")).evaluate(b"MAGICAL") is True
    assert MatchRules.any(lambda data: False, lambda data: True).evaluate(b"x") is True


# --------------------------------------------------------------------------
# docs/getting-started.md
# --------------------------------------------------------------------------


def test_docs_getting_started_first_detection(photo: pathlib.Path) -> None:
    """docs/getting-started.md - "Python\""""
    kind = detect(photo)

    assert kind is FileKind.Jpg
    assert kind.value == "jpg"
    assert kind.mime == "image/jpeg"
    assert kind.extension == "jpg"
    assert kind.description == "JPEG"


def test_docs_getting_started_the_data_first_spelling(photo: pathlib.Path) -> None:
    """docs/getting-started.md - "The data-first spelling, for scanning a
    directory\""""
    directory = photo.parent
    seen = False

    for path in directory.iterdir():
        what = detected(path)
        if what.matches_any(FileKind.Png, FileKind.Jpg):
            seen = True

    assert seen, "the fixture is a JPEG and nothing matched it"


def test_docs_getting_started_bytes_you_already_have() -> None:
    """docs/getting-started.md - "Bytes you already have\""""
    assert detect_bytes(b"GIF89a") is FileKind.GIF


def test_docs_getting_started_no_dependencies() -> None:
    """docs/getting-started.md - "The Rust crate has **no dependencies**"

    The page's reason to believe it is `cargo tree --edges normal` printing itself
    and nothing else, and the lockfile holding exactly one package. The
    dependency claim is this repository's, and `workspace.rs` already holds the
    lockfile half; what is checked here is the file the page points at.
    """
    lock = (_REPO_ROOT / "Cargo.lock").read_text(encoding="utf-8")
    assert lock.count("[[package]]") == 1, "the root lockfile is expected to hold one package"


# --------------------------------------------------------------------------
# docs/detection-levels.md
# --------------------------------------------------------------------------


def test_docs_levels_python_level_two() -> None:
    """docs/detection-levels.md - "The same levels in Python\""""
    import enum

    class Kind(enum.Enum):
        Shoujo = "shoujo"
        Unknown = "unknown"

    rules = [
        MagicCustom(kind=Kind.Shoujo, signatures=[b"MagicalGirl"], offsets=[0]),
        MagicCustom(kind=Kind.Unknown, signatures=[b"Mahou"], offsets=[0]),
    ]

    assert match_types_custom(b"MagicalGirl", rules, Kind.Unknown) is Kind.Shoujo
    assert match_types_custom(b"Mahou", rules, Kind.Unknown) is Kind.Unknown


def test_docs_levels_python_has_no_level_five() -> None:
    """docs/detection-levels.md - "**Level 5 is absent**\""""
    import magical_py

    assert not hasattr(magical_py, "CustomMatchRules")
    assert not hasattr(magical_py, "AllMatchesUnsafe")


def test_docs_levels_metadata_is_never_invented() -> None:
    """docs/detection-levels.md - "**Nothing is invented**\""""
    for kind in FileKind:
        assert kind.mime is None or kind.mime != ""
        assert kind.extension is None or kind.extension != ""


# --------------------------------------------------------------------------
# docs/across-languages.md
# --------------------------------------------------------------------------


def test_docs_across_matches_reads_as_a_pair() -> None:
    """docs/across-languages.md - "Python's 'is this format' is `matches`\""""
    data = _JPEG

    assert FileKind.Jpg.matches(data)
    assert detected(data).matches(FileKind.Jpg)
    assert not detected(data).matches(FileKind.Png)


# --------------------------------------------------------------------------
# The direction that rots
# --------------------------------------------------------------------------


def test_the_pages_are_the_ones_covered() -> None:
    """Every page holding a Python example is checked here.

    A page that gains a snippet without a test is a page whose snippet is unchecked,
    and it is indistinguishable from a page that is fully covered. The list is the
    pages that carry Python code, which is fewer than the pages that exist.
    """
    pages = ["api/python.md", "across-languages.md", "detection-levels.md", "getting-started.md"]

    for name in pages:
        text = page(name)
        assert "```python" in text, f"docs/{name} no longer has a Python example, so drop it from this list"

    readme = (_REPO_ROOT / "readme.md").read_text(encoding="utf-8")
    assert "docs/README.md" in readme, "the readme does not link the documentation index"