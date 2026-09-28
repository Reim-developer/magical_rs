//! Magic bytes for the extended format table.
//!
//! These formats are **appended** to `SIGNATURE_KIND` rather than interleaved
//! with the original 48, so no pre-existing rule can ever be shadowed by a new
//! one. [`FileKind::match_types`] returns the first match, therefore anything
//! listed here only wins when nothing in the original table matched first.
//!
//! Every signature in this file is covered automatically by
//! `tests/signature_coverage.rs`, which walks the table and asserts each entry
//! detects itself.
//!
//! [`FileKind::match_types`]: crate::magical::magic::FileKind::match_types

// ---------------------------------------------------------------------------
// Archives and compression
// ---------------------------------------------------------------------------

pub(crate) const SEVEN_ZIP: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
pub(crate) const XZ: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];
pub(crate) const LZ4: &[u8] = &[0x04, 0x22, 0x4D, 0x18];
pub(crate) const ZSTD: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];
/// `-lh`, the LHA/LZH archive header. Always at offset 2.
pub(crate) const LZH: &[u8] = &[0x2D, 0x6C, 0x68];
/// `07070`, the ASCII (odc) cpio header.
pub(crate) const CPIO: &[u8] = &[0x30, 0x37, 0x30, 0x37, 0x30];
pub(crate) const ARJ: &[u8] = &[0x60, 0xEA];
pub(crate) const STUFFIT: &[u8] = &[0x53, 0x74, 0x75, 0x66, 0x66, 0x49, 0x74];
pub(crate) const STUFFIT_SIT: &[u8] = &[0x53, 0x49, 0x54, 0x21];
pub(crate) const PAR2: &[u8] = &[0x50, 0x41, 0x52, 0x32, 0x0A, 0x50, 0x4B, 0x54];
/// zlib stream headers. `78 9C` is common in any deflate stream, so this rule
/// is deliberately placed last.
pub(crate) const ZLIB: &[u8] = &[0x78, 0x9C];
pub(crate) const ZLIB_LOW: &[u8] = &[0x78, 0x01];
pub(crate) const ZLIB_BEST: &[u8] = &[0x78, 0xDA];
pub(crate) const ZLIB_DEFAULT_COMP: &[u8] = &[0x78, 0x5E];

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

pub(crate) const TIFF_LE: &[u8] = &[0x49, 0x49, 0x2A, 0x00];
pub(crate) const TIFF_BE: &[u8] = &[0x4D, 0x4D, 0x00, 0x2A];
pub(crate) const TIFF_BIGTIFF_LE: &[u8] = &[0x49, 0x49, 0x2B, 0x00];
pub(crate) const TIFF_BIGTIFF_BE: &[u8] = &[0x4D, 0x4D, 0x2B, 0x00];
/// PCX version byte. The second byte selects the colour encoding.
pub(crate) const PCX: &[u8] = &[0x0A, 0x00];
pub(crate) const PCX_V2: &[u8] = &[0x0A, 0x02];
pub(crate) const PCX_V3: &[u8] = &[0x0A, 0x03];
pub(crate) const PCX_V5: &[u8] = &[0x0A, 0x05];
pub(crate) const DDS: &[u8] = &[0x44, 0x44, 0x53, 0x20];
pub(crate) const KTX: &[u8] = &[0xAB, 0x4B, 0x54, 0x58, 0x20];
pub(crate) const KTX2: &[u8] = &[0xAB, 0x4B, 0x54, 0x58, 0x20, 0x32, 0x30, 0xBB, 0x0D, 0x0A, 0x1A, 0x0A];
pub(crate) const OPENEXR: &[u8] = &[0x76, 0x2F, 0x31, 0x01];
pub(crate) const RADIANCE: &[u8] = &[0x23, 0x3F, 0x52, 0x41, 0x44, 0x49, 0x41, 0x4E, 0x43, 0x45];
/// Bare codestream JXL.
pub(crate) const JPEG_XL_CODESTREAM: &[u8] = &[0xFF, 0x0A];
/// ISOBMFF-wrapped JXL.
pub(crate) const JPEG_XL_CONTAINER: &[u8] =
    &[0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A];
