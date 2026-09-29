# Regenerates bindings/python/python/magical_py/_kinds.py.
#
# The enum is emitted as static Python source rather than built with a
# functional `Enum(...)` call, because pyright resolves members of a
# functional enum as `Unknown` and strict mode would then reject every
# `FileKind.X` reference. Static members also give real IDE completion and
# real docstrings in `help(FileKind)`.
#
# Display names come from the format tables in readme.md, which
# tests/readme_coverage.rs already checks against the Rust detection table,
# so a format added on the Rust side without a matching row here fails the
# build rather than shipping a gap.
#
# The repository root, resolved from this script's own location. Everything
# below reads a relative path from here, so hard-coding an absolute path meant
# the generator only ran in one particular checkout. This script sits in
# `scripts/`, so the root is one level up rather than two.
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $repoRoot

# Variant -> value, mime, extension.
# A null mime or extension means "no registered or verified value", not
# "invented". A wrong MIME type is a documentation bug of the same kind as a
# wrong magic byte, and this project does not ship those.
$data = @{
    '_8BPS'                             = @('psd',   'image/vnd.adobe.photoshop', 'psd')
    'AceCompressed'                     = @('ace',   $null,                        $null)
    'Aiff'                              = @('aiff',  'audio/aiff',                 'aiff')
    'Amr'                               = @('amr',   'audio/amr',                  'amr')
    'AppleDiskImage'                    = @('dmg',   'application/x-apple-diskimage', 'dmg')
    'AppleIconImage'                    = @('icns',  'image/icns',                 'icns')
    'Arj'                               = @('arj',   'application/x-arj',          'arj')
    'Asf'                               = @('asf',   'video/x-ms-asf',             'asf')
    'AuAudioFileFormat'                 = @('au',    'audio/basic',                'au')
    'Avro'                              = @('avro',  'application/avro',           'avro')
    'BinaryPlist'                       = @('bplist', 'application/x-bplist',      'bplist')
    'Bitmap'                            = @('bmp',   'image/bmp',                  'bmp')
    'BitTorrent'                        = @('torrent', 'application/x-bittorrent', 'torrent')
    'BLENDER'                           = @('blend', 'application/x-blender',      'blend')
    'Bzip'                              = @('bz2',   'application/x-bzip2',        'bz2')
    'Cabinet'                           = @('cab',   'application/vnd.ms-cab-compressed', 'cab')
    'Chm'                               = @('chm',   'application/vnd.ms-htmlhelp', 'chm')
    'Class'                             = @('class', 'application/java-vm',        'class')
    'CoreAudio'                         = @('caf',   'audio/x-caf',                'caf')
    'Cpio'                              = @('cpio',  'application/x-cpio',         'cpio')
    'CreativeVoiceFile'                 = @('voc',   'audio/x-voc',                'voc')
    'Cursor'                            = @('cur',   'image/x-icon',               'cur')
    'Dalvik'                            = @('dex',   'application/vnd.android.dex', 'dex')
    'Dds'                               = @('dds',   'image/vnd-ms.dds',           'dds')
    'Deb'                               = @('deb',   'application/vnd.debian.binary-package', 'deb')
    'Djvu'                              = @('djvu',  'image/vnd.djvu',             'djvu')
    'DoomWad'                           = @('wad',   'application/x-doom-wad',     'wad')
    'ELF'                               = @('elf',   $null,                        $null)
    'FbxBinary'                         = @('fbx',   $null,                        'fbx')
    'Fits'                              = @('fits',  'image/fits',                 'fits')
    'Flac'                              = @('flac',  'audio/flac',                 'flac')
    'FlashVideo'                        = @('flv',   'video/x-flv',                'flv')
    'FontCollection'                    = @('ttc',   'font/collection',            'ttc')
    'Gguf'                              = @('gguf',  $null,                        'gguf')
    'GIF'                               = @('gif',   'image/gif',                  'gif')
    'GimpXcf'                           = @('xcf',   'image/x-xcf',                'xcf')
    'GltfBinary'                        = @('glb',   'model/gltf-binary',          'glb')
    'GoogleChromeExtension'             = @('crx',   'application/x-chrome-extension', 'crx')
    'Gzip'                              = @('gz',    'application/gzip',           'gz')
    'Hdf5'                              = @('h5',    'application/x-hdf5',         'h5')
    'ICO'                               = @('ico',   'image/x-icon',               'ico')
    'ISO'                               = @('iso',   'application/x-iso9660-image', 'iso')
    'IsoMedia'                          = @('mp4',   'video/mp4',                  'mp4')
    'JPEG2000'                          = @('jp2',   'image/jp2',                  'jp2')
    'JpegXl'                            = @('jxl',   'image/jxl',                  'jxl')
    'Jpg'                               = @('jpg',   'image/jpeg',                 'jpg')
    'Ktx'                               = @('ktx',   'image/ktx',                  'ktx')
    'Ktx2'                              = @('ktx2',  'image/ktx2',                 'ktx2')
    'Lua'                               = @('luac',  'application/x-lua-bytecode', 'luac')
    'Lz4'                               = @('lz4',   'application/x-lz4',          'lz4')
    'Lzh'                               = @('lzh',   'application/x-lzh-compressed', 'lzh')
    'MachO'                             = @('macho', $null,                        $null)
    'Matlab'                            = @('mat',   'application/x-matlab-data',  'mat')
    'MatroskaMediaContainer'            = @('mkv',   'video/x-matroska',           'mkv')
    'Midi'                              = @('mid',   'audio/midi',                 'mid')
    'Mobipocket'                        = @('mobi',  'application/x-mobipocket-ebook', 'mobi')
    'ModuleForEvenvironmentModules'     = @('module', $null,                       $null)
    'MonkeyAudio'                       = @('ape',   'audio/x-monkeys-audio',      'ape')
    'MP3'                               = @('mp3',   'audio/mpeg',                 'mp3')
    'MpegProgramStream'                 = @('mpg',   'video/mpeg',                 'mpg')
    'MSDOS'                             = @('exe',   'application/vnd.microsoft.portable-executable', 'exe')
    'NoodlesoftHazel'                   = @('hazel', $null,                        $null)
    'Numpy'                             = @('npy',   $null,                        'npy')
    'OGG'                               = @('ogg',   'audio/ogg',                  'ogg')
    'OleCompoundFile'                   = @('ole',   $null,                        $null)
    'OpenExr'                           = @('exr',   'image/x-exr',                'exr')
    'OpenGLIrisPerformer'               = @('iv',    $null,                        $null)
    'OpenTypeFont'                      = @('otf',   'font/otf',                   'otf')
    'Orc'                               = @('orc',   'application/vnd.apache.orc',  'orc')
    'Par2'                              = @('par2',  'application/x-par2',          'par2')
    'Parquet'                           = @('parquet', 'application/vnd.apache.parquet', 'parquet')
    'Pcap'                              = @('pcap',  'application/vnd.tcpdump.pcap', 'pcap')
    'PcapNg'                            = @('pcapng', 'application/x-pcapng',       'pcapng')
    'Pcx'                               = @('pcx',   'image/x-pcx',                'pcx')
    'PDF'                               = @('pdf',   'application/pdf',            'pdf')
    'PhotoCapTemplate'                  = @('pct',   $null,                        'pct')
    'Pickle'                            = @('pickle', $null,                       $null)
    'PkgZip'                            = @('zip',   'application/zip',            'zip')
    'Ply'                               = @('ply',   'text/x-ply',                 'ply')
    'Png'                               = @('png',   'image/png',                  'png')
    'PostScript'                        = @('ps',    'application/postscript',      'ps')
    'Qcow'                              = @('qcow',  'application/x-qemu-qcow',     'qcow')
    'Qcow2'                             = @('qcow2', 'application/x-qemu-qcow',     'qcow2')
    'Radiance'                          = @('hdr',   'image/vnd.radiance',         'hdr')
    'RAR'                               = @('rar',   'application/vnd.rar',        'rar')
    'RData'                             = @('rdata', 'application/x-r-data',      'rdata')
    'RichTextFormat'                    = @('rtf',   'application/rtf',            'rtf')
    'RPM'                               = @('rpm',   'application/x-rpm',          'rpm')
    'ScriptExecute'                     = @('script', $null,                       $null)
    'SerializedJavaData'                = @('ser',   $null,                        $null)
    'SevenZip'                          = @('7z',    'application/x-7z-compressed', '7z')
    'Slob'                              = @('slob',  $null,                        'slob')
    'SQLite'                            = @('sqlite', 'application/vnd.sqlite3',   'sqlite')
    'Stuffit'                           = @('stuffit', 'application/x-stuffit',     'sit')
    'StuffitSit'                        = @('sit',   'application/x-stuffit',      'sit')
    'Swf'                               = @('swf',   'application/x-shockwave-flash', 'swf')
    'Tar'                               = @('tar',   'application/x-tar',          'tar')
    'Tiff'                              = @('tiff',  'image/tiff',                 'tiff')
    'TrueTypeFont'                      = @('ttf',   'font/ttf',                   'ttf')
    'VBScriptEncoded'                   = @('vbe',   'text/x-vbscript',            'vbe')
    'VirtualBoxVdi'                     = @('vdi',   'application/x-vdi',          'vdi')
    'VirtualHd'                         = @('vhd',   'application/x-vhd',          'vhd')
    'Vmdk'                              = @('vmdk',  'application/x-vmdk',         'vmdk')
    'WASM'                              = @('wasm',  'application/wasm',           'wasm')
    'WavPack'                           = @('wv',    'audio/x-wavpack',            'wv')
    'WEBP'                              = @('webp',  'image/webp',                 'webp')
    'WindowImagingFormat'               = @('wim',   'application/x-ms-wim',       'wim')
    'WindowsShortcut'                   = @('lnk',   'application/x-ms-shortcut',  'lnk')
    'Woff'                              = @('woff',  'font/woff',                  'woff')
    'Woff2'                             = @('woff2', 'font/woff2',                 'woff2')
    'XML'                               = @('xml',   'application/xml',            'xml')
    'Xz'                                = @('xz',    'application/x-xz',           'xz')
    'Zlib'                              = @('zlib',  $null,                        $null)
    'Zstd'                              = @('zst',   'application/zstd',           'zst')
}

