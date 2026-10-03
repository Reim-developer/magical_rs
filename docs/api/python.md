# Python API reference

The `magical-py` package. For what the calls *mean* rather than what they are
called, read [Concepts](../concepts.md) first.

## Install

```bash
pip install magical-py
```

Supports CPython 3.8 and later on Windows, macOS and Linux, from one
`abi3-py38` wheel per platform. The package is typed (`py.typed`).

## The public surface

```python
from magical_py import (
    # level 1
    detect, detect_bytes, detected, Detected, read_header, bytes_read,
    DEFAULT_MAX_BYTES_READ,
    # the table itself
    describe, signature_table, read_limits, ReadLimits, Signature,
    # level 2
    MagicCustom, MatchRules, Predicate, match_types_custom, match_types_custom_all,
    # level 3
    DynMagicCustom, match_dyn_types, match_dyn_types_all,
    # level 4
    AsyncDynMagic, AsyncPredicate, match_async_dyn_types, match_async_dyn_types_all,
    # the answer type
    FileKind,
    version,
)
```

## Level 1

| Call | Returns | Notes |
| --- | --- | --- |
| `detect(source, *, max_bytes_read=None)` | `FileKind \| None` | A path, or anything with a `read` method |
| `detect_bytes(data, *, max_bytes_read=None)` | `FileKind \| None` | For bytes you already have |
| `detected(source, *, max_bytes_read=None)` | `Detected` | The file first, for a scan. Reads lazily |
| `read_header(source, *, max_bytes=None)` | `bytes` | The leading bytes, detecting nothing |
| `bytes_read()` | `int` | 36,870 — the window that reaches every format |
| `DEFAULT_MAX_BYTES_READ` | `int` | 2,048 — what one rule usually declares. **Not the same number** |

```python
from magical_py import FileKind, detect, detect_bytes

assert detect("photo.jpg") is FileKind.Jpg
assert detect_bytes(b"GIF89a") is FileKind.GIF
assert detect_bytes(b"." * 64) is None
```

`detect` takes a path **or anything open for binary reading**. A path goes to the
extension, so the `OSError` subclasses come from where the rest of them do; a
stream is read here, since it has no path to hand over. That means a socket, a pipe
and a `zipfile.ZipExtFile` all work.

