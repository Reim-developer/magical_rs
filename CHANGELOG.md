# CHANGELOG
- [CHANGELOG](#changelog)
  - [Version: 0.1.3](#version-013)
  - [Version: 0.2.0](#version-020)
  - [Version: 0.2.1:](#version-021)
  - [Version: 0.3.0:](#version-030)
  - [Version: 0.3.1, `Minor edits`](#version-031-minor-edits)
  - [Version: 0.4.0 `Major API Update`](#version-040-major-api-update)
  - [Version: 0.4.5 `Major API Update`](#version-045-major-api-update)
  - [Version: 0.6.0 `Signature Tightening`](#version-060-signature-tightening)
  - [Version: 0.6.1 `Documentation and Test Coverage`](#version-061-documentation-and-test-coverage)


## Version: 0.1.3
**What has been changed:**
* Added examples of how to use `magical_rs`. 
* Specifically as follows:

| Use case           | Can be found at                        |
| ------------------ | -------------------------------------- |
| Basic usage        | [normal_usage](examples/normal_usage)  |
| Magic Custom usage | [magic_custom](examples/magic_custom/) |

* Added category `"no-std"` to [`[Cargo.toml]`](Cargo.toml)
* Changed repository URL of `magical_rs` in [`Cargo.toml`](Cargo.toml)
* Added examples to [`Cargo metadata`](Cargo.toml)

## Version: 0.2.0
**What has been changed:**
* Added methods to normalize and extends file matching in `CustomRulesMatches`
* Added documentation and test cases.
* Still retains backward compatibility for `no_std`.
* Added some macro to standardize the syntax sugar to make the API more friendly.
* Some examples of using the macros have also been added. Can be found at: [examples](examples).
* Some examples of using `DynMagic` have also been added. Can be found at: [examples](examples).
 
	**Bellow is a list of macros that have been added:**

---
  | Macro name         | Support `no_std`, backward compatibility? |
  | ------------------ | ----------------------------------------- |
  | `match_custom!`    | Yes                                       |
  | `magic_custom!`    | Yes                                       |
  | `with_fn_matches!` | Yes                                       |
  | `any_matches!`     | Yes                                       |
  | `all_matches!`     | Yes                                       |

---
* The list of supported file in `readme.md` will also synchronized.

  **Bellow is a list of the signature files have been added:**

  | Name                    | Signature                | Offset |
  | ----------------------- | ------------------------ | ------ |
  | VMDK File               | `0x4B, 0x44, 0x4D`       | `0`    |
  | Google Chrome Extension | `0x43, 0x72, 0x32, 0x34` | `0`    |

* Bellow is the development roadmap for version `0.2.0`:
  
| Name              | Description                                                             | Status |
| ----------------- | ----------------------------------------------------------------------- | ------ |
| `Macro Supported` | Allows the use of macros to sugar-syntaxize the API                     | [x]    |
| `MultipeFn`       | Support for multiple `OR`, `AND` type pointer function in `CustomMagic` | [x]    |

## Version: 0.2.1:
**What has been changed:**

* Fixed the documentation and added use for each module in [readme.md](readme.md)
* Fixed blank signatures & offsets blank in [magic_custom example](examples/magic_custom/src/v_2_0_0/magic_custom_macro.rs)

## Version: 0.3.0:
**What has been changed:**

* Added feature only avalable in version `0.3.0` of `magical_rs`: `AsyncDynMagic`
* Added documentation and usage warnings to [`lib.rs`](src/lib.rs) and [`readme.md`](readme.md)
* From this version onwards, `AsyncDynMagic` becomes an optional module. Cargo and flags are required to enable it:

```bash
cargo add magical_rs --features magical_async_dyn
```
* Of course, flag `magical_async_dyn` has also been added to [`Cargo.toml`](Cargo.toml)
* Instructions on how to use have also added at [`AsyncDynMagic Examples`](examples/async_dyn_magic)
* Current flags in version `0.3.0` can be used:

| Name                | Description                                                    | Cargo flag              |
| ------------------- | -------------------------------------------------------------- | ----------------------- |
| `magical_dyn`       | Unlock lvl 3 with file dection with infinite rules at run time | `magical_dyn`           |
| `magical_async_dyn` | Has all the features of level 3 but supports asynchronous      | `magical_async_dyn`     |
| `no_std`            | Used in non-std environments like kernel, emebedded            | `--no-default-features` |

## Version: 0.3.1, `Minor edits`
**What has been changed:**
* Minor edit in [`Cargo.toml`](Cargo.toml), added category slug `asynchronous`
* Edited some keywords related to the framework in [`Cargo.toml`](Cargo.toml)
* Changed the description of the framework to better identify it's purpose


## Version: 0.4.0 `Major API Update`
**What has been changed:**
* Added feature flag `unsafe_context` to [`Cargo.toml`](Cargo.toml)
* Release new features included in the module `magic_custom` is `WithUnsafeFn`
  - Test can be found at: [`here`](tests/unsafe.rs).
  - Documentation and instructions, security warnings have also added for `magic_custom` module.
  - This unsafe feature is only compiled and used when the `unsafe_context` flag is explicitly enabled via `Cargo`:
    ```bash
    cargo add magical_rs --features unsafe_context
    ```
  - This version also adds more documentation and warnings for features like `Default`, `WithFn`.
  - However, `no_std` support is still absolutely guaranteed.
  - Edited [`Makefile`](Makefile) rules, allowing testing with `unsafe_context` feature
  - Added example for using `unsafe_context` [`here`](examples/unsafe_context) and [`readme.md`](readme.md)
  - We do a plan to add bindings to Python. However, we can't show them yet. So, the `bindings` folder will be ignored by Git for now. [`.gitignore`](.gitignore)

## Version: 0.4.5 `Major API Update`
**What has been changed:**
* Added support for multiple unsafe function pointers. It will be disabled by default.
  - Only usable if feature flag is explicitly used by Cargo:
  - ```bash
    cargo add magical_rs --features unsafe_context
    ```
  - These new features will not affect `no_std`, and will still be supported.
  - Added testing for the above features. Can be found at [`test`](tests/unsafe.rs)
  - Samples for the above features:
  - `AllMatchesUnsafe`:
  - ```rust
    use core::slice;
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    fn is_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            slice.starts_with(b"MagicalGirl")
        }
    }

    fn is_not_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            !slice.starts_with(b"MagicalGirl")
        }
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::AllMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);

    assert_ne!(result, MagicKind::MoeMoe);
    assert_eq!(result, MagicKind::UnknownFallback);
    ```
  - `AnyMatchesUnsafe`:
  - ```rust
    use core::slice;
    use magical_rs::magical::magic_custom::match_types_custom;
    use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum MagicKind {
        MoeMoe,
        UnknownFallback,
    }

    fn is_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            slice.starts_with(b"MagicalGirl")
        }
    }

    fn is_not_shoujo_girl(data: *const ()) -> bool {
        unsafe {
            let slice_ptr = data.cast::<u8>();
            let slice = slice::from_raw_parts(slice_ptr, 100);

            !slice.starts_with(b"MagicalGirl")
        }
    }

    let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
        signatures: &[],
        offsets: &[],
        max_bytes_read: 200,
        kind: MagicKind::MoeMoe,
        rules: CustomMatchRules::AnyMatchesUnsafe(&[is_shoujo_girl, is_not_shoujo_girl]),
    }];

    let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);

    assert_eq!(result, MagicKind::MoeMoe);
    assert_ne!(result, MagicKind::UnknownFallback);
    ```


## Version: 0.6.1 `Documentation and Test Coverage`

**No API change, and no format added or removed.** The signature table is
byte-for-byte the one in `0.6.0`, and the twelve public items are the same
twelve. This release exists because `0.6.0` shipped a crate that nothing in
the repository was checking, and a readme whose claims no test agreed with.

**What this fixes:**

* The readme is now the single source of the crate's documentation. `src/lib.rs`
  carries `#![doc = include_str!("../readme.md")]` instead of 242 lines of
  duplicated doc comment, so the two can no longer disagree. Nothing but
  documentation was removed from the crate.
* The format table is checked rather than asserted. `tests/table_size.rs`,
  `tests/signature_coverage.rs` and `tests/readme_coverage.rs` fail if the
  built-in table and the readme stop matching, and the nine two-byte-only
  signatures are listed and checked individually.
* The `no_std` claim is verified instead of asserted. `make test-nostd` runs
  the suite without `std` and cross compiles to `thumbv7em-none-eabi`. It
  excludes doctests deliberately, since the readme's quick start calls a
  `std`-gated function; the cross compile is the real gate.
* `crate_dev.yml` now runs on pull requests into `master`, not only into
  `dev`. It had not run at all on the code that shipped as `0.6.0`, because
  `dev` had not moved since the commit before it.

**Not part of this release:** the Python bindings are a separate package,
`magical-py`, and are not reachable through this crate.

## Version: 0.6.0 `Signature Tightening` and `Format Table Expansion`

**Breaking: three signatures changed.**
Detection results for the same bytes can differ from `0.5.x`. Review before upgrading.

| Format | `0.5.x` | `0.6.0` | Why |
| --- | --- | --- | --- |
| `FileKind::Bzip` | `BZ` | `BZh` | `BZ` is only a prefix. The bzip2 block header is `BZh`, so the old rule reported any file starting with those two letters as bzip2. |
| `FileKind::ScriptExecute` | `#!` | `#!` plus `/` on the same line | `#!` claimed every file starting with those bytes. A real shebang must name an interpreter by path. |
| `FileKind::Ply` | `ply` | `ply` plus a line break | `ply` claimed any text file starting with that word. The PLY spec puts a line ending straight after the keyword. |

**What this fixes:**
* An AMR audio file is now reported as [`FileKind::Amr`] instead of
  `FileKind::ScriptExecute`. The AMR header `#!AMR` has no path separator, so
  it no longer matches the narrowed shebang rule.
* Bzip2 detection no longer produces false positives on files that merely
  begin with the letters `BZ`.
* A `#` comment or `#include` line in a source file is no longer reported as a
  script.

**New public module:** `magical::ext_fn::shebang`, exposing `is_shebang`.
Entries that rely on a predicate rather than a byte signature are now
`ScriptExecute` and `WEBP`.

**Known limitation of the new shebang rule:** a relative interpreter name such
as `#!python` is not detected, because it contains no path separator. Such a
script is non-portable in practice. Previously it was also undetected for a
different reason, so no realistic script is lost.

**What was changed:**
* `magical_rs` is now licensed under the MIT License instead of the GNU General
  Public License v3.0.
* The built-in format table grew from 48 to 114 formats (142 distinct magic
  signatures), defined in the new `src/magical/signatures_ext.rs`.
* The 65 new rules are **appended** to `SIGNATURE_KIND`, never interleaved.
  Because `match_types` returns the first match, appending guarantees no
  pre-existing rule can be shadowed. Behaviour of the original 48 formats is
  bit-for-bit unchanged.
* `readme.md` was rewritten and is now the single source of crate
  documentation, pulled in by `#![doc = include_str!("../readme.md")]`.
  Previously the README and the crate docs were two separate copies that had
  drifted apart; the crate docs contained a doctest referencing `async_std`,
  which is not a dependency, so `cargo test --doc` was already failing on
  `master`.
* The 65 new rules are **appended** to `SIGNATURE_KIND`, never interleaved.
  Because `match_types` returns the first match, appending guarantees no
  pre-existing rule can be shadowed, apart from the three deliberate changes
  listed above.

**Bugs found and fixed in the documentation:**
* The old format table misdescribed 9 signatures, including XML (the docs
  claimed `<!DOCTYPE` was accepted; the code only matches `<?xml ` with a
  trailing space) and the environment module format (docs said
  `MODULE\0\0\0`; the code checks `#%Module`).
* The level 5 example in the old README did not compile. It referenced
  `CustomMatchRules::WithFnUnsafe`, which does not exist; the available
  variants are `AllMatchesUnsafe` and `AnyMatchesUnsafe`.
* `with_bytes_read()` returns 36,870 bytes, not 2,048, because ISO 9660 stores
  its magic at offset 36,865. This is now documented and covered by a test.

**New tests:**
* `tests/signature_coverage.rs` walks `SIGNATURE_KIND` and asserts every entry
  detects itself, that no signature matches at an undeclared offset, that
  padding is never misdetected, and that truncated input never panics. Any
  format added in the future is covered automatically.
* `tests/signature_tightening.rs` pins the `0.6.0` behaviour change. Each of the
  three tightened formats is tested both ways: real files are still detected,
  and the specific over-match the change was made to fix no longer happens.
* `tests/readme_coverage.rs` checks the README against the code in both
  directions. Every format the table can return must be named in the README,
  every `FileKind` the README advertises must be one the table can actually
  return, and the stated table size must match `SIGNATURE_KIND.len()`. A guard
  assertion fails the test if the README table layout changes and the parser
  silently stops matching anything.
* `tests/readme_examples.rs` compiles and runs every code example in the
  README.
* `tests/table_size.rs` reports the live table size.

**Formats evaluated and deliberately excluded:**
* 3D Studio Max, whose magic is byte-for-byte identical to BigTIFF.
* Text-based formats, which have no magic bytes.

**Why:**
* The previous GPL-3.0 license capped adoption. Many organizations block GPL-licensed
  dependencies in their build and security policy, which ruled out both corporate
  adoption and the use of `magical_rs` inside permissively licensed tooling.
* MIT removes that ceiling without changing a single line of library code.

**What this does not change:**
* `magical_rs` is a single-author project. No other contributor holds copyright,
  so no third-party consent was required for this relicense.
* Versions `0.4.5` and earlier were already distributed under the GPL. Those grants
  are permanent and cannot be revoked, for anyone who already received those
  versions. Anyone depending on `0.4.5` may continue to use it under GPL terms.
  Only versions from `0.5.0` onward are MIT-licensed.
* No API, behavior, feature flag, or `no_std` support was modified by this change.
  