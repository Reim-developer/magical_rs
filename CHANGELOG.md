# CHANGELOG
- [CHANGELOG](#changelog)
  - [Version: 0.1.3](#version-013)
  - [Version: 0.2.0](#version-020)
  - [Version: 0.2.1:](#version-021)
  - [Version: 0.3.0:](#version-030)
  - [Version: 0.3.1, `Minor edits`](#version-031-minor-edits)
  - [Version: 0.4.0 `Major API Update`](#version-040-major-api-update)
  - [Version: 0.4.5 `Major API Update`](#version-045-major-api-update)


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
  