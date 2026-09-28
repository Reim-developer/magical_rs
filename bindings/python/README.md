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

One `abi3` wheel covers CPython 3.8 and later, so there is no need to build from
source on any platform we publish for.

`FileKind` covers all 114 formats the Rust crate detects, and the enum is
declared in Python rather than generated, so your editor completes
`FileKind.` and `help()` reads like documentation rather than a debug dump.

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

Each member carries its display name, which `help(magical_py.FileKind)` shows
alongside all 113 others. The same text is available programmatically:

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
