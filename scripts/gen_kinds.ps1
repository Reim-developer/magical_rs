# Regenerates the Rust and Python format metadata.
#
#   crates/magical_rs/src/magical/kinds_meta.rs
#   bindings/python/python/magical_py/_kinds.py
#
# Both are generated from `formats.json`, which is the one place a format's
# short name, display name, MIME type and extension are written down. It was not
# always that way: the MIME types and extensions lived in a hashtable at the top
# of this script and the display names were parsed back out of the markdown
# tables in readme.md, so the same value was written down twice and a format
# added on the Rust side needed a hand-edited row here to match.
#
# The magic bytes are still not in the dataset, because they are not metadata:
# they are facts about the formats, and `crates/magical_rs/tests/readme_coverage.rs`
# checks the readme against `SIGNATURE_KIND` -- so the bytes are verified against
# the code that matches on them, and the names are verified against formats.json.
#
# `abi_order` is read out of `pub enum FileKind` when the dataset is built (see
# scripts/build_formats.mjs) rather than here, because it is the position in a
# declaration rather than a fact about a format. The check below that it is a
# permutation of 0..113 is what keeps a hand-edited renumbering from shipping.
#
# The repository root, resolved from this script's own location. Everything
# below reads a relative path from here, so hard-coding an absolute path meant
# the generator only ran in one particular checkout. This script sits in
# `scripts/`, so the root is one level up rather than two.
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $repoRoot

# --- read the dataset --------------------------------------------------------

$datasetPath = Join-Path $repoRoot 'formats.json'
if (-not (Test-Path -LiteralPath $datasetPath)) {
    throw "formats.json is missing. It is the source of truth for this script; nothing else is."
}
$dataset = Get-Content -Raw -LiteralPath $datasetPath | ConvertFrom-Json

# Pinned because every generated file and every test states the count, so a
# silent 115 has to take a deliberate edit rather than arrive by being one line
# longer in a data file.
$expected = 114
if ($dataset.formats.Count -ne $expected) {
    throw ('formats.json has {0} formats, expected {1}' -f $dataset.formats.Count, $expected)
}

# variant -> (token, mime, extension)
# A null mime or extension means "no registered or verified value", not
# "invented". A wrong MIME type is a documentation bug of the same kind as a
# wrong magic byte, and this project does not ship those.
$data = @{}

# variant -> display name
$names = @{}

# abi_order -> variant, so the positions can be checked as a permutation.
# Keyed by text because `ConvertFrom-Json` hands back `Int64` and the loop below
# counts with `Int32`, and a PowerShell hashtable treats those as different keys.
$abi = @{}

foreach ($format in $dataset.formats) {
    $variant = $format.variant
    if (-not $variant -or $variant -notmatch '^[A-Za-z_][A-Za-z0-9_]*$') {
        throw ('{0} is not a Rust identifier, and `variant` is the name of the enum member' -f $variant)
    }
    if ($data.ContainsKey($variant)) { throw "two formats are both $variant" }
    if ($names.ContainsKey($variant)) { throw "two formats are both $variant" }

    foreach ($field in @('token', 'name')) {
        $value = $format.$field
        if (-not $value -or -not ($value -is [string])) { throw "$variant has no $field" }
    }

    # The two nullable fields have to be *present*. `$null` is a real answer --
    # "no registered MIME type" -- and a missing key would produce the same
    # `$null` here while meaning something different, so the difference is
    # checked rather than inferred from the value.
    foreach ($field in @('mime', 'extension')) {
        if (-not $format.PSObject.Properties.Name.Contains($field)) {
            throw "$variant has no $field key; use null for none"
        }
        $value = $format.$field
        if ($null -ne $value -and -not ($value -is [string] -and $value.Length -gt 0)) {
            throw "$variant.$field is neither a non-empty string nor null"
        }
    }

    $order = $format.abi_order
    if (-not ($order -is [int] -or $order -is [long])) {
        throw "$variant has no numeric abi_order"
    }
    $key = [string]$order
    if ($abi.ContainsKey($key)) {
        throw ('abi_order {0} is both {1} and {2}' -f $order, $abi[$key], $variant)
    }

    $abi[$key] = $variant
    $data[$variant] = @($format.token, $format.mime, $format.extension)
    $names[$variant] = $format.name
}

