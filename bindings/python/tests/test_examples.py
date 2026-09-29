"""Runs every example in ``examples/`` and checks what each one claims.

The main crate verifies the code in its own readme with
``tests/readme_examples.rs``, and ``test_readme.py`` does the same for this
package's readme. An examples directory is the same kind of claim in a
different place, so it is held to the same standard: every example must run to
completion as a script, must depend on nothing but the standard library and
``magical_py`` so it still works for anyone who installed the wheel, and must
print the values it asserts.

The claims below are restated against the crate rather than copied out of an
earlier run, so the table cannot quietly go stale: ``len(FileKind)`` is read
from the enum and ``bytes_read()`` from the extension.

As in ``test_readme.py``, this checks the assertion and not the formatting.
Nothing here depends on how wide a column is, only on what the example said.
"""

from __future__ import annotations

import ast
import os
import pathlib
import re
import subprocess
import sys

import pytest

from magical_py import FileKind, bytes_read

_ROOT = pathlib.Path(__file__).resolve().parents[1]
_EXAMPLES = _ROOT / "examples"
_README = _ROOT / "README.md"

# Python 3.8, which the abi3-py38 wheel has to keep working for, has no
# `sys.stdlib_module_names`. The fallback covers what the examples import
# today; if 3.8 support ever ends it can become the real list.
_STDLIB = getattr(sys, "stdlib_module_names", None) or frozenset(
    {"__future__", "pathlib", "tempfile"}
)

EXAMPLE_NAMES = sorted(path.name for path in _EXAMPLES.glob("*.py"))


def _tally_by_media_type() -> dict[str, int]:
    """Restate the example's own count, so the claim is checked, not copied."""
    tally: dict[str, int] = {}
    for kind in FileKind:
        mime = kind.mime
        if mime is None:
            continue
        name = mime.split("/")[0]
        tally[name] = tally.get(name, 0) + 1
    return tally


def _imported_roots(path: pathlib.Path) -> set[str]:
    """Return the top-level module names *path* imports."""
    roots: set[str] = set()
    for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
        if isinstance(node, ast.Import):
            roots.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            # `level == 0` skips relative imports, which have no root module.
            if node.level == 0 and node.module is not None:
                roots.add(node.module.split(".")[0])
    return roots


def _run(name: str, cwd: pathlib.Path) -> str:
    """Run the example as a script from *cwd* and return what it printed."""
    environment = dict(os.environ)
    # A child's stdout encoding is the platform default, which on Windows is not
    # UTF-8. Pinning both ends keeps a non-ASCII character in an example from
    # failing this test for the wrong reason.
    environment["PYTHONIOENCODING"] = "utf-8"
    finished = subprocess.run(
        [sys.executable, str(_EXAMPLES / name)],
        cwd=cwd,
        env=environment,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
        check=False,
    )
    assert finished.returncode == 0, f"{name} exited {finished.returncode}:\n{finished.stderr}"
    assert finished.stderr == "", f"{name} wrote to stderr:\n{finished.stderr}"
    return finished.stdout


