"""Keeps the Python enum and the Rust detection table in step.

``FileKind`` is declared in Python while the signatures live in Rust, so the
two can drift apart. ``readme_coverage.rs`` guards the readme against the
Rust table, and this file guards the Python enum against the compiled
extension. Together they close the loop: a format added on the Rust side
without a matching readme row and enum member fails a test rather than
shipping a gap.
"""

from __future__ import annotations

import pathlib

import pytest

from magical_py import FileKind
from magical_py import _magical_rs

# The readme tables are the source of the display names, and this test only
# makes sense inside the repository. A wheel has no readme, and no Rust
# sources either, so skip rather than fail when installed from PyPI.
_REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
_README = _REPO_ROOT / "readme.md"
# The generated enum, by the path gen_kinds.ps1 writes it to.
_GENERATED = pathlib.Path(__file__).resolve().parents[1] / "python" / "magical_py"


def test_generated_enum_is_written_without_a_bom() -> None:
    """`gen_kinds.ps1` writes this file, and it ships inside the wheel.

    PowerShell 5.1's `Set-Content -Encoding utf8` prepends a UTF-8 BOM, which
    Python tolerates and nothing else notices. A silent generator regression
    would put that byte back into every published wheel, so it is asserted.
    """
    source = _GENERATED / "_kinds.py"
    if not source.is_file():
        pytest.skip("not running from a source checkout")
    assert not source.read_bytes().startswith(b"\xef\xbb\xbf"), (
        "_kinds.py starts with a UTF-8 BOM; gen_kinds.ps1 must write it without one"
    )


def test_every_rust_kind_is_a_python_member() -> None:
    rust_names = _magical_rs.all_kinds()
    unknown = sorted(set(rust_names) - set(FileKind.__members__))
    assert not unknown, (
        "the Rust detection table can return these kinds, but FileKind has no "
        f"member for them: {unknown}. Add them to gen_kinds.ps1 and regenerate."
    )


def test_every_python_member_is_reachable_from_rust() -> None:
    rust_names = set(_magical_rs.all_kinds())
    unreachable = sorted(set(FileKind.__members__) - rust_names)
    assert not unreachable, (
        f"FileKind advertises these members, but no signature produces them: {unreachable}. "
        "Either add a signature to magical_rs or remove the member."
    )


def test_member_counts_agree() -> None:
    assert len(FileKind) == len(_magical_rs.all_kinds())


def test_variant_names_are_unique_in_rust() -> None:
    """Several formats share one entry's magic, so this guards the mapping.

    ``all_kinds`` returns one name per table entry, and the tables are 1:1
    with the variants. A repeat here would mean a table entry had been given
    a kind that another entry already claims.
    """
    names = _magical_rs.all_kinds()
    assert len(names) == len(set(names))


def test_readme_rows_match_the_enum() -> None:
    """The readme advertises the formats; the enum must match it exactly.

    ``readme_coverage.rs`` already checks the readme against the Rust table,
    so this closes the third side of the triangle. The tables are re-parsed
    rather than trusting a count written in prose, because a hard-coded
    number would need editing by hand and could go stale unnoticed.

    ``set()`` equality rather than a count alone: a table that dropped one row
    and gained another would keep the count at 114 while two formats were
    silently wrong.
    """
    if not _README.is_file():
        pytest.skip("not running from a source checkout")
    variants = _readme_variants()
    assert len(variants) == len(FileKind)
    assert variants == set(FileKind.__members__)


def _readme_variants() -> set[str]:
    variants: set[str] = set()
    for line in _README.read_text(encoding="utf-8").splitlines():
        cells = [cell.strip() for cell in line.split("|")]
        if len(cells) != 6 or not cells[1]:
            continue
        offsets = cells[4].split(",")
        if not all(part.strip().isdigit() for part in offsets):
            continue
        candidate = cells[2].strip("`")
        if candidate and all(c.isalnum() or c in "_-" for c in candidate):
            variants.add(candidate)
    return variants
