# CHANGELOG
- [CHANGELOG](#changelog)
  - [Version: 0.1.3](#version-013)
  - [Version: 0.2.0](#version-020)


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