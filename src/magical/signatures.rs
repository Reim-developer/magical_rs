use crate::magical::bytes_read::{
    DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS, TAR_MAX_BYTES_READ,
    TAR_OFFSETS,
};
use crate::magical::ext_fn::shebang::is_shebang;
use crate::magical::ext_fn::webp::is_webp;
use crate::magical::magic::FileKind;
use crate::magical::match_rules::MatchRules;

use crate::magical::signatures_ext::{
    AIFF, AMR_NARROW, AMR_WIDE, APACHE_ORC, ARJ, ASF, AVRO, BINARY_PLIST, BITTORRENT, CHM,
    CORE_AUDIO, CPIO, CURSOR,
    DALVIK, DDS, DJVU, DOOM_WAD, FBX_BINARY, FITS, FLAC, FONT_COLLECTION, GGUF, GIMP_XCF,
    GLTF_BINARY, HDF5, ISO_MEDIA, JPEG_XL_CODESTREAM, JPEG_XL_CONTAINER, KTX, KTX2, LUA, LZH,
    LZ4, MACH_O_32, MACH_O_32_SWAPPED, MACH_O_64, MACH_O_64_SWAPPED, MATLAB, MIDI, MOBIPOCKET,
    MONKEY_AUDIO, MPEG_PROGRAM_STREAM, NUMPY, OLE_COMPOUND_FILE, OPENEXR, PAR2, PARQUET, PCAP_BE,
    PCAP_LE, PCAP_NG, PCAP_NS_BE, PCAP_NS_LE, PCX, PCX_V2, PCX_V3, PCX_V5, PICKLE_V2, PICKLE_V3,
    PICKLE_V4, PICKLE_V5, PLY_CRLF, PLY_LF, POSTSCRIPT, QCOW, QCOW2, R_DATA_V2, R_DATA_V3,
    RADIANCE,
    SEVEN_ZIP, STUFFIT, STUFFIT_SIT, SWF_LZMA, SWF_UNCOMPRESSED, SWF_ZLIB, TIFF_BE,
    TIFF_BIGTIFF_BE, TIFF_BIGTIFF_LE, TIFF_LE, VIRTUALBOX_VDI, VIRTUAL_HD, WAVPACK,
    WINDOWS_SHORTCUT, WOFF, WOFF2, XZ, ZLIB, ZLIB_BEST, ZLIB_DEFAULT_COMP, ZLIB_LOW, ZSTD,
};

/// Build one `Magic` entry for the extended format table.
///
/// Defined before `SIGNATURE_KIND` because `macro_rules!` must appear textually
/// before any use. The original 48 entries are written out longhand below.
macro_rules! entry {
    ($kind:ident, [$($sig:ident),* $(,)?], [$($off:expr),* $(,)?]) => {
        Magic {
            signatures: &[$($sig),*],
            offsets: &[$($off),*],
            max_bytes_read: DEFAULT_MAX_BYTES_READ,
            kind: FileKind::$kind,
            rules: MatchRules::Default,
        }
    };
}

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const GZIP_SIGNATURE: &[u8] = &[0x1F, 0x8B];
/// `BZh`, the bzip2 block header. The trailing `h` is part of the format, so
/// the two-byte prefix `BZ` is not a valid signature: it would claim any file
/// that happens to begin with those letters.
const BZIP_SIGNATURE: &[u8] = &[0x42, 0x5A, 0x68];
const PKG_ZIP_SIGNATURE: &[u8] = &[0x50, 0x4B, 0x03, 0x04];
const BITMAP_SIGNATURE: &[u8] = &[0x42, 0x4D];
const TAR_SIGNATURE: &[u8] = &[0x75, 0x73, 0x74, 0x61, 0x72];
const MS_DOS_SIGNATURE: &[u8] = &[0x4D, 0x5A];
const JPG_SIGNATURE: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0];
const CLASS_SIGNATURE: &[u8] = &[0xCA, 0xFE, 0xBA, 0xBE];
const MP3_SIGNATURE_1: &[u8] = &[0xFF, 0xFB];
const MP3_SIGNATURE_2: &[u8] = &[0xFF, 0xF3];
const MP3_SIGNATURE_3: &[u8] = &[0xFF, 0xF2];
const ISO_SIGNATURE: &[u8] = &[0x43, 0x44, 0x30, 0x30, 0x31];
const RPM_SIGNATURE: &[u8] = &[0xED, 0xAB, 0xEE, 0xDB];
const SQLITE_SIGNATURE: &[u8] = &[
    0x53, 0x51, 0x4C, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x74, 0x20, 0x33, 0x00,
];
const XML_SIGNATURE: &[u8] = &[0x3C, 0x3F, 0x78, 0x6D, 0x6C, 0x20];
const ICO_SIGNATURE: &[u8] = &[0x00, 0x00, 0x01, 0x00];
const WASM_SIGNATURE: &[u8] = &[0x00, 0x61, 0x73, 0x6D];
const DEB_SIGNATURE: &[u8] = &[0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E, 0x0A];
const SCRIPT_EXECUTE_SIGNATURE: &[u8] = &[0x23, 0x21];
const RAR_SIGNATURE: &[&[u8]] = &[
    &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00],
    &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00],
];
const ELF_SIGNATURE: &[u8] = &[0x7F, 0x45, 0x4C, 0x46];
const OGG_SIGNATURE: &[u8] = &[0x4F, 0x67, 0x67, 0x53];
const _8BPS_SIGNATURE: &[u8] = &[0x38, 0x42, 0x50, 0x53];
const BLENDER_SIGNATURE: &[u8] = &[0x42, 0x4C, 0x45, 0x4E, 0x44, 0x45, 0x52];
const TRUE_TYPE_FONT_SIGNATURE: &[u8] = &[0x00, 0x01, 0x00, 0x00, 0x00];
const OPEN_TYPE_FONT_SIGNATURE: &[u8] = &[0x4F, 0x54, 0x54, 0x4F];
const MODULEFILE_FOR_ENVIRONMENT_MODULES_SIGNATURE: &[u8] =
    &[0x23, 0x25, 0x4D, 0x6F, 0x64, 0x75, 0x6C, 0x65];