# The positions have to be 0 to n-1 with no gaps, because a kind crosses the wasm
# boundary as its Rust discriminant: entry N here is what the compiled module
# reports as N, and a gap would make one name answer for another's index.
for ($at = 0; $at -lt $dataset.formats.Count; $at++) {
    if (-not $abi.ContainsKey([string]$at)) { throw "abi_order $at is missing" }
}

# A duplicate token would silently turn the second enum member into an alias of
# the first, so the Python enum would have fewer members than the table.
$tokens = $data.Values | ForEach-Object { $_[0] }
$duplicates = $tokens | Group-Object | Where-Object { $_.Count -gt 1 }
if ($duplicates) {
    throw ('duplicate tokens: {0}' -f (($duplicates | ForEach-Object { $_.Name }) -join ', '))
}

# --- emit -------------------------------------------------------------------

$sb = [System.Text.StringBuilder]::new()
function Emit([string]$text) { [void]$sb.AppendLine($text) }

Emit '"""The :class:`FileKind` enum, one member per detectable file format,'
Emit ''
Emit 'and the :class:`Signature` that describes how each one is detected.'
Emit ''
Emit 'Generated by ``scripts/gen_kinds.ps1`` from ``formats.json``. Do not edit by'
Emit 'hand; ``tests/test_metadata.py`` reads the same file back.'
Emit ''
Emit '``Signature`` lives here rather than in ``_signatures`` because the two are'
Emit 'mutually recursive: a ``Signature`` names a ``FileKind``, and a ``FileKind``'
Emit 'has a ``rule`` that is a ``Signature``. Keeping them in one module is what'
Emit 'avoids an import cycle, and the alternative was a deferred import that a type'
Emit 'checker flags anyway. ``_signatures`` holds the functions that walk the'
Emit 'table and imports from here.'
Emit '"""'
Emit ''
Emit 'from __future__ import annotations'
Emit ''
Emit 'import dataclasses'
Emit 'from enum import Enum'
Emit 'from typing import Final'
Emit ''
Emit 'from ._magical_rs import kind_matches, signature_of'
Emit ''
Emit '__all__ = ["FileKind", "Signature"]'
Emit ''
Emit '# The row the extension returns is spelled out at each unpacking site rather'
Emit '# than aliased. A quoted module-level alias would be the obvious way to keep'
Emit '# the two sites in step, but pyright does not accept a string as a type'
Emit '# alias, and `typing.TypeAlias` is 3.10 while this package supports 3.8.'
Emit '# `tests/test_stub.py` pins the extension against the stub, which is where'
Emit '# the shape is actually asserted.'
Emit ''
Emit ''
Emit '@dataclasses.dataclass(frozen=True)'
Emit 'class Signature:'
Emit '    """One entry of the built-in detection table.'
Emit ''
Emit '    Frozen because the table is a ``static`` in Rust and nothing here can'
Emit '    change it: a caller that wants a different answer wants a level 2 rule,'
Emit '    which is what :class:`~magical_py.MagicCustom` is for.'
Emit '    """'
Emit ''
Emit '    kind: FileKind'
Emit '    signatures: tuple[bytes, ...]'
Emit '    offsets: tuple[int, ...]'
Emit '    max_bytes_read: int'
Emit '    uses_predicate: bool'
Emit ''
Emit '    @property'
Emit '    def max_offset(self) -> int:'
Emit '        """The furthest byte position this entry compares at.'
Emit ''
Emit '        Zero for a predicate entry, whose rule decides on the buffer as a'
Emit '        whole rather than at a position.'
Emit '        """'
Emit '        return max(self.offsets, default=0)'
Emit ''
Emit '    def matches(self, data: bytes) -> bool:'
Emit '        """Report whether *this* format''s rule matches, ignoring the table.'
Emit ''
Emit '        The one thing :func:`magical_py.detect` cannot tell you. Detection'
Emit '        stops at the first entry that matched, so a format whose magic is'
Emit '        also another format''s is unreachable through it. A KTX2 file answers'
Emit '        :func:`~magical_py.detect_bytes` with ``FileKind.Ktx2`` and this with'
Emit '        ``True`` for ``FileKind.Ktx``, whose three magic bytes are a prefix'
Emit '        of KTX2''s.'
Emit ''
Emit '        :param data: The leading bytes of a file.'
Emit '        :returns: Whether this format''s own rule would match them.'
Emit '        """'
Emit '        return kind_matches(self.kind.name, data)'
Emit ''
Emit '    @classmethod'
Emit '    def _from_row(cls, row: tuple[str, list[bytes], list[int], int, bool]) -> Signature:'
Emit '        kind, signatures, offsets, max_bytes_read, uses_predicate = row'
Emit '        return cls('
Emit '            kind=FileKind[kind],'
Emit '            signatures=tuple(signatures),'
Emit '            offsets=tuple(offsets),'
Emit '            max_bytes_read=max_bytes_read,'
Emit '            uses_predicate=uses_predicate,'
Emit '        )'
Emit ''
Emit ''
Emit 'class FileKind(Enum):'
Emit '    """A file format that ``magical_rs`` can detect.'
Emit ''
Emit '    Members are named after the Rust ``FileKind`` variants, so a name that'
Emit '    appears in the ``magical_rs`` documentation appears here too. The'
Emit '    :attr:`value` of each member is a short, stable identifier intended for'
Emit '    serialisation; use :attr:`mime` when you need a media type.'
Emit ''
Emit '    Detect a format with :func:`magical_py.detect` or'
Emit '    :func:`magical_py.detect_bytes`.'
Emit '    """'
Emit ''
Emit '    @classmethod'
Emit '    def from_value(cls, value: str) -> FileKind:'
Emit '        """Look a member up by its serialised value.'
Emit ''
Emit '        :param value: A member :attr:`value`, such as ``"png"``.'
Emit '        :returns: The matching member.'
Emit '        :raises ValueError: If no member has that value.'
Emit '        """'
Emit '        return cls(value)'
Emit ''
Emit '    @property'
Emit '    def description(self) -> str:'
Emit '        """A human-readable name for the format, such as ``"PNG"``."""'
Emit '        return _META[self.name][0]'
Emit ''
Emit '    @property'
Emit '    def mime(self) -> str | None:'
Emit '        """The registered media type, or ``None`` if there is none.'
Emit ''
Emit '        ``None`` means no media type is registered or verified for this'
Emit '        format. It never means "unknown", which would be a guess.'
Emit '        """'
Emit '        return _META[self.name][1]'
Emit ''
Emit '    @property'
Emit '    def extension(self) -> str | None:'
Emit '        """The conventional file extension, or ``None`` if there is none.'
Emit ''
Emit '        The extension carries no leading dot. It is advisory: detection'
Emit '        never looks at the file name.'
Emit '        """'
Emit '        return _META[self.name][2]'
Emit ''
Emit '    @property'
Emit '    def rule(self) -> Signature:'
Emit '        """The table entry that detects this format.'
Emit ''
Emit '        What the format is matched on: the bytes, the offsets they are'
Emit '        compared at, and the read size the entry declares. A format'
Emit '        decided by a function rather than a fixed pattern reports'
Emit '        ``uses_predicate`` and no signatures.'
Emit ''
Emit '        :raises ValueError: If no signature produces this format, which'
Emit '            would mean the enum and the table had drifted apart.'
Emit '        """'
Emit '        entry = signature_of(self.name)'
Emit '        if entry is None:'
Emit '            raise ValueError('
Emit '                f"{self.name} is not a format the detection table produces"'
Emit '            )'
Emit '        return Signature._from_row(entry)'
Emit ''
Emit '    def matches(self, data: bytes) -> bool:'
Emit '        """Report whether this format''s own rule matches ``data``.'
Emit ''
Emit '        :func:`magical_py.detect` stops at the first table entry that'
Emit '        matched, so a format whose magic is also another format''s is'
Emit '        unreachable through it. This asks only about this one.'
Emit ''
Emit '        :param data: The leading bytes of a file.'
Emit '        :returns: Whether this format''s own rule would match them.'
Emit '        """'
Emit '        return kind_matches(self.name, data)'

