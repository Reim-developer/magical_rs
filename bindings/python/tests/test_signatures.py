"""Tests for the level 1 introspection surface.

:func:`magical_py.detect` answers "what is this file" and stops at the first
table entry that matched. These test the three questions that leaves open: what
an entry compares, whether one format would match regardless of order, and what
a read window costs.

Everything here is checked against the crate's own table rather than a list
written out by hand, so a signature changed on the Rust side fails these tests
instead of leaving a stale figure in a docstring.
"""

from __future__ import annotations

import dataclasses

import pytest

from magical_py import (
    DEFAULT_MAX_BYTES_READ,
    FileKind,
    Signature,
    bytes_read,
    describe,
    detect_bytes,
    read_limits,
    signature_table,
)

PNG = bytes([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
ZIP = bytes([0x50, 0x4B, 0x03, 0x04])

# The two entries the crate decides with a function rather than a fixed byte
# pattern. Both are listed in readme.md and in tests/signature_coverage.rs.
PREDICATE_KINDS = {FileKind.ScriptExecute, FileKind.WEBP}


# --- read_limits -----------------------------------------------------------


def test_read_limits_reports_the_crate_constants() -> None:
    limits = read_limits()
    assert limits.default_max_bytes_read == 2048
    assert limits.default_offset == 0
    assert limits.iso_offsets == (32769, 34817, 36865)
    assert limits.tar_offsets == (257,)


def test_iso_read_size_is_derived_from_its_furthest_offset() -> None:
    """The figure the crate documents as ``~32774`` is really 36,870.

    ``max_bytes`` takes the *largest* offset, and ``ISO_OFFSETS`` ends at 36,865,
    not at the 32,769 it starts with. The comment in ``magic.rs`` had the first
    offset's arithmetic and was out by exactly 4,096, which is one sector.
    """
    limits = read_limits()
    assert limits.iso_max_bytes_read == 36870
    assert limits.iso_max_bytes_read == max(limits.iso_offsets) + len("CD001")


def test_iso_is_what_makes_the_total_read_long() -> None:
    """``bytes_read()`` is the ISO entry's figure, and nothing exceeds it."""
    limits = read_limits()
    assert bytes_read() == limits.iso_max_bytes_read


def test_tar_read_size_is_derived_from_its_offset() -> None:
    limits = read_limits()
    assert limits.tar_max_bytes_read == 262
    assert limits.tar_max_bytes_read == max(limits.tar_offsets) + len(b"ustar")


def test_default_max_bytes_read_is_the_floor_for_almost_everything() -> None:
    """What a window below 2,048 actually costs, measured rather than asserted.

    2,048 is what the bulk of the table declares, so a smaller window drops
    nearly all of it. ``MP3`` is the exception: it declares 262, because a
    frame header ends well before the second sector and the table does not
    claim a read it does not need. ISO and Tar declare their own, larger,
    figures for the offsets they live at.
    """
    assert DEFAULT_MAX_BYTES_READ == 2048
    declared = {e.kind: e.max_bytes_read for e in signature_table()}

    below = {k.name: v for k, v in declared.items() if v < DEFAULT_MAX_BYTES_READ}
    assert below == {"MP3": 262}

    # The one format a 2,048 window still cannot reach, because its magic is
    # past that point.
    assert declared[FileKind.Png] == DEFAULT_MAX_BYTES_READ
    assert declared[FileKind.ISO] > DEFAULT_MAX_BYTES_READ


def test_read_limits_is_frozen_and_independent() -> None:
    """Nothing a caller can do to a returned object reaches the next one.

    A dict returned from Rust would have been mutable, and one caller's edit
    would have been visible in the next caller's copy had it been cached. A
    frozen dataclass built per call gives the same guarantee with a type on it.
    """
    limits = read_limits()
    with pytest.raises(dataclasses.FrozenInstanceError):
        limits.default_max_bytes_read = 0  # type: ignore[misc]
    assert read_limits().default_max_bytes_read == 2048


# --- describe and FileKind.rule -------------------------------------------


def test_describe_returns_the_entry_for_a_format() -> None:
    entry = describe(FileKind.Png)
    assert entry is not None
    assert entry.kind is FileKind.Png
    assert entry.signatures == (PNG,)
    assert entry.offsets == (0,)
    assert entry.max_bytes_read == DEFAULT_MAX_BYTES_READ
    assert entry.uses_predicate is False


def test_file_kind_rule_is_describe() -> None:
    assert FileKind.Png.rule == describe(FileKind.Png)


def test_describe_reports_the_furthest_iso_offset() -> None:
    entry = FileKind.ISO.rule
    assert entry.offsets == (32769, 34817, 36865)
    assert entry.max_offset == 36865
    assert entry.max_bytes_read == 36870


def test_describe_reports_tar_at_its_real_offset() -> None:
    entry = FileKind.Tar.rule
    assert entry.offsets == (257,)
    assert entry.max_offset == 257


def test_a_predicate_entry_reports_no_signatures() -> None:
    """ScriptExecute and WEBP are decided by a function, not by bytes.

    Reporting the two bytes a shebang starts with as if they were the rule
    would be a lie: the crate requires a ``/`` later on the line and ignores
    the signature entirely, which is why ``#!AMR`` is detectable at all.
    """
    for kind in PREDICATE_KINDS:
        entry = describe(kind)
        assert entry is not None
        assert entry.uses_predicate is True
        assert entry.signatures == ()


def test_every_kind_has_exactly_one_entry() -> None:
    for kind in FileKind:
        assert describe(kind) is not None, f"{kind.name} has no table entry"


def test_describe_agrees_with_the_table_listing() -> None:
    by_name = {entry.kind: entry for entry in signature_table()}
    for kind in FileKind:
        assert describe(kind) == by_name[kind]


# --- signature_table -------------------------------------------------------


def test_the_table_covers_every_kind() -> None:
    assert {entry.kind for entry in signature_table()} == set(FileKind)


def test_the_table_lists_predicate_entries_exactly() -> None:
    found = {e.kind for e in signature_table() if e.uses_predicate}
    assert found == PREDICATE_KINDS


def test_every_entry_declares_a_read_size_and_something_to_match_on() -> None:
    """A signature entry needs bytes at an offset; a predicate entry needs
    neither, because it decides on the buffer as a whole.

    ``WEBP`` is the one entry with no offsets at all, and asserting they are
    all non-empty would be asserting a property of the table that the crate
    does not promise.
    """
    for entry in signature_table():
        assert entry.max_bytes_read > 0, f"{entry.kind.name} declares no read size"
        if entry.uses_predicate:
            assert entry.signatures == ()
        else:
            assert entry.signatures, f"{entry.kind.name} has no signatures"
            assert entry.offsets, f"{entry.kind.name} has no offsets"


def test_the_table_order_is_what_detection_tries() -> None:
    """Order is the answer to "why did my file come back as this".

    Walking the table and stopping at the first match has to reproduce
    ``detect_bytes`` exactly, or ``signature_table`` would be a table that
    looks like the real one and is not.
    """
    for kind in (FileKind.Png, FileKind.PkgZip, FileKind.Gzip):
        data = kind.rule.signatures[0]
        first = next(e for e in signature_table() if e.matches(data))
        assert first.kind is detect_bytes(data)


# --- FileKind.matches ------------------------------------------------------


def test_matches_agrees_with_detect_for_an_unambiguous_header() -> None:
    assert FileKind.Png.matches(PNG)
    assert detect_bytes(PNG) is FileKind.Png


def test_matches_is_false_for_another_formats_header() -> None:
    assert not FileKind.Jpg.matches(PNG)


def test_matches_is_false_for_rubbish() -> None:
    assert not FileKind.Png.matches(b"not a png at all")
    assert not FileKind.Png.matches(b"")


def test_matches_finds_a_format_detection_can_never_reach() -> None:
    """The case ``detect`` structurally cannot answer.

    ``Ktx2``'s magic begins with all three bytes of ``Ktx``'s, and ``Ktx2`` sits
    first in the table, so a KTX2 file is reported as ``Ktx2`` and never as
    ``Ktx``. ``Qcow2`` shadows ``Qcow`` the same way. Both are real, and both
    were found by walking the table rather than by assuming: of 114 entries,
    these two pairs are the only magic that matches more than one rule.

    Asking about the shadowed format directly is the different answer, and the
    true one.
    """
    ktx2 = bytes([0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A])
    assert FileKind.Ktx2.matches(ktx2)
    assert FileKind.Ktx.matches(ktx2)
    assert detect_bytes(ktx2) is FileKind.Ktx2

    qcow2 = bytes([0x51, 0x46, 0x49, 0xFB])
    assert FileKind.Qcow2.matches(qcow2)
    assert FileKind.Qcow.matches(qcow2)
    assert detect_bytes(qcow2) is FileKind.Qcow2


def test_the_shadowing_pairs_are_the_only_ones() -> None:
    """Pin the count, so a table change cannot quietly add a third.

    Not an exhaustive proof that the table is unambiguous; a proof of that
    would need every byte string, which is not a thing. It is a statement about
    the table as it stands, and it is what makes the pair above a real finding
    rather than a lucky example.
    """
    table = signature_table()
    shadowed: list[tuple[str, list[str]]] = []

    for entry in table:
        if entry.uses_predicate or not entry.signatures:
            continue
        for signature in entry.signatures:
            for offset in entry.offsets:
                buffer = bytearray([0x2A] * (offset + len(signature)))
                buffer[offset : offset + len(signature)] = signature
                hits = [e.kind.name for e in table if e.matches(bytes(buffer))]
                if len(hits) > 1:
                    shadowed.append(tuple(hits))  # type: ignore[arg-type]
                    break
            else:
                continue
            break

    # Listed shadower-first, which is the order ``signature_table`` reports
    # and therefore the order the shadow actually runs in.
    assert sorted(shadowed) == [("Ktx2", "Ktx"), ("Qcow2", "Qcow")]


def test_a_shadowed_format_is_still_first_in_its_own_right() -> None:
    """A bare KTX1 file is a KTX, and detection says so.

    The shadow only runs one way, so the fix for it cannot have been to
    reorder the table.
    """
    ktx1 = bytes([0xAB, 0x4B, 0x54, 0x58, 0x20]) + b"\xBB\r\n\x1a\n"
    assert detect_bytes(ktx1) is FileKind.Ktx
    assert FileKind.Ktx2.matches(ktx1) is False


def test_signature_matches_agrees_with_the_kind_method() -> None:
    for kind in (FileKind.Png, FileKind.Jpg, FileKind.Gzip, FileKind.ELF):
        data = kind.rule.signatures[0]
        assert kind.rule.matches(data)
        assert kind.matches(data)


def test_matches_ignores_the_window_that_detect_honours() -> None:
    """A per-kind question has no window to declare.

    ``detect_bytes`` with a small ``max_bytes_read`` can decline a format whose
    rule would match the bytes it was given, because the rule's declared read
    size does not fit. ``matches`` asks only about the comparison, so it is not
    filtered, and saying so is the point of the distinction.
    """
    assert not detect_bytes(PNG, max_bytes_read=1)
    assert FileKind.Png.matches(PNG)


def test_matches_on_a_predicate_entry_runs_the_predicate() -> None:
    """The two rules decided by a function still answer this way."""
    assert FileKind.ScriptExecute.matches(b"#!/bin/sh\n")
    assert not FileKind.ScriptExecute.matches(b"#!AMR\n")
    assert FileKind.ScriptExecute.matches(b"#!AMR\n") is False


# --- Signature itself ------------------------------------------------------


def test_signature_is_frozen() -> None:
    """The table is a ``static`` in Rust; nothing here can change it."""
    entry = FileKind.Png.rule
    with pytest.raises(Exception):  # noqa: B017 - FrozenInstanceError on 3.8+
        entry.offsets = (5,)  # type: ignore[misc]


def test_max_offset_of_a_predicate_entry_is_zero() -> None:
    for kind in PREDICATE_KINDS:
        assert describe(kind).max_offset == 0  # type: ignore[union-attr]


def test_signature_repr_is_readable() -> None:
    assert "Png" in repr(FileKind.Png.rule)
    assert isinstance(FileKind.Png.rule, Signature)
