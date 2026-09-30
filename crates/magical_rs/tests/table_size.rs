//! Reports the live counts so no number in the docs is guesswork.

use magical_rs::magical::match_rules::MatchRules;
use magical_rs::magical::signatures::{Magic, SIGNATURE_KIND};

#[test]
fn report_counts() {
    let entries = SIGNATURE_KIND.len();
    let signatures: usize = SIGNATURE_KIND
        .iter()
        .flat_map(|m: &Magic| m.signatures.iter())
        .count();

    let short: Vec<(String, usize)> = SIGNATURE_KIND
        .iter()
        .filter_map(|m| {
            let shortest = m.signatures.iter().map(|s| s.len()).min()?;
            Some((format!("{:?}", m.kind), shortest))
        })
        .filter(|(_, len)| *len <= 2)
        .collect();

    let custom: Vec<String> = SIGNATURE_KIND
        .iter()
        .filter(|m| matches!(m.rules, MatchRules::WithFn(_)))
        .map(|m| format!("{:?}", m.kind))
        .collect();

    println!("table entries       = {entries}");
    println!("magic signatures    = {signatures}");
    println!("custom-rule entries = {custom:?}");
    println!("entries still at 2 bytes or fewer = {}", short.len());
    for (kind, len) in &short {
        println!("  {len} bytes  {kind}");
    }
}