foreach ($variant in ($names.Keys | Sort-Object)) {
    $meta = $data[$variant]
    $name = $names[$variant].Replace("'", "\'")
    Emit ''
    Emit "    $variant = '$($meta[0])'"
    # Built by concatenation because PowerShell does not use backslash as an
    # escape character, so an escaped `\"\"\"` here would emit a literal `\`.
    Emit ('    """' + $name + '."""')
}

Emit ''
Emit ''
Emit '#: member name -> (description, mime, extension)'
Emit '_META: Final[dict[str, tuple[str, str | None, str | None]]] = {'

foreach ($variant in ($names.Keys | Sort-Object)) {
    $meta = $data[$variant]
    $name = $names[$variant].Replace("'", "\'")
    $mime = if ($null -eq $meta[1]) { 'None' } else { "'" + $meta[1] + "'" }
    $ext = if ($null -eq $meta[2]) { 'None' } else { "'" + $meta[2] + "'" }
    Emit "    '$variant': ('$name', $mime, $ext),"
}

Emit '}'
Emit ''
Emit '# ``enum`` ignores a string literal written after a member, so without this'
Emit '# ``FileKind.Png.__doc__`` would fall back to the class docstring and'
Emit '# ``help(FileKind)`` would list a hundred and fourteen identical paragraphs.'
Emit '# Assigning it explicitly is what makes the display names show up there.'
Emit 'for _name, _meta in _META.items():'
Emit '    setattr(FileKind[_name], "__doc__", _meta[0])'

