"""The hand-written type stub must match the compiled extension.

maturin only emits a ``.pyi`` when it packages a wheel, so the stub in
``python/magical_py/_magical_rs.pyi`` is maintained by hand. Without a check
it would drift, and pyright would then report errors that do not reflect the
extension, or worse, stay silent about a function that no longer exists.
"""

from __future__ import annotations

import ast
import pathlib

from magical_py import _magical_rs

_STUB = pathlib.Path(__file__).resolve().parents[1] / "python" / "magical_py" / "_magical_rs.pyi"


def _stub_public_names() -> set[str]:
    """Return the names the stub declares at module level."""
    tree = ast.parse(_STUB.read_text(encoding="utf-8"))
    names: set[str] = set()
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            names.add(node.name)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            names.add(node.target.id)
        elif isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name):
                    names.add(target.id)
    return names


def _module_public_names() -> set[str]:
    """Return the names the extension actually exports.

    The interpreter attaches these to every module, and PyO3 adds ``__all__``,
    so none of them belong in a type stub. ``__version__`` is *not* excluded:
    the extension sets it deliberately and the stub declares it.
    """
    not_part_of_the_stub = {
        "__name__",
        "__doc__",
        "__package__",
        "__loader__",
        "__spec__",
        "__file__",
        "__cached__",
        "__builtins__",
        "__all__",
    }
    return set(dir(_magical_rs)) - not_part_of_the_stub


def test_stub_declares_everything_the_module_exports() -> None:
    missing = sorted(_module_public_names() - _stub_public_names())
    assert not missing, f"the stub does not declare: {missing}"


def test_stub_declares_nothing_the_module_lacks() -> None:
    extra = sorted(_stub_public_names() - _module_public_names())
    assert not extra, f"the stub declares functions the module does not have: {extra}"


def test_stub_is_valid_python() -> None:
    """A syntax error in the stub would make pyright fall back silently."""
    assert ast.parse(_STUB.read_text(encoding="utf-8")) is not None