**A `None` from `detect_bytes` means one of two things**, and naming the window is
what tells them apart: no rule matched, or no rule *whose read size fits the window
you named* matched. See [Concepts](../concepts.md#none-means-one-thing-except-when-it-does-not).

### `Detected` — the data-first spelling

`detect` puts the function first because a question asked once should read that
way. A scan asks about every file, and mostly the answer is no, so `detected` puts
the file first.

```python
from magical_py import FileKind, detected

what = detected("photo.jpg")

what.kind                       # FileKind.Jpg | None, as detect()
bool(what)                      # True when something matched
what.mime                       # 'image/jpeg', or None
what.extension                  # 'jpg', or None
what.description                # 'JPEG', or 'no match'
what.rule.signatures            # the bytes the winning rule compares
what.window                     # the window in force, 36_870 by default

what.matches(FileKind.Jpg)      # would Jpg's own rule match these bytes?
what.matches_any([Jpg, Png])    # would any of these?  varargs also work
what.within(2048).kind          # re-classify the same bytes in a smaller window
```

| Method | Returns | Notes |
| --- | --- | --- |
| `kind` | `FileKind \| None` | Cached. Asking twice reads nothing twice |
| `matches(kind)` | `bool` | One named format, ignoring table order |
| `matches_any(*kinds)` | `bool` | Varargs or one iterable. Empty is `False` |
| `within(max_bytes_read)` | `Detected` | Returns `self`, so it chains |
| `window` | `int` | The window actually in force |
| `rule` | `Signature \| None` | `None` when nothing matched |
| `mime` / `extension` | `str \| None` | `None` when nothing matched |
| `description` | `str` | `"no match"` when nothing matched |
| `bool(detected)` | `bool` | `True` exactly when something matched |

Three things worth knowing:

**Nothing is read by the constructor.** The read happens on the first method that
needs an answer, so building one is free and a caller who only passes it on never
touches the disk.

**The bytes are read once**, however many methods are called. A path is opened
once; a stream is read once, which matters because a socket or a pipe has its bytes
only once.

**It is called `matches`, not `is`, because `is` is a keyword in Python.**
`what.is(FileKind.Jpg)` is a `SyntaxError`. The name is not a loss: `matches` is the
inverse of the `FileKind.matches(data)` below, so the two read as a pair.

`within` re-classifies the bytes already in hand and does not read again, because
it does not have to — the bytes are a superset of a narrower window. Widening the
window past the original read cannot recover bytes that were never read, which is
what `window` reports.

## `FileKind`

A real `enum` of 114 members, not a string to parse. `FileKind.Png.value` is
`'png'`, a stable id for serialising.

| Member | Returns | Notes |
| --- | --- | --- |
| `FileKind.description` | `str` | e.g. `"PNG"`. Also each member's `__doc__` |
| `FileKind.mime` | `str \| None` | `None` means "none registered", never "unknown" |
| `FileKind.extension` | `str \| None` | `None` means "none registered" |
| `FileKind.rule` | `Signature` | What this format is matched on. Raises if the table lacks it |
| `FileKind.matches(data)` | `bool` | Would this format's own rule match `data` |
| `FileKind.from_value(value)` | `FileKind` | Look a member up by its serialised value |

Python 3.9 and later render each member's `__doc__` in `help()`; 3.8 does not, so
on 3.8 read `FileKind.Png.__doc__` directly.

## Asking the table a question

| Call | Returns | Notes |
| --- | --- | --- |
| `describe(kind)` | `Signature \| None` | One format's entry |
| `signature_table()` | `tuple[Signature, ...]` | All 114, in detection order. Walking it and stopping at the first match reproduces `detect` |
| `read_limits()` | `ReadLimits` | The smallest window that finds each format |
| `Signature.signatures` | `tuple[bytes, ...]` | The bytes compared |
| `Signature.offsets` | `tuple[int, ...]` | Where they are compared |
| `Signature.max_bytes_read` | `int` | What this entry declares |
| `Signature.max_offset` | `int` | The furthest byte position touched |

A `Signature` is frozen and built fresh per call, so nothing a caller does to one
can reach the next.

`signature_table()` carries the order through rather than sorting, because the order
is the answer to "why did my file come back as this": walking the tuple and
stopping at the first match reproduces `detect` exactly.

## Levels 2, 3 and 4

All three follow the same rule: **order is the answer**, and the first rule that
matches wins.

```python
import enum
from magical_py import MagicCustom, match_types_custom, match_types_custom_all

class Kind(enum.Enum):
    """Yours. `kind` is carried through untouched, so this is your type."""
    Shoujo = "shoujo"
    Unknown = "unknown"

RULES = [
    MagicCustom(kind=Kind.Shoujo, signatures=[b"MagicalGirl"], offsets=[0]),
    MagicCustom(kind=Kind.Unknown, signatures=[b"Mahou"], offsets=[0]),
]

assert match_types_custom(b"MagicalGirl", RULES, Kind.Unknown) is Kind.Shoujo
assert match_types_custom(b"nothing", RULES, Kind.Unknown) is Kind.Unknown

# Every match, not just the first.
assert match_types_custom_all(b"MagicalGirl", RULES) == [Kind.Shoujo]
```

`MagicCustom` takes `kind`, `signatures`, `offsets`, `max_bytes_read` and
`rules`. **`kind` is any object at all** — the binding carries it through untouched,
so the type above is yours to define and could equally be a `str`, a dataclass or a
`NamedTuple`. `fallback` must be the same type as every rule's kind, and the answer
is never `None`: a level 2 rule set is homogeneous, and deciding what "none of
these" means is the caller's job, which is why the argument is required.

**`offsets` is a list of positions, not a pairing with `signatures`.** Any
signature matching at any of the offsets is a match — the crate's arm is
`signatures.any(offsets.any(...))`. So two signatures at one offset means "either
prefix, at the start", and it is *not* the same as one signature at two offsets. A
rule that needs the two signatures separately placed wants one rule each.

`MagicCustom` validates four things and no more: `max_bytes_read` and every offset
must be non-negative; `signatures` and `offsets` must both be given or both be
empty; and `rules`, when set, must be given on its own — the crate's fields are
`'static` and it discards them silently, so the binding refuses instead. A
**length mismatch between the two lists is not checked**, because there is nothing
to check once offsets are a set of positions rather than a pairing.

| Level | Type | Entry points |
| --- | --- | --- |
| 2 | `MagicCustom`, `MatchRules`, `Predicate` | `match_types_custom`, `match_types_custom_all` |
| 3 | `DynMagicCustom` | `match_dyn_types`, `match_dyn_types_all` |
| 4 | `AsyncDynMagic`, `AsyncPredicate` | `match_async_dyn_types`, `match_async_dyn_types_all` |

[Detection levels](../detection-levels.md) says what each costs.

## Errors

| Raised | When |
| --- | --- |
| `ValueError` | A negative `max_bytes_read` or `max_bytes` |
| `FileNotFoundError` | The path does not exist |
| `PermissionError` | The path cannot be read. A directory lands here on Windows |
| `IsADirectoryError` | A directory, on platforms that raise it instead |

The negative-size message is worded to match the level 2 and 3 constructors, so a
caller moving between levels learns one message rather than three.

## Notes

**`detect` is deliberately blocking.** It reads a header and compares bytes, and a
file read is a file read — making it `async` would mean either an executor this
package does not have or a thread it should not start. For many files, read them
yourself and use `detect_bytes`. Level 4 is the async one, and it takes
*predicates*, not paths: the coroutine is yours, so a caller reads the file wherever
it likes.

**`version()` returns the installed package's version**, so a caller can branch on
it. It is the extension module's, which is the same string.