def _expected_claims() -> dict[str, list[str]]:
    """What each example has to show, restated against the crate."""
    no_mime = sorted(kind.name for kind in FileKind if kind.mime is None)
    no_extension = sorted(kind.name for kind in FileKind if kind.extension is None)
    return {
        "01_detect_a_file.py": [
            f"member: {FileKind.Jpg.name}",
            f"value: {FileKind.Jpg.value}",
            f"display: {FileKind.Jpg.__doc__}",
            f"media type: {FileKind.Jpg.mime}",
            f"extension: {FileKind.Jpg.extension}",
            "no signature matched",
        ],
        "02_detect_bytes.py": [
            f"bytes_read() is {bytes_read()}",
            f"{bytes_read()} bytes of header -> FileKind.{FileKind.ISO.name}",
            f"{bytes_read() - 1} bytes of header -> None",
            f"36865 bytes of header -> None",
            "2048 bytes of header -> None",
            "max_bytes_read=2048",
            f"ISO 9660 needs {bytes_read()}",
            f"{64 * 1024 * 1024:,} byte file, read {bytes_read():,} bytes",
            f"-> FileKind.{FileKind.ISO.name}",
            f"-> FileKind.{FileKind.ISO.name}, from the open file",
        ],
        "03_name_does_not_matter.py": [
            f"photo.jpg -> {FileKind.Png.name}",
            "notes.png -> no signature matched",
            f"logo -> {FileKind.Png.name}",
            f"FileKind.Png.extension is {FileKind.Png.extension!r}",
        ],
        "04_list_supported_formats.py": [
            f"{len(FileKind)} formats are detectable by this build.",
            *[f"{name}: {count}" for name, count in _tally_by_media_type().items()],
            f"{len(no_mime)} formats have no registered media type",
            ", ".join(no_mime),
            f"{len(no_extension)} formats have no conventional extension",
            ", ".join(no_extension),
            f"FileKind.Png.value: {FileKind.Png.value!r}",
            f"FileKind.from_value({FileKind.Png.value!r}): {FileKind.Png.name}",
            f"FileKind[{FileKind.Png.name!r}]: {FileKind.Png.name}",
        ],
        "05_scan_a_directory.py": [
            "6 files under the tree, tallied by kind:",
            f"{FileKind.GIF.__doc__} x 1 ({FileKind.GIF.mime})",
            f"{FileKind.Jpg.__doc__} x 1 ({FileKind.Jpg.mime})",
            f"{FileKind.PDF.__doc__} x 1 ({FileKind.PDF.mime})",
            f"{FileKind.PkgZip.__doc__} x 1 ({FileKind.PkgZip.mime})",
            f"{FileKind.Png.__doc__} x 1 ({FileKind.Png.mime})",
            "1 of them matched no signature:",
            "notes.txt (",
        ],
        "06_handle_errors.py": [
            f"an existing PNG: returned {FileKind.Png.name}",
            "bytes with no magic: returned None, no signature matched",
            "a path that is absent: raised FileNotFoundError",
            # Which of the two arrives depends on the platform, so the claim
            # being checked here is that a directory raises at all rather than
            # returning None.
            "a directory: raised ",
        ],
    }


def test_examples_directory_is_not_empty() -> None:
    assert EXAMPLE_NAMES, "bindings/python/examples holds no .py files"


@pytest.mark.parametrize("name", EXAMPLE_NAMES)
def test_every_example_runs_as_a_script(name: str, tmp_path: pathlib.Path) -> None:
    """A runnable example is the only kind worth documenting."""
    assert _run(name, tmp_path)


@pytest.mark.parametrize("name", EXAMPLE_NAMES)
def test_every_example_prints_what_it_claims(
    name: str, tmp_path: pathlib.Path
) -> None:
    output = _run(name, tmp_path)
    claims = _expected_claims()
    assert name in claims, f"{name} has no claims recorded for it"
    missing = [claim for claim in claims[name] if claim not in output]
    assert not missing, f"{name} did not print {missing}\n\nit printed:\n{output}"


@pytest.mark.parametrize("name", EXAMPLE_NAMES)
def test_every_example_imports_nothing_else(name: str) -> None:
    """An example needing a dev dependency cannot be run by an installed user."""
    allowed = set(_STDLIB) | {"magical_py"}
    unexpected = sorted(_imported_roots(_EXAMPLES / name) - allowed)
    assert not unexpected, f"{name} imports {unexpected}"


def test_every_example_is_referenced_from_the_readme() -> None:
    readme = _README.read_text(encoding="utf-8")
    unreferenced = [name for name in EXAMPLE_NAMES if name not in readme]
    assert not unreferenced, f"the readme never mentions {unreferenced}"


def test_every_example_the_readme_references_exists() -> None:
    """A name left in the readme after an example was deleted is a dead link."""
    readme = _README.read_text(encoding="utf-8")
    referenced = set(re.findall(r"\b(\d\d_[a-z0-9_]+\.py)\b", readme))
    missing = sorted(referenced - set(EXAMPLE_NAMES))
    assert not missing, f"the readme names examples that do not exist: {missing}"