# --- read the display names out of the format tables -----------------------

$names = @{}
foreach ($line in Get-Content 'readme.md') {
    $cells = $line.Split('|') | ForEach-Object { $_.Trim() }
    if ($cells.Count -eq 6 -and $cells[1]) {
        $offsets = $cells[4].Split(',') | ForEach-Object { $_.Trim() }
        if ($offsets.Count -gt 0 -and -not ($offsets | Where-Object { $_ -notmatch '^\d+$' })) {
            $variant = $cells[2].Trim('`')
            if ($variant -and $variant -match '^[A-Za-z_][A-Za-z0-9_]*$') {
                $names[$variant] = ($cells[1] -replace '`', '')
            }
        }
    }
}

# --- cross-check the two tables before emitting anything --------------------

if ($names.Count -ne $data.Count) {
    throw "readme.md has $($names.Count) rows but the metadata table has $($data.Count)"
}
foreach ($variant in $names.Keys) {
    if (-not $data.ContainsKey($variant)) { throw "no metadata for $variant" }
}
foreach ($variant in $data.Keys) {
    if (-not $names.ContainsKey($variant)) { throw "no readme row for $variant" }
}

$values = $data.Values | ForEach-Object { $_[0] }
$duplicates = $values | Group-Object | Where-Object { $_.Count -gt 1 }
if ($duplicates) {
    # A duplicate value would silently turn the second member into an alias
    # of the first, so the enum would have fewer members than the table.
    throw "duplicate enum values: $($duplicates.Name -join ', ')"
}