const WINDOW_IMAGING_FORMAT_SIGNATURE: &[u8] = &[
    0x4D, 0x53, 0x57, 0x49, 0x4D, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00, 0x00,
];
const SLOB_SIGNATURE: &[u8] = &[0x21, 0x2D, 0x31, 0x53, 0x4C, 0x4F, 0x42, 0x1F];
const SERIALIZED_JAVA_DATA_SIGNATURE: &[u8] = &[0xAC, 0xED];
const CREATIVE_VOICE_FILE_SIGNATURE: &[u8] = &[
    0x43, 0x72, 0x65, 0x61, 0x74, 0x69, 0x69, 0x76, 0x65, 0x20, 0x56, 0x6F, 0x69, 0x63, 0x65, 0x20,
    0x46, 0x69, 0x6C, 0x65, 0x1A, 0x1A, 0x00,
];
const AU_AUDIO_FILE_FORMAT_SIGNATURE: &[u8] = &[0x2E, 0x73, 0x6E, 0x64];
const OPENGL_IRIS_PERFORMER_SIGNATURE: &[u8] = &[0xDB, 0x0A, 0xCE, 0x00];
const NOODLESOFT_HAZEL_SIGNATURE: &[u8] = &[0x48, 0x5A, 0x4C, 0x52, 0x00, 0x00, 0x00, 0x18];
const VB_SCRIPT_ENCODED_SIGNATURE: &[u8] = &[0x23, 0x40, 0x7E, 0x5E];
const APPLE_ICON_IMAGE_SIGNATURE: &[u8] = &[0x69, 0x63, 0x6E, 0x73];
const GIF_SIGNATURE: &[&[u8]] = &[
    &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61],
    &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61],
];
const JPEG_2000_SIGNATURE: &[&[u8]] = &[
    &[
        0x00, 0x00, 0x00, 0xC, 0xA, 0x6A, 0x50, 0x20, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
    ],
    &[0xFF, 0x4F, 0xFF, 0x51],
];
const PDF_SIGNATURE: &[u8] = &[0x25, 0x50, 0x44, 0x46, 0x2D];
const APPLE_DISK_IMAGE_SIGNATURE: &[u8] = &[0x6B, 0x6F, 0x6C, 0x79];
const CABINET_SIGNATURE: &[u8] = &[0x4D, 0x53, 0x43, 0x46];
const MATROSKA_MEDIA_CONTAINER_SIGNATURE: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3];
const RICHTEXT_FORMAT_SIGNATURE: &[u8] = &[0x7B, 0x5C, 0x72, 0x74, 0x66, 0x31];
const PHOTOCAP_TEMPLATE_SIGNATURE: &[u8] = &[0x78, 0x56, 0x34];
const ACE_COMPRESSED_SIGNATURE: &[u8] = &[0x2A, 0x2A, 0x41, 0x43, 0x45, 0x2A, 0x2A];
const FLASH_VIDEO_SIGNATURE: &[u8] = &[0x46, 0x4C, 0x56];
const VMDK_FILE_SIGNATURE: &[u8] = &[0x4B, 0x44, 0x4D];
const GOOGLE_CHROME_EXTENSION_SIGNATURE: &[u8] = &[0x43, 0x72, 0x32, 0x34];

