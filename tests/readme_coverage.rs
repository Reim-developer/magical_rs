//! Every format the crate can actually detect must be named in the readme.
//!
//! A format that is detected but undocumented is invisible to anyone deciding
//! whether to use the crate, and the readme tables are the main reason anyone
//! looks. This derives the list of detectable kinds from `SIGNATURE_KIND`
//! itself, so a format added to the table without a readme row fails the build
//! rather than going unnoticed.

use magical_rs::magical::signatures::SIGNATURE_KIND;

const fn readme() -> &'static str {
    include_str!("../readme.md")
}

/// Distinct `FileKind` names the detection table can actually return.
fn detectable_kinds() -> Vec<String> {
    let mut names: Vec<String> = SIGNATURE_KIND
        .iter()
        .map(|entry| format!("{:?}", entry.kind))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Variant names parsed out of the readme's format tables, sorted and
/// deduplicated.
///
/// This is the only place that knows what a format table row looks like, and
/// every check below reads it. The parser used to be copied into each test,
/// which was a live hazard rather than a stylistic one: the copies had already
/// drifted, one of them lacking a condition the others had, and a parser that
/// silently stops recognising rows makes the tests that use it pass for the
/// wrong reason.
///
/// A row is `| Name | `Variant` | magic | offset |`. Splitting on `|` yields
/// leading and trailing empty cells, so four columns give six cells with the
/// offset at index 4. Requiring that cell to hold one or more comma-separated
/// integers is what separates these rows from the other tables and from prose,
/// so cell 2 is the `FileKind` column.
///
/// The offset may list several values, as ISO 9660 does (32769, 34817, 36865),
/// and a variant name may begin with an underscore, as `_8BPS` does. Accepting
/// only a bare integer, or only letter-initial names, skips those two rows
/// without any sign that it happened.
fn readme_table_variants() -> Vec<String> {
    let mut variants: Vec<String> = readme()
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            if cells.len() != 6 {
                return None;
            }
            if cells[1].is_empty() || cells[2].is_empty() {
                return None;
            }
            let offsets_are_numeric = !cells[4].is_empty()
                && cells[4]
                    .split(',')
                    .map(str::trim)
                    .all(|o| !o.is_empty() && o.parse::<usize>().is_ok());
            if !offsets_are_numeric {
                return None;
            }
            let variant = cells[2].strip_prefix('`')?.strip_suffix('`')?;
            if variant.is_empty()
                || !variant
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                || !variant.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            {
                return None;
            }
            Some(variant.to_owned())
        })
        .collect();
    variants.sort();
    variants.dedup();
    variants
}

/// The readme must state the live table size, otherwise the "114 formats"
/// claim drifts the moment a format is added.
#[test]
fn readme_states_the_live_table_size() {
    let expected = format!("{} formats", SIGNATURE_KIND.len());

    assert!(
        readme().contains(&expected),
        "readme.md does not mention \"{expected}\"; the stated table size is stale"
    );
}

/// Every detectable `FileKind` needs a row in one of the readme tables.
#[test]
fn every_detectable_kind_is_named_in_the_readme() {
    let doc = readme();
    let names = detectable_kinds();

    let missing: Vec<&String> = names.iter().filter(|n| !doc.contains(n.as_str())).collect();

    assert!(
        missing.is_empty(),
        "{} of {} detectable formats are missing from readme.md: {missing:?}",
        missing.len(),
        names.len()
    );
}

/// Every `FileKind` the readme advertises must be one the table can return.
///
/// This catches the opposite mistake: a row claiming a format that does not
/// exist, which would send users looking for a result the crate never produces.
#[test]
fn readme_does_not_advertise_formats_that_do_not_exist() {
    let real = detectable_kinds();
    let advertised = readme_table_variants();

    // Guard against the row shape changing, which would make this test pass
    // for the wrong reason.
    assert!(
        advertised.len() >= 110,
        "only parsed {} format table rows; the readme table layout must have changed \
         and this test is no longer checking anything",
        advertised.len()
    );

    let bogus: Vec<&String> = advertised.iter().filter(|a| !real.contains(a)).collect();

    assert!(
        bogus.is_empty(),
        "readme.md advertises these FileKind variants but the detection table cannot \
         return them: {bogus:?}"
    );
}

/// The parsed rows must account for every detectable kind exactly.
///
/// The two tests above use a substring check and a one-way filter respectively,
/// and both can pass while silently skipping rows the parser does not
/// recognise. Comparing the parsed set against the real set closes that gap: a
/// row the parser misses shows up as a kind absent from the table rather than
/// as a quietly smaller count.
#[test]
fn every_detectable_kind_has_its_own_table_row() {
    let real = detectable_kinds();
    let advertised = readme_table_variants();

    let absent: Vec<&String> = real.iter().filter(|k| !advertised.contains(k)).collect();

    assert!(
        absent.is_empty(),
        "these kinds are detectable but have no parseable row in readme.md: {absent:?}"
    );
    assert_eq!(
        advertised, real,
        "the set of variants in the readme tables differs from the detection table"
    );
}