# --- emit -------------------------------------------------------------------

$sb = [System.Text.StringBuilder]::new()
function Emit([string]$text) { [void]$sb.AppendLine($text) }

Emit '"""The :class:`FileKind` enum, one member per detectable file format,'
Emit ''
Emit 'and the :class:`Signature` that describes how each one is detected.'
Emit ''
Emit 'Generated by ``scripts/gen_kinds.ps1``. Do not edit by hand.'
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
EmitRust '//! Generated by `scripts/gen_kinds.ps1` from the format tables in'
EmitRust '//! `readme.md` plus the metadata table at the top of that script. Do not'
EmitRust '//! edit by hand; `tests/kinds_meta.rs` holds it against `SIGNATURE_KIND`.'
EmitRust '//!'
EmitRust '//! The readme is the source for the display names because it is what a reader'
EmitRust '//! sees, and a crate whose `Debug` output says `Png` where its documentation'
EmitRust '//! says "PNG" has two names for one thing. The MIME types and extensions are'
EmitRust '//! in the generator because they are not in the readme and are not derivable'
EmitRust '//! from magic bytes: `50 4B 03 04` is a zip container, and whether that is'
EmitRust '//! `application/zip` or `application/java-archive` or'
EmitRust '//! `application/vnd.android.package-archive` is a question about the bytes'
EmitRust '//! *inside* it, which is a longer answer than a magic number.'
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
