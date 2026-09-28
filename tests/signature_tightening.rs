//! Regression tests for the `0.6.0` signature tightening.
//!
//! Three formats previously matched on a signature that was either wrong or
//! far too broad. Each test below pins both the new behaviour and the exact
//! case the change was made to fix, so the old bug cannot return unnoticed.

use magical_rs::magical::ext_fn::shebang::is_shebang;
use magical_rs::magical::magic::FileKind;

fn detect(bytes: &[u8]) -> Option<FileKind> {
    FileKind::match_types(bytes)
}

// ---------------------------------------------------------------------------
// Bzip: `BZ` (2 bytes) was wrong. The bzip2 block header is `BZh`.
// ---------------------------------------------------------------------------

#[test]
fn bzip2_matches_the_real_block_header() {
    for header in [
        &b"BZh9"[..],
        &b"BZh0"[..],
        &b"BZh1"[..],
        &b"BZh"[..],
    ] {
        assert_eq!(
            detect(header),
            Some(FileKind::Bzip),
            "{header:?} is a valid bzip2 block header"
        );
    }
}

#[test]
fn bzip2_no_longer_claims_a_bare_bz_prefix() {
    // The old two-byte rule matched anything starting with `BZ`, which
    // includes files that are not bzip2 at all.
    for not_bzip2 in [
        &b"BZ"[..],
        &b"BZfile"[..],
        &b"BZ\x00\x00\x00"[..],
        &b"Business"[..],
    ] {
        assert_ne!(
            detect(not_bzip2),
            Some(FileKind::Bzip),
            "{not_bzip2:?} is not a bzip2 block header"
        );
    }
}

// ---------------------------------------------------------------------------
// ScriptExecute: `#!` (2 bytes) claimed the AMR audio header.
// ---------------------------------------------------------------------------

#[test]
fn shebang_requires_an_interpreter_path() {
    for script in [
        &b"#!/bin/sh\n"[..],
        &b"#!/usr/bin/env python3\n"[..],
        &b"#!/usr/bin/perl -w\n"[..],
        &b"#!/bin/bash -e\n"[..],
    ] {
        assert_eq!(
            detect(script),
            Some(FileKind::ScriptExecute),
            "{script:?} is a real shebang"
        );
    }
}

#[test]
fn shebang_no_longer_claims_bare_hash_bang() {
    for not_a_script in [
        &b"#!"[..],
        &b"#!\n"[..],
        &b"#! \n"[..],
        &b"#!x"[..],
    ] {
        assert_ne!(
            detect(not_a_script),
            Some(FileKind::ScriptExecute),
            "{not_a_script:?} names no interpreter path"
        );
    }
}

/// The specific collision that motivated the change.
#[test]
fn amr_audio_is_no_longer_mistaken_for_a_script() {
    let amr = b"#!AMR\n\x00\x00\x00\x00";
    assert_eq!(
        detect(amr),
        Some(FileKind::Amr),
        "AMR audio must not be reported as a script"
    );

    let amr_wide = b"#!AMR-WP\n";
    assert_eq!(detect(amr_wide), Some(FileKind::Amr));
}

#[test]
fn shebang_predicate_rejects_amr_directly() {
    assert!(!is_shebang(b"#!AMR\n"));
    assert!(!is_shebang(b"#!AMR-WP\n"));
    assert!(is_shebang(b"#!/bin/sh\n"));
}

#[test]
fn hash_comment_is_not_a_shebang() {
    assert!(!is_shebang(b"# comment\n"));
    assert!(!is_shebang(b"#include <stdio.h>\n"));
    assert_eq!(detect(b"#include <stdio.h>\n"), None);
}

// ---------------------------------------------------------------------------
// PLY: bare `ply` claimed any text file starting with that word.
// ---------------------------------------------------------------------------

#[test]
fn ply_requires_the_header_line_break() {
    for header in [
        &b"ply\nformat ascii 1.0\n"[..],
        &b"ply\n"[..],
        &b"ply\r\n"[..],
    ] {
        assert_eq!(
            detect(header),
            Some(FileKind::Ply),
            "{header:?} is a valid PLY header"
        );
    }
}

#[test]
fn ply_no_longer_claims_a_bare_word() {
    for not_ply in [
        &b"ply"[..],
        &b"plywood"[..],
        &b"plygonal math"[..],
    ] {
        assert_ne!(
            detect(not_ply),
            Some(FileKind::Ply),
            "{not_ply:?} is not a PLY file"
        );
    }
}

// ---------------------------------------------------------------------------
// The tightened rules must not have broken the formats that share a prefix.
// ---------------------------------------------------------------------------

#[test]
fn tightening_did_not_break_neighbouring_formats() {
    // Deb and other ar-based formats still resolve.
    assert_eq!(detect(b"!<arch>\ndebian"), Some(FileKind::Deb));
    // GIF, PNG and other exact-signature formats are unaffected.
    assert_eq!(detect(b"GIF89a"), Some(FileKind::GIF));
    assert_eq!(
        detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        Some(FileKind::Png)
    );
}
