"""What the level 1 table contains, and how to ask it a question.

:func:`magical_py.detect` answers one question - what is this file - and
answers it with the first table entry that matched. That is the right default,
and it is not enough for three others:

* **Why did my file come back as this?** The table is ordered, and a format late
  in it is unreachable for any buffer an earlier entry also matches.
  :func:`describe` and :attr:`FileKind.rule` say what an entry compares.
* **Would this one format match?** :meth:`FileKind.matches` asks that, ignoring
  the rest of the table. ``Ktx`` is the case that matters: its magic is a prefix
  of ``Ktx2``'s, so detection reports a KTX2 file as KTX2 and never as KTX.
* **What does a smaller window cost me?** :func:`read_limits` reports the
  crate's read sizes, so ``max_bytes_read=2048`` can be read as a statement
  about which formats survive rather than as a number to look up.

:class:`~magical_py.Signature` is declared in :mod:`magical_py._kinds` next to
the enum, because the two name each other and a type checker will not accept a
deferred import to break the tie. This module holds the functions that walk the
table, and imports from there.
"""

from __future__ import annotations

import dataclasses
from typing import Final

from ._kinds import FileKind, Signature

# Imported from the submodule rather than as `from . import _magical_rs`, because
# the latter names the package, and the package is in the middle of importing
# this module. `_levels` makes the same choice for the same reason.
from ._magical_rs import read_limits as _crate_read_limits
from ._magical_rs import signature_of as _crate_signature_of
from ._magical_rs import signature_table as _crate_signature_table

__all__ = [
    "DEFAULT_MAX_BYTES_READ",
    "ReadLimits",
    "describe",
    "read_limits",
    "signature_table",
]


@dataclasses.dataclass(frozen=True)
class ReadLimits:
    """The crate's read sizes and offsets.

    These are ``pub const`` in Rust, and they are the answer to "what does a
    smaller ``max_bytes_read`` cost me".

    Frozen, and built fresh per call, so nothing a caller does to a returned
    object can reach the next one.
    """

    #: What the bulk of the table declares, and so the floor for most formats.
    default_max_bytes_read: int
    #: The offset that all but four of the entries compare at.
    default_offset: int
    #: Where ISO 9660 keeps its magic, a tuple because there are three places.
    iso_offsets: tuple[int, ...]
    #: What ISO needs, and also the total: :func:`magical_py.bytes_read`.
    iso_max_bytes_read: int
    #: Where a tar header sits, well past the first sector.
    tar_offsets: tuple[int, ...]
    #: What a tar header needs. 262 bytes, and the smallest entry in the table.
    tar_max_bytes_read: int

    @classmethod
    def _from_row(
        cls,
        row: tuple[int, int, list[int], int, list[int], int],
    ) -> ReadLimits:
        (
            default_max_bytes_read,
            default_offset,
            iso_offsets,
            iso_max_bytes_read,
            tar_offsets,
            tar_max_bytes_read,
        ) = row
        return cls(
            default_max_bytes_read=default_max_bytes_read,
            default_offset=default_offset,
            iso_offsets=tuple(iso_offsets),
            iso_max_bytes_read=iso_max_bytes_read,
            tar_offsets=tuple(tar_offsets),
            tar_max_bytes_read=tar_max_bytes_read,
        )


def read_limits() -> ReadLimits:
    """Return the crate's read sizes and offsets.

    Three of them matter more than the rest.

    * ``default_max_bytes_read`` is 2,048, and 113 of the 114 entries declare it
      or more. A window below it drops nearly everything, so the number is not a
      suggestion but the boundary of the usable table.
    * ``iso_offsets`` ends at 36,865, which is why ``iso_max_bytes_read`` is
      36,870 and not the 32,774 the crate's comment claimed. ``max_bytes`` takes
      the *largest* offset, not the first.
    * ``tar_max_bytes_read`` is 262. It is the smallest entry, and it is what a
      tar-only reader needs instead of the full 36,870.

    :returns: A fresh :class:`ReadLimits`.
    """
    row: tuple[int, int, list[int], int, list[int], int] = _crate_read_limits()
    return ReadLimits._from_row(row)


def describe(kind: FileKind) -> Signature | None:
    """Return one format's table entry.

    :param kind: The format to describe.
    :returns: Its :class:`~magical_py.Signature`, or ``None`` if no signature
        produces it. Every :class:`~magical_py.FileKind` member has one, so
        ``None`` would mean the enum and the table had drifted, which
        ``tests/test_drift.py`` fails on. :attr:`FileKind.rule` raises instead
        of returning ``None``, and is the shorter way to ask.
    """
    row: tuple[str, list[bytes], list[int], int, bool] | None = _crate_signature_of(kind.name)
    if row is None:
        return None
    return Signature._from_row(row)


def signature_table() -> tuple[Signature, ...]:
    """Return every table entry, in the order detection tries them.

    The order is carried through rather than sorted, because it is the answer to
    "why did my file come back as this": walking the list and stopping at the
    first match reproduces :func:`magical_py.detect` exactly.

    :returns: One :class:`~magical_py.Signature` per entry, in the table's own
        order.
    """
    rows: list[tuple[str, list[bytes], list[int], int, bool]] = _crate_signature_table()
    return tuple(Signature._from_row(row) for row in rows)


#: What the bulk of the table declares, and the floor for most formats.
#:
#: Bound at import because it is a constant in the crate, and reading it out of
#: a fresh object on every use would be silly.
DEFAULT_MAX_BYTES_READ: Final[int] = read_limits().default_max_bytes_read
