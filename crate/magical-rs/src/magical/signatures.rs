use crate::magical::bytes_read::{
    DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS, TAR_MAX_BYTES_READ,
    TAR_OFFSETS,
};
use crate::magical::magic::FileKind;

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const GZIP_SIGNATURE: &[u8] = &[0x1F, 0x8B];
const BZIP_SIGNATURE: &[u8] = &[0x42, 0x5A];
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

pub struct Magic {
    pub signatures: &'static [&'static [u8]],
    pub offsets: &'static [usize],
    pub max_bytes_read: usize,
    pub kind: FileKind,
}

pub static SIGNATURE_KIND: &[Magic] = &[
    Magic {
        signatures: &[PNG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Png,
    },
    Magic {
        signatures: &[CLASS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Class,
    },
    Magic {
        signatures: &[JPG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Jpg,
    },
    Magic {
        signatures: &[GZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Gzip,
    },
    Magic {
        signatures: &[BZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Bzip,
    },
    Magic {
        signatures: &[PKG_ZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::PkgZip,
    },
    Magic {
        signatures: &[BITMAP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Bitmap,
    },
    Magic {
        signatures: &[MS_DOS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::MSDOS,
    },
    Magic {
        signatures: &[TAR_SIGNATURE],
        offsets: TAR_OFFSETS,
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Tar,
    },
    Magic {
        signatures: &[MP3_SIGNATURE_1, MP3_SIGNATURE_2, MP3_SIGNATURE_3],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: TAR_MAX_BYTES_READ,
        kind: FileKind::MP3,
    },
    Magic {
        signatures: &[ISO_SIGNATURE],
        offsets: ISO_OFFSETS,
        max_bytes_read: ISO_MAX_BYTES_READ,
        kind: FileKind::ISO,
    },
    Magic {
        signatures: &[RPM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::RPM,
    },
    Magic {
        signatures: &[SQLITE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::SQLite,
    },
    Magic {
        signatures: &[XML_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::XML,
    },
    Magic {
        signatures: &[ICO_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ICO,
    },
    Magic {
        signatures: &[WASM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::WASM,
    },
    Magic {
        signatures: &[DEB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Deb,
    },
    Magic {
        signatures: &[SCRIPT_EXECUTE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ScriptExecute,
    },
    Magic {
        signatures: RAR_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::RAR,
    },
    Magic {
        signatures: &[ELF_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ELF,
    },
    Magic {
        signatures: &[OGG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OGG,
    },
    Magic {
        signatures: &[_8BPS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::_8BPS,
    },
    Magic {
        signatures: &[BLENDER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::BLENDER,
    },
    Magic {
        signatures: &[TRUE_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::TrueTypeFont,
    },
    Magic {
        signatures: &[OPEN_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OpenTypeFont,
    },
    Magic {
        signatures: &[MODULEFILE_FOR_ENVIRONMENT_MODULES_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::ModuleForEvenvironmentModules,
    },
    Magic {
        signatures: &[WINDOW_IMAGING_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::WindowImagingFormat,
    },
    Magic {
        signatures: &[SLOB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::Slob,
    },
    Magic {
        signatures: &[SERIALIZED_JAVA_DATA_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::SerializedJavaData,
    },
    Magic {
        signatures: &[CREATIVE_VOICE_FILE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::CreativeVoiceFile,
    },
    Magic {
        signatures: &[AU_AUDIO_FILE_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::AuAudioFileFormat,
    },
    Magic {
        signatures: &[OPENGL_IRIS_PERFORMER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: FileKind::OpenGLIrisPerformer,
    },
];
