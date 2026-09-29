# `magical_py`

Native Python bindings for
[`magical_rs`](https://github.com/Reim-developer/magical_rs), a zero-dependency
Rust library that identifies files by their magic bytes.

This is **not** a drop-in replacement for `python-magic`, and it is not trying to
be one. `python-magic` returns a string like `"image/png"` and leaves you to parse
it. This returns a typed value that carries its media type and conventional
extension with it.

## Install

```bash
pip install magical-py
```

There is a binary wheel for every platform we publish for. One `abi3` wheel
covers CPython 3.8 and later, so you will not have to build from source.

`FileKind` covers all 114 formats the Rust crate detects, and the enum is
declared in Python rather than generated, so it is a real `enum.Enum` and your
editor completes `FileKind.` with every member and its docstring.

## Use

```python
from magical_py import FileKind, detect

kind = detect("photo.jpg")

kind is FileKind.Jpg      # True, a real enum member your editor can complete
kind.value                 # 'jpg', a stable id for serialising
kind.mime                  # 'image/jpeg'
kind.extension             # 'jpg'
kind.description           # 'JPEG'
```

For data you already have in memory:

```python
from magical_py import detect_bytes

detect_bytes(header)       # FileKind.Png | None
```

Both return `None` when nothing matches, and raise the usual `OSError`
subclasses when a file cannot be read:

```python
try:
    kind = detect(path)
except FileNotFoundError:
    kind = None
```

## What it looks like

Each member carries its display name as its `__doc__`, which is the attribute
the REPL and `help()` read. Python 3.9 and later render it for all 114 members;
Python 3.8 omits it from `help()`, so on 3.8 read `__doc__` directly as below.

```python
>>> FileKind.Png.__doc__
'PNG'
>>> FileKind.ISO.__doc__
'ISO 9660'
>>> FileKind.PkgZip.description
'Zip / JAR / APK'
```

## Things worth knowing

**Detection reads magic bytes only.** The file name is never consulted, so a
`.jpg` that actually contains PNG data is reported as `FileKind.Png`. That is
the behaviour you want from a detection library, and it is also why the
extension is advisory rather than something to switch on.

**Some media types are `None`.** `FileKind.mime` and `FileKind.extension`
return `None` when no media type is registered or verified for that format.
`None` means "there isn't one", never "we guessed". A wrong media type would be
a documentation bug of the same kind as a wrong magic byte, and this project
does not ship those.

**`detect` reads 36,870 bytes.** ISO 9660 stores its magic at offset 36,865, so
detecting it means reading that far into the file. `detect` does that for you;
`magical_py.bytes_read()` reports the figure if you are reading headers
yourself. Do not hardcode 2048 and assume you are done.

**Some signatures are only two bytes.** 9 formats match on nothing but a
two-byte prefix: `Arj`, `Bitmap`, `Gzip`, `MP3`, `MSDOS`, `Pcx`, `Pickle`,
`SerializedJavaData` and `Zlib`. Those are the values the formats themselves
specify, so a file beginning with those bytes is reported as that format even
if it is not really one. Each one is listed with its bytes in the
[`magical_rs` README](https://github.com/Reim-developer/magical_rs#two-things-to-know-before-you-rely-on-it).

**Two rules look at more than a fixed pattern.** `FileKind.ScriptExecute`
requires a `/` later on the first line, so a bare `#!` does not count;
`FileKind.WEBP` requires the full `RIFF....WEBP` layout. The shebang rule
exists because `#!AMR` is the literal magic of AMR audio: without the `/`
check, the script rule claimed those files and they became undetectable.

## Detection levels

The Rust crate has five detection levels. Level 1 is `detect` and
`detect_bytes`, the table of 114 formats above. The other four are custom rules,
and three of them have a Python counterpart. They differ in what a match costs,
which is the whole reason they are three things.

| Level | Rule | A match costs | Kinds | Nothing matches |
| --- | --- | --- | --- | --- |
| 2 | `MagicCustom` | one Rust call | one type across the set | your `fallback` |
| 3 | `DynMagicCustom` | a trip into Python | any, mixed freely | `None` |
| 4 | `AsyncDynMagic` | an awaited call on your loop | any, mixed freely | `None` |

Level 5 is raw pointers. It is `unsafe` by definition and is not exposed.

### Level 2: say what the format looks like

`match_types_custom` returns the first rule that matched, or the `fallback` you
pass as the last argument. `match_types_custom_all` returns every kind that
matched, in rule order.

```python
from magical_py import MagicCustom, match_types_custom

PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])

rules = [
    MagicCustom("gif", [b"GIF87a", b"GIF89a"], [0]),
    MagicCustom("png", [PNG], [0]),
]

match_types_custom(PNG, rules, "unknown")     # 'png'
match_types_custom(b"zzz", rules, "unknown")  # 'unknown', the fallback
```

A signature matches at any of the offsets, so one rule can cover every variant
of a format. Order is priority order: the first match wins and no later rule is
tested, which is what makes putting a cheap signature rule first worthwhile.

When the bytes alone are not enough, add predicates:

```python
from magical_py import MagicCustom, MatchRules

def is_utf8(data: bytes) -> bool:
    try:
        data.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return True

rule = MagicCustom("text", rules=MatchRules.all(is_utf8))
rule.matches(header)  # True or False
```

`MatchRules.all` needs every predicate, `MatchRules.any` needs one, and
`MatchRules.with_fn` takes a single one. All three stop as soon as they can.
Signatures and predicates are alternatives, not a conjunction: a rule with
`rules` set will not also compare signatures, and asking for both raises
`ValueError` rather than quietly ignoring one of them.

### Level 3: decide in Python

Reach for level 3 when the format is not known when you write the code: a rule
from a config file, chosen by a plugin, assembled in a loop. `match_dyn_types`
returns the first kind that matched, or `None`; `match_dyn_types_all` returns
every one, in rule order.

```python
from magical_py import DynMagicCustom, match_dyn_types, match_dyn_types_all

rules = [
    DynMagicCustom(lambda data: data.startswith(b"RIFF"), "riff"),
    DynMagicCustom(lambda data: data[4:8] == b"WEBP", "webp"),
]

match_dyn_types(header, rules)      # a kind, or None
match_dyn_types_all(header, rules)  # every kind that matched, in rule order
```

Kinds do not have to agree, so one list can mix strings, enums, dataclasses and
`None`. Nothing is converted on the way through; each rule hands back the exact
object it was given.

### Level 4: decide with something that takes time

`match_async_dyn_types` and `match_async_dyn_types_all` are the level 3 pair,
awaited.

```python
import asyncio

from magical_py import AsyncDynMagic, match_async_dyn_types

async def registered_checksum(data: bytes) -> bool:
    async with session.get(registry_url) as response:
        return await response.text() == data[:32].hex()

rule = AsyncDynMagic(registered_checksum, "verified")

kind = asyncio.run(match_async_dyn_types(header, [rule]))
```

The matcher is awaited on the loop you are already on. That is the difference
from the crate, which drives its own future from Rust, and it is why level 4
needs no feature flag and no extra dependency, unlike
`cargo add magical_rs --features magical_async_dyn`. A matcher that is handed a
coroutine object instead of a function is a `TypeError`, because a rule gets
matched against many buffers and a coroutine can only be awaited once.

### Choosing one

Level 2 for a format you know. It is one call with no Python in it, and the
difference between that and a Python callable is exactly the difference between
the two rows above.

Level 3 for a format you do not know yet, or a rule that is data. Level 4 only
when the decision genuinely has to wait for something, since awaiting is worth
it exactly when there is something to await.

## Examples

Six runnable scripts live in
[`bindings/python/examples/`](https://github.com/Reim-developer/magical_rs/tree/master/bindings/python/examples).
Each one builds the files it reads from the magic bytes in the format tables
above and writes them to a temporary directory, so nothing has to be checked in
and every example runs from a bare install:

```bash
python 01_detect_a_file.py
```

| Example | Shows |
| --- | --- |
| `01_detect_a_file.py` | `detect()` on a path, and the metadata a `FileKind` carries |
| `02_detect_bytes.py` | `detect_bytes()`, `bytes_read()`, and reading a header rather than a file |
| `03_name_does_not_matter.py` | Why the file name is never consulted |
| `04_list_supported_formats.py` | Enumerating `FileKind`, and which media types are `None` |
| `05_scan_a_directory.py` | Walking a tree and tallying it by kind |
| `06_handle_errors.py` | A kind, `None` and an exception as three different outcomes |

`tests/test_examples.py` runs every one of them and checks the values they
print, so an example that stops working fails the suite instead of quietly
misleading someone.

## Type checking

The package ships a `py.typed` marker and stubs for the compiled module, and it
is checked with `pyright` in strict mode. `pythonVersion` is pinned to 3.8 so the
checker enforces the same floor as the wheel.

```python
from magical_py import FileKind

# Narrowed to FileKind after the None check.
kind: FileKind | None = detect("photo.jpg")
if kind is not None:
    reveal_type(kind)     # FileKind
    reveal_type(kind.mime)  # str | None
```

## Development

Managed with [uv](https://docs.astral.sh/uv/).

```bash
uv sync                # create the venv and install maturin, pytest, pyright
uv run maturin develop # build the extension into the venv
uv run pytest          # run the tests
uv run pyright         # type check in strict mode
uv build               # build the wheel and the sdist
```

`python/magical_py/_kinds.py` is generated. Edit the metadata table in
`gen_kinds.ps1` and re-run it; `tests/test_drift.py` fails if the enum and the
Rust detection table disagree.

## License

MIT, the same as `magical_rs`.
