# CHANGELOG
- [CHANGELOG](#changelog)
  - [Version: 0.1.3](#version-013)
  - [Version: 0.2.0](#version-020)
  - [Version: 0.2.1:](#version-021)
  - [Version: 0.3.0:](#version-030)


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