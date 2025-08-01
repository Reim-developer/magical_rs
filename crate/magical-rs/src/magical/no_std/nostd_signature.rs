use crate::magical::ext_fn::webp::is_webp;
use crate::magical::no_std::no_std_bytes_read::{
    DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS, TAR_MAX_BYTES_READ,
    TAR_OFFSETS,
};
use crate::magical::no_std::no_std_match_rules::NoStdMatchRules;
use crate::magical::no_std::nostd_magic::NoStdFileKind;

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

pub struct NoStdMagic {
    pub signatures: &'static [&'static [u8]],
    pub offsets: &'static [usize],
    pub max_bytes_read: usize,
    pub kind: NoStdFileKind,
    pub rules: NoStdMatchRules,
}

impl NoStdMagic {
    #[must_use]
    #[inline]
    pub fn no_std_matches(&self, bytes: &[u8]) -> bool {
        match &self.rules {
            NoStdMatchRules::Default => self.signatures.iter().any(|&signature| {
                self.offsets.iter().any(|&offset| {
                    let offset_end = offset + signature.len();

                    bytes.len() >= offset_end && &bytes[offset..offset_end] == signature
                })
            }),
            NoStdMatchRules::WithFn(func) => func(bytes),
        }
    }
}

pub static SIGNATURE_KIND: &[NoStdMagic] = &[
    NoStdMagic {
        signatures: &[PNG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Png,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[CLASS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Class,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[JPG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Jpg,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[GZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Gzip,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[BZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Bzip,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[PKG_ZIP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::PkgZip,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[BITMAP_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Bitmap,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[MS_DOS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::MSDOS,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[TAR_SIGNATURE],
        offsets: TAR_OFFSETS,
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Tar,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[MP3_SIGNATURE_1, MP3_SIGNATURE_2, MP3_SIGNATURE_3],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: TAR_MAX_BYTES_READ,
        kind: NoStdFileKind::MP3,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[ISO_SIGNATURE],
        offsets: ISO_OFFSETS,
        max_bytes_read: ISO_MAX_BYTES_READ,
        kind: NoStdFileKind::ISO,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[RPM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::RPM,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[SQLITE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::SQLite,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[XML_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::XML,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[ICO_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::ICO,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[WASM_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::WASM,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[DEB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Deb,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[SCRIPT_EXECUTE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::ScriptExecute,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: RAR_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::RAR,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[ELF_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::ELF,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[OGG_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::OGG,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[_8BPS_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::_8BPS,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[BLENDER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::BLENDER,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[TRUE_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::TrueTypeFont,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[OPEN_TYPE_FONT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::OpenTypeFont,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[MODULEFILE_FOR_ENVIRONMENT_MODULES_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::ModuleForEvenvironmentModules,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[WINDOW_IMAGING_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::WindowImagingFormat,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[SLOB_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Slob,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[SERIALIZED_JAVA_DATA_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::SerializedJavaData,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[CREATIVE_VOICE_FILE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::CreativeVoiceFile,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[AU_AUDIO_FILE_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::AuAudioFileFormat,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[OPENGL_IRIS_PERFORMER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::OpenGLIrisPerformer,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[NOODLESOFT_HAZEL_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::NoodlesoftHazel,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[VB_SCRIPT_ENCODED_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::VBScriptEncoded,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[],
        offsets: &[],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::WEBP,
        rules: NoStdMatchRules::WithFn(is_webp),
    },
    NoStdMagic {
        signatures: &[APPLE_ICON_IMAGE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::AppleIconImage,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: GIF_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::GIF,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: JPEG_2000_SIGNATURE,
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::JPEG2000,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[PDF_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::PDF,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[APPLE_DISK_IMAGE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::AppleDiskImage,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[CABINET_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::Cabinet,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[MATROSKA_MEDIA_CONTAINER_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::MatroskaMediaContainer,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[RICHTEXT_FORMAT_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::RichTextFormat,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[PHOTOCAP_TEMPLATE_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::PhotoCapTemplate,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[ACE_COMPRESSED_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::AceCompressed,
        rules: NoStdMatchRules::Default,
    },
    NoStdMagic {
        signatures: &[FLASH_VIDEO_SIGNATURE],
        offsets: &[DEFAULT_OFFSET],
        max_bytes_read: DEFAULT_MAX_BYTES_READ,
        kind: NoStdFileKind::FlashVideo,
        rules: NoStdMatchRules::Default,
    },
];