# Absolute, built from `$repoRoot`, and not the relative path this used to be.
#
# `Set-Location` above is enough for `Get-Content`, which is a PowerShell
# cmdlet and resolves against PowerShell's own location. `WriteAllText` is a
# .NET method and resolves against the process working directory, which
# `Set-Location` does not move. So a relative `$out` silently meant the
# generator only ever wrote the file when it was run from the repository root,
# and from anywhere else it threw `DirectoryNotFoundException` after the whole
# table had been parsed. The comment above this script claimed otherwise, and
# the claim was checked only by reading it.
$out = Join-Path $repoRoot 'bindings\python\python\magical_py\_kinds.py'
# Written without a BOM. PowerShell 5.1's `-Encoding utf8` emits one, and this
# file ships inside the wheel, where a leading U+FEFF is noise.
[System.IO.File]::WriteAllText(
    $out,
    $sb.ToString(),
    (New-Object System.Text.UTF8Encoding $false))
Remove-Item (Join-Path $repoRoot 'bindings\python\python\magical_py\_kinds_data.py') `
    -ErrorAction SilentlyContinue
Write-Output "wrote $out with $($names.Count) members"

# ---------------------------------------------------------------------------
# The same table, emitted as Rust.
#
# The bindings have their own copies of this metadata because each of them is
# loaded into a language that cannot call into the library, and a third copy for
# the library's own Rust users is not a copy at all -- it is the one the other
# two could have been reading. So it lives here, as inherent methods on
# `FileKind`, and the library's own tests hold it against `SIGNATURE_KIND`.
# ---------------------------------------------------------------------------

# Rust string literals, escaped. PowerShell has no backslash escape, so a `"\""`
# here would emit two characters rather than one, and a display name containing
# a quote would produce a file that does not parse. None of the 114 names do,
# which is exactly why a name starting with one would not be noticed.
function Escape-RustString([string]$value) {
    return $value.Replace('\', '\\').Replace('"', '\"')
}

# `Option<&'static str>`, with `None` for a value the table has none for. A
# missing MIME type is left missing rather than guessed: this project does not
# ship a wrong MIME type, and `None` says "not registered" where a guess would
# say "binary".
function Emit-RustOption($value) {
    if ($null -eq $value) { return 'None' }
    return 'Some("' + (Escape-RustString $value) + '")'
}

# The order every emitted list uses: by the short value the Python binding
# carries as its enum value, then by the variant name to break a tie.
#
# Defined once because the emitted `ALL_KINDS`, the `meta` arms and the
# `from_name` arms all have to agree, and a list that is sorted in three places
# is a list that is sorted in two places the next time someone edits one of them.
# The short value leads because it is what a reader would sort by: "7z" before
# "ace" before "amr". The tiebreaker is not decorative -- `Sort-Object` is stable
# but a Hashtable has no order to be stable *with*, so without it two formats
# sharing a short value would swap places between runs and every run would
# rewrite the file.
$sorted = $names.Keys | Sort-Object { $data[$_][0] }, { $_ }

$rust = [System.Text.StringBuilder]::new()
function EmitRust([string]$text) { [void]$rust.AppendLine($text) }

