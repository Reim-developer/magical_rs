use crate::magical::bytes_read::{
    DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS, TAR_MAX_BYTES_READ,
    TAR_OFFSETS,
};
use crate::magical::ext_fn::webp::is_webp;
use crate::magical::magic::FileKind;
use crate::magical::match_rules::MatchRules;

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
        rules: MatchRules::Default,
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
];