/// Windows cursor directory. Distinct from the icon entry in the base table.
pub(crate) const CURSOR: &[u8] = &[0x00, 0x00, 0x02, 0x00];
pub(crate) const GIMP_XCF: &[u8] = &[0x67, 0x69, 0x6D, 0x70];
pub(crate) const FITS: &[u8] = &[0x53, 0x49, 0x4D, 0x50, 0x4C, 0x45, 0x20, 0x20];

// ---------------------------------------------------------------------------
// Audio and video
// ---------------------------------------------------------------------------

pub(crate) const MIDI: &[u8] = &[0x4D, 0x54, 0x68, 0x64];
pub(crate) const AIFF: &[u8] = &[0x46, 0x4F, 0x52, 0x4D];
pub(crate) const FLAC: &[u8] = &[0x66, 0x4C, 0x61, 0x43];
pub(crate) const WAVPACK: &[u8] = &[0x77, 0x61, 0x76, 0x70];
pub(crate) const CORE_AUDIO: &[u8] = &[0x63, 0x61, 0x66, 0x66];
pub(crate) const MONKEY_AUDIO: &[u8] = &[0x4D, 0x41, 0x43, 0x20];
/// `ftyp`, the ISO base media file format box. Covers MP4, MOV, 3GP, HEIC
/// and AVIF, which differ only in the four brand bytes that follow.
pub(crate) const ISO_MEDIA: &[u8] = &[0x66, 0x74, 0x79, 0x70];
pub(crate) const SWF_UNCOMPRESSED: &[u8] = &[0x46, 0x57, 0x53];
pub(crate) const SWF_ZLIB: &[u8] = &[0x43, 0x57, 0x53];
pub(crate) const SWF_LZMA: &[u8] = &[0x5A, 0x57, 0x53];
pub(crate) const ASF: &[u8] = &[0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11];
pub(crate) const MPEG_PROGRAM_STREAM: &[u8] = &[0x00, 0x00, 0x01, 0xBA];

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

pub(crate) const POSTSCRIPT: &[u8] = &[0x25, 0x21, 0x50, 0x53];
pub(crate) const DJVU: &[u8] = &[0x41, 0x54, 0x26, 0x54, 0x26, 0x46, 0x4F, 0x52, 0x4D];
/// `BOOKMOBI`, located past the `PalmDOC` header.
pub(crate) const MOBIPOCKET: &[u8] = &[0x42, 0x4F, 0x4F, 0x4B, 0x4D, 0x4F, 0x42, 0x49];
pub(crate) const AMR_NARROW: &[u8] = b"#!AMR";
pub(crate) const AMR_WIDE: &[u8] = b"#!AMR-WP";
pub(crate) const CHM: &[u8] = &[0x49, 0x54, 0x53, 0x46];
/// OLE2 compound file, used by MS Office legacy formats, MSI and VSTA projects.
pub(crate) const OLE_COMPOUND_FILE: &[u8] =
    &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

// ---------------------------------------------------------------------------
// Executables and byte code
// ---------------------------------------------------------------------------

pub(crate) const MACH_O_32: &[u8] = &[0xCE, 0xFA, 0xED, 0xFE];
pub(crate) const MACH_O_64: &[u8] = &[0xCF, 0xFA, 0xED, 0xFE];
pub(crate) const MACH_O_32_SWAPPED: &[u8] = &[0xFE, 0xED, 0xFA, 0xCE];
pub(crate) const MACH_O_64_SWAPPED: &[u8] = &[0xFE, 0xED, 0xFA, 0xCF];
pub(crate) const DALVIK: &[u8] = &[0x64, 0x65, 0x78, 0x0A];
pub(crate) const LUA: &[u8] = &[0x1B, 0x4C, 0x75, 0x61];
pub(crate) const WINDOWS_SHORTCUT: &[u8] = &[0x4C, 0x00, 0x00, 0x00, 0x01, 0x14, 0x02, 0x00];

// ---------------------------------------------------------------------------
// Fonts
// ---------------------------------------------------------------------------

pub(crate) const WOFF: &[u8] = &[0x77, 0x4F, 0x46, 0x46];
pub(crate) const WOFF2: &[u8] = &[0x77, 0x4F, 0x46, 0x32];
pub(crate) const FONT_COLLECTION: &[u8] = &[0x74, 0x74, 0x63, 0x66];