EmitRust '//! The name, MIME type and conventional extension of every format.'
EmitRust '//!'
EmitRust '//! Generated by `scripts/gen_kinds.ps1` from `formats.json`. Do not edit'
EmitRust '//! by hand; `tests/kinds_meta.rs` holds it against `SIGNATURE_KIND`.'
EmitRust '//!'
EmitRust '//! `formats.json` is the one place a name, a MIME type and an extension are'
EmitRust '//! written down, for this file and for the Python and JavaScript bindings at'
EmitRust '//! once. None of them is derivable from magic bytes: `50 4B 03 04` is a zip'
EmitRust '//! container, and whether that is `application/zip` or'
EmitRust '//! `application/java-archive` or `application/vnd.android.package-archive` is a'
EmitRust '//! question about the bytes *inside* it, which is a longer answer than a magic'
EmitRust '//! number. A wrong MIME type is served to a browser, so the project does not'
EmitRust '//! ship one: a format with no registered type answers `None` rather than a guess.'
EmitRust ''
EmitRust 'use crate::magical::magic::FileKind;'
EmitRust ''
EmitRust '/// Every format, in the order the metadata table is sorted.'
EmitRust '///'
# The backticks in the two lines below are literal characters in the generated
# file, and writing them from PowerShell needs a single-quoted string. A
# double-quoted one eats them: the backtick is PowerShell's escape character,
# and `` `a `` is the alert escape, so "`ace`" comes out as a BEL followed by
# "ce" and "`amr`" as "mr". A BEL in a doc comment is invisible in a diff and
# obvious in the rendered documentation, which is the worst place to find it.
#
# `clippy::doc_markdown` then rejects the result for naming `SIGNATURE_KIND`
# and `7z` without backticks, so the damage is caught -- but the error names
# the generated file, not the line in this script that caused it.
EmitRust '/// Sorted by the short name the Python binding uses as its enum value -- `7z`'
EmitRust '/// before `ace` before `amr` -- and by nothing else. It is deliberately not'
EmitRust '/// `SIGNATURE_KIND` order, which is the order detection has to try rules in:'
EmitRust '/// 71 of its 114 entries are out of enum order, so a catalogue built from it'
EmitRust '/// reads as though something were wrong with the formats near the end. Nor is'
EmitRust '/// it enum order, which is neither alphabetical nor grouped, and which changed'
EmitRust '/// meaning last time a format was added in the section it belonged to.'
EmitRust '///'
EmitRust '/// A display name, a MIME type or an extension is the stable way to name a'
EmitRust '/// format in a table, and a list of those is what a reader can scan.'
# The parentheses are load-bearing and not decoration. Written as
# `EmitRust 'text' + $count + '] = ['`, PowerShell parses that as the command
# `EmitRust` with five arguments -- the literal, `+`, the count, `+`, and the
# tail -- rather than as one concatenated string. The function takes the first
# and discards the rest, so the emitted line came out as
# `pub static ALL_KINDS: [FileKind; ` with no length and no `= [`, and the file
# it produced did not compile. The format operator is here instead of
# concatenation because it cannot be misparsed that way.
EmitRust ('pub static ALL_KINDS: [FileKind; {0}] = [' -f $names.Count)
foreach ($variant in $sorted) {
    EmitRust "    FileKind::$variant,"
}
EmitRust '];'
EmitRust ''
EmitRust 'impl FileKind {'
EmitRust ''
EmitRust '    /// All four answers at once, in one match.'
EmitRust '    ///'
EmitRust '    /// One `match` rather than four, because a 114-arm `match` is a hundred lines'
EmitRust '    /// long and five of them is five hundred lines of generated code where one'
EmitRust '    /// would do -- and five places for a `clippy::too_many_lines` allowance to be'
EmitRust '    /// repeated. Four one-line accessors read the tuple.'
EmitRust '    ///'
EmitRust '    /// `clippy::too_many_lines` is allowed rather than avoided because there is no'
EmitRust '    /// way to avoid it: 114 arms is 116 lines. The alternative is an index into a'
EmitRust '    /// `static` slice, which trades a lint the tool has a number for a bounds'
EmitRust '    /// check on every call.'
EmitRust '    #[allow(clippy::too_many_lines)]'
EmitRust '    const fn meta(self) -> (&''static str, Option<&''static str>, Option<&''static str>, &''static str) {'
EmitRust '        match self {'
foreach ($variant in $sorted) {
    $display = Escape-RustString $names[$variant]
    $mime = Emit-RustOption $data[$variant][1]
    $ext = Emit-RustOption $data[$variant][2]
    # `Self::` rather than `FileKind::`, because the crate denies
    # `clippy::use_self` and a pattern arm is still a path.
    EmitRust "            Self::$variant => (`"$display`", $mime, $ext, `"$variant`"),"
}
EmitRust '        }'
EmitRust '    }'
EmitRust ''
EmitRust '    /// The name this format is written with in documentation and in a `file`'
EmitRust '    /// listing, such as `"PNG"` or `"Zip / JAR / APK"`.'
EmitRust '    ///'
EmitRust '    /// `const` because the answer is a literal in the generated body, and a caller'
EmitRust '    /// building a table of labels at compile time should not have to give that up.'
EmitRust '    #[must_use]'
EmitRust '    #[inline]'
EmitRust '    pub const fn display_name(self) -> &''static str {'
EmitRust '        self.meta().0'
EmitRust '    }'
EmitRust ''
EmitRust '    /// The registered MIME type, or [`None`] when there is none.'
EmitRust '    ///'
EmitRust '    /// [`None`] means "not registered or not verified", never "unknown". A caller'
EmitRust '    /// that needs an answer should fall back to `application/octet-stream` itself,'
EmitRust '    /// so the choice stays visible at the call site rather than baked in here.'
EmitRust '    #[must_use]'
EmitRust '    #[inline]'
EmitRust '    pub const fn mime(self) -> Option<&''static str> {'
EmitRust '        self.meta().1'
EmitRust '    }'
EmitRust ''
EmitRust '    /// The conventional file extension, or [`None`] when there is none.'
EmitRust '    ///'
EmitRust '    /// Without a leading dot, and advisory: detection never reads a file name, so a'
EmitRust '    /// `.jpg` holding a PNG is reported as a PNG. The extension is for choosing'
EmitRust '    /// what to *write*, which is the one question a magic number cannot answer.'
EmitRust '    #[must_use]'
EmitRust '    #[inline]'
EmitRust '    pub const fn extension(self) -> Option<&''static str> {'
EmitRust '        self.meta().2'
EmitRust '    }'
EmitRust ''
EmitRust '    /// The name of the enum variant, which is what `format!("{:?}")` prints.'
EmitRust '    ///'
EmitRust '    /// Its own method because the two names answer different questions and are not'
EmitRust '    /// interchangeable: this one is `Png` and [`Self::display_name`] is `"PNG"`,'
EmitRust '    /// and a caller that wants a stable identifier wants this one.'
EmitRust '    #[must_use]'
EmitRust '    #[inline]'
EmitRust '    pub const fn variant_name(self) -> &''static str {'
EmitRust '        self.meta().3'
EmitRust '    }'
EmitRust ''
EmitRust '    /// The format whose [`Self::variant_name`] is `name`, or [`None`].'
EmitRust '    ///'
EmitRust '    /// Exact and case-sensitive, and that is the point: `Png`, `png` and `PNG` are'
EmitRust '    /// three spellings and only one is the identifier this crate uses. A loose'
EmitRust '    /// lookup belongs in whatever parses a command line, where forgiving is right,'
EmitRust '    /// rather than in the library, where a wrong answer is silent.'
EmitRust '    ///'
EmitRust '    /// ```rust'
EmitRust '    /// use magical_rs::magical::magic::FileKind;'
EmitRust '    ///'
EmitRust '    /// assert_eq!(FileKind::from_name("Png"), Some(FileKind::Png));'
EmitRust '    /// assert_eq!(FileKind::from_name("png"), None);'
EmitRust '    /// assert_eq!(FileKind::from_name("Nonsense"), None);'
EmitRust '    /// ```'
EmitRust '    #[must_use]'
EmitRust '    #[inline]'
EmitRust '    // A hundred and fourteen arms, for the reason the meta match gives. Unlike'
EmitRust '    // that one this could have been a table, but a match on a string compiles'
EmitRust '    // to a length switch and a memcmp rather than a scan over an array.'
EmitRust '    #[allow(clippy::too_many_lines)]'
EmitRust '    pub fn from_name(name: &str) -> Option<Self> {'
EmitRust '        match name {'
foreach ($variant in $sorted) {
    EmitRust "            `"$variant`" => Some(Self::$variant),"
}
EmitRust '            _ => None,'
EmitRust '        }'
EmitRust '    }'
EmitRust '}'
EmitRust ''

$rustOut = Join-Path $repoRoot 'crates\magical_rs\src\magical\kinds_meta.rs'
[System.IO.File]::WriteAllText(
    $rustOut,
    $rust.ToString(),
    (New-Object System.Text.UTF8Encoding $false))
Write-Output "wrote $rustOut with $($names.Count) formats"