pub struct Magic {
    pub signatures: &'static [&'static [u8]],
    pub offsets: &'static [usize],
    pub max_bytes_read: usize,
    pub kind: FileKind,
    pub rules: MatchRules,
}

impl Magic {
    #[must_use]
    #[inline]
    pub fn matches(&self, bytes: &[u8]) -> bool {
        match &self.rules {
            MatchRules::Default => self.signatures.iter().any(|&signature| {
                self.offsets.iter().any(|&offset| {
                    let offset_end = offset + signature.len();

                    bytes.len() >= offset_end && &bytes[offset..offset_end] == signature
                })
            }),
            MatchRules::WithFn(func) => func(bytes),
        }
    }
}

pub static SIGNATURE_KIND: &[Magic] = &[
    Magic {
        signatures: &[PNG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Png,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[CLASS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Class,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[JPG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Jpg,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[GZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Gzip,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[BZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Bzip,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[PKG_ZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::PkgZip,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[BITMAP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Bitmap,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[MS_DOS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::MSDOS,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[TAR_SIGNATURE],
        offsets: TAR_OFFSETS,
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Tar,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[MP3_SIGNATURE_1, MP3_SIGNATURE_2, MP3_SIGNATURE_3],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: TAR_MAX_BYTES_READ,
        kind: FileKind::MP3,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[ISO_SIGNATURE],
        offsets: ISO_OFFSETS,
        max_bytes_read: ISO_MAX_BYTES_READ,
        kind: FileKind::ISO,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[RPM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::RPM,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[SQLITE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::SQLite,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[XML_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::XML,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[ICO_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ICO,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[WASM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::WASM,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[DEB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Deb,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[SCRIPT_EXECUTE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ScriptExecute,
        // `#!/` alone would claim every file beginning with those two bytes,
        // including the `#!AMR` audio header. Require a path separator.
        rules: MatchRules::WithFn(is_shebang),
    },
    Magic {
        signatures: RAR_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::RAR,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[ELF_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ELF,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[OGG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OGG,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[_8BPS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::_8BPS,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[BLENDER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::BLENDER,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[TRUE_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::TrueTypeFont,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[OPEN_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OpenTypeFont,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[MODULEFILE_FOR_ENVIRONMENT_MODULES_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ModuleForEvenvironmentModules,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[WINDOW_IMAGING_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::WindowImagingFormat,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[SLOB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Slob,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[SERIALIZED_JAVA_DATA_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::SerializedJavaData,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[CREATIVE_VOICE_FILE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::CreativeVoiceFile,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[AU_AUDIO_FILE_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::AuAudioFileFormat,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[OPENGL_IRIS_PERFORMER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OpenGLIrisPerformer,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[NOODLESOFT_HAZEL_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::NoodlesoftHazel,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[VB_SCRIPT_ENCODED_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::VBScriptEncoded,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[],
        offsets: &[],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::WEBP,
        rules: MatchRules::WithFn(is_webp),
    },
    Magic {
        signatures: &[APPLE_ICON_IMAGE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::AppleIconImage,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: GIF_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::GIF,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: JPEG_2000_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::JPEG2000,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[PDF_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::PDF,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[APPLE_DISK_IMAGE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::AppleDiskImage,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[CABINET_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Cabinet,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[MATROSKA_MEDIA_CONTAINER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::MatroskaMediaContainer,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[RICHTEXT_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::RichTextFormat,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[PHOTOCAP_TEMPLATE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::PhotoCapTemplate,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[ACE_COMPRESSED_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::AceCompressed,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[FLASH_VIDEO_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::FlashVideo,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[VMDK_FILE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Vmdk,
        rules: MatchRules::Default,
    },
    Magic {
        signatures: &[GOOGLE_CHROME_EXTENSION_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::GoogleChromeExtension,
        rules: MatchRules::Default,
    },
    // -----------------------------------------------------------------------
    // Extended formats.
    //
    // These are appended rather than interleaved. `match_types` returns the
    // first match, so appending guarantees that no rule from the original 48
    // can ever be shadowed. `entry!` keeps each one to a single line.
    // -----------------------------------------------------------------------
    entry!(SevenZip, [SEVEN_ZIP], [0]),
    entry!(Xz, [XZ], [0]),
    entry!(Lz4, [LZ4], [0]),
    entry!(Zstd, [ZSTD], [0]),
    // `Ktx2` must precede `Ktx`: the KTX1 magic is a byte-for-byte prefix of
    // the KTX2 magic, and `match_types` stops at the first match.
    entry!(Ktx2, [KTX2], [0]),
    entry!(Ktx, [KTX], [0]),
    entry!(Lzh, [LZH], [2]),
    entry!(Cpio, [CPIO], [0]),
    entry!(Arj, [ARJ], [0]),
    entry!(Stuffit, [STUFFIT], [0]),
    entry!(StuffitSit, [STUFFIT_SIT], [0]),
    entry!(Par2, [PAR2], [0]),
    // Images
    entry!(
        Tiff,
        [TIFF_LE, TIFF_BE, TIFF_BIGTIFF_LE, TIFF_BIGTIFF_BE],
        [0]
    ),
    entry!(Pcx, [PCX, PCX_V2, PCX_V3, PCX_V5], [0]),
    entry!(Dds, [DDS], [0]),
    entry!(OpenExr, [OPENEXR], [0]),
    entry!(Radiance, [RADIANCE], [0]),
    entry!(JpegXl, [JPEG_XL_CODESTREAM, JPEG_XL_CONTAINER], [0]),
    entry!(Cursor, [CURSOR], [0]),
    entry!(GimpXcf, [GIMP_XCF], [0]),
    entry!(Fits, [FITS], [0]),
    // Audio and video
    entry!(Midi, [MIDI], [0]),
    entry!(Aiff, [AIFF], [0]),
    entry!(Flac, [FLAC], [0]),
    entry!(WavPack, [WAVPACK], [0]),
    entry!(CoreAudio, [CORE_AUDIO], [0]),
    // No longer shadowed: the shebang rule now requires a path separator, so
    // `#!AMR` falls through to here.
    entry!(Amr, [AMR_NARROW, AMR_WIDE], [0]),
    entry!(MonkeyAudio, [MONKEY_AUDIO], [0]),
    entry!(IsoMedia, [ISO_MEDIA], [4]),
    entry!(Swf, [SWF_UNCOMPRESSED, SWF_ZLIB, SWF_LZMA], [0]),
    entry!(Asf, [ASF], [0]),
    entry!(MpegProgramStream, [MPEG_PROGRAM_STREAM], [0]),
    // Documents
    entry!(PostScript, [POSTSCRIPT], [0]),
    entry!(Djvu, [DJVU], [0]),
    entry!(Mobipocket, [MOBIPOCKET], [60]),
    entry!(Chm, [CHM], [0]),
    entry!(OleCompoundFile, [OLE_COMPOUND_FILE], [0]),
    // Executables and byte code
    entry!(
        MachO,
        [MACH_O_32, MACH_O_64, MACH_O_32_SWAPPED, MACH_O_64_SWAPPED],
        [0]
    ),
    entry!(Dalvik, [DALVIK], [0]),
    entry!(Lua, [LUA], [0]),
    entry!(WindowsShortcut, [WINDOWS_SHORTCUT], [0]),
    // Fonts
    entry!(Woff, [WOFF], [0]),
    entry!(Woff2, [WOFF2], [0]),
    entry!(FontCollection, [FONT_COLLECTION], [0]),
    // Data, columnar and machine learning
    entry!(Numpy, [NUMPY], [0]),
    entry!(Hdf5, [HDF5], [0]),
    entry!(Matlab, [MATLAB], [0]),
    entry!(Parquet, [PARQUET], [0]),
    entry!(Orc, [APACHE_ORC], [0]),
    entry!(Avro, [AVRO], [0]),
    entry!(BinaryPlist, [BINARY_PLIST], [0]),
    entry!(Pickle, [PICKLE_V2, PICKLE_V3, PICKLE_V4, PICKLE_V5], [0]),
    entry!(Gguf, [GGUF], [0]),
    entry!(RData, [R_DATA_V2, R_DATA_V3], [0]),
    // 3D assets and game data
    entry!(GltfBinary, [GLTF_BINARY], [0]),
    entry!(FbxBinary, [FBX_BINARY], [0]),
    entry!(Ply, [PLY_LF, PLY_CRLF], [0]),
    entry!(DoomWad, [DOOM_WAD], [0]),
    // Disk images
    // `Qcow2` must precede `Qcow`: `51 46 49` is a prefix of `51 46 49 FB`.
    entry!(Qcow2, [QCOW2], [0]),
    entry!(Qcow, [QCOW], [0]),
    entry!(VirtualBoxVdi, [VIRTUALBOX_VDI], [0]),
    entry!(VirtualHd, [VIRTUAL_HD], [0]),
    // Network capture and transfer
    entry!(Pcap, [PCAP_LE, PCAP_BE, PCAP_NS_LE, PCAP_NS_BE], [0]),
    entry!(PcapNg, [PCAP_NG], [0]),
    entry!(BitTorrent, [BITTORRENT], [0]),
    // `Zlib` is deliberately last: `78 9C` appears in any deflate stream, so it
    // must only win when no structural signature matched first.
    entry!(Zlib, [ZLIB, ZLIB_LOW, ZLIB_BEST, ZLIB_DEFAULT_COMP], [0]),
];