// ---------------------------------------------------------------------------
// Data, columnar and machine-learning formats
// ---------------------------------------------------------------------------

pub(crate) const NUMPY: &[u8] = &[0x93, 0x4E, 0x55, 0x4D, 0x50, 0x59];
pub(crate) const HDF5: &[u8] = &[0x89, 0x48, 0x44, 0x46, 0x0D, 0x0A, 0x1A, 0x0A];
pub(crate) const MATLAB: &[u8] = &[0x4D, 0x41, 0x54, 0x4C, 0x42, 0x20, 0x35, 0x2E, 0x30];
pub(crate) const PARQUET: &[u8] = &[0x50, 0x41, 0x52, 0x31];
pub(crate) const APACHE_ORC: &[u8] = &[0x4F, 0x52, 0x43];
pub(crate) const AVRO: &[u8] = &[0x4F, 0x62, 0x6A, 0x01];
pub(crate) const BINARY_PLIST: &[u8] = &[0x62, 0x70, 0x6C, 0x69, 0x73, 0x74];
pub(crate) const PICKLE_V2: &[u8] = &[0x80, 0x02];
pub(crate) const PICKLE_V3: &[u8] = &[0x80, 0x03];
pub(crate) const PICKLE_V4: &[u8] = &[0x80, 0x04];
pub(crate) const PICKLE_V5: &[u8] = &[0x80, 0x05];
pub(crate) const GGUF: &[u8] = &[0x47, 0x47, 0x55, 0x46];
pub(crate) const R_DATA_V2: &[u8] = &[0x52, 0x44, 0x58, 0x32];
pub(crate) const R_DATA_V3: &[u8] = &[0x52, 0x44, 0x58, 0x33];

// ---------------------------------------------------------------------------
// 3D assets and game data
// ---------------------------------------------------------------------------

pub(crate) const GLTF_BINARY: &[u8] = &[0x67, 0x6C, 0x54, 0x46];
pub(crate) const FBX_BINARY: &[u8] =
    &[0x4B, 0x61, 0x79, 0x64, 0x61, 0x72, 0x61, 0x20, 0x46, 0x42, 0x58, 0x20];
/// `ply` followed by a line break. The PLY specification puts a line ending
/// straight after the magic keyword, and it may be LF or CRLF. A bare `ply`
/// would claim any text file starting with that word.
///
/// Two signatures are required because the byte immediately after `ply` is
/// either `\n` or `\r`.
pub(crate) const PLY_LF: &[u8] = b"ply\n";
pub(crate) const PLY_CRLF: &[u8] = b"ply\r\n";
pub(crate) const DOOM_WAD: &[u8] = &[0x49, 0x57, 0x41, 0x44];

// ---------------------------------------------------------------------------
// Disk images
// ---------------------------------------------------------------------------

pub(crate) const QCOW: &[u8] = &[0x51, 0x46, 0x49];
pub(crate) const QCOW2: &[u8] = &[0x51, 0x46, 0x49, 0xFB];
pub(crate) const VIRTUALBOX_VDI: &[u8] =
    &[0x3C, 0x3C, 0x3C, 0x20, 0x4F, 0x72, 0x61, 0x63, 0x6C, 0x65];
pub(crate) const VIRTUAL_HD: &[u8] = &[0x63, 0x6F, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x78];

// ---------------------------------------------------------------------------
// Network capture and transfer
// ---------------------------------------------------------------------------

pub(crate) const PCAP_LE: &[u8] = &[0xD4, 0xC3, 0xB2, 0xA1];
pub(crate) const PCAP_BE: &[u8] = &[0xA1, 0xB2, 0xC3, 0xD4];
pub(crate) const PCAP_NS_LE: &[u8] = &[0x4D, 0x3C, 0xB2, 0xA1];
pub(crate) const PCAP_NS_BE: &[u8] = &[0xA1, 0xB2, 0x3C, 0x4D];
pub(crate) const PCAP_NG: &[u8] = &[0x0A, 0x0D, 0x0D, 0x0A];
pub(crate) const BITTORRENT: &[u8] =
    &[0x64, 0x38, 0x3A, 0x61, 0x6E, 0x6E, 0x6F, 0x75, 0x6E, 0x63, 0x65];
