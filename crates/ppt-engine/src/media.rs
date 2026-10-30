//! DrawingML Media Engine: Embedded Audio & Video nodes, OPC media extractor,
//! relationships parsing, and media playback metadata representation.

use ooxml_opc::relationships::{Relationships, TargetMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Media types supported in PresentationML DrawingML nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    Video,
    Audio,
}

/// Standard DrawingML media file format / mime kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaFormat {
    Mp4,
    WebM,
    Mp3,
    Wav,
    Ogg,
    Unknown(String),
}

impl MediaFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp4" | "m4v" => Self::Mp4,
            "webm" => Self::WebM,
            "mp3" => Self::Mp3,
            "wav" => Self::Wav,
            "ogg" | "oga" | "ogv" => Self::Ogg,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::WebM => "video/webm",
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Ogg => "audio/ogg",
            Self::Unknown(_) => "application/octet-stream",
        }
    }
}

/// Bounding box layout coordinates for media canvas overlay (pixels or EMUs).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MediaBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64, // degrees
}

impl Default for MediaBounds {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            width: 480.0,
            height: 270.0,
            rotation: 0.0,
        }
    }
}

/// Embedded Media Node definition extracted from a slide canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaNode {
    pub id: String,
    pub name: String,
    pub media_type: MediaType,
    pub media_format: MediaFormat,
    pub r_id: String,                // relationship Id (e.g. "rId2")
    pub part_path: String,           // e.g. "ppt/media/media1.mp4"
    pub poster_r_id: Option<String>, // Relationship ID for poster frame picture
    pub poster_part_path: Option<String>,
    pub bounds: MediaBounds,
    pub auto_play: bool,
    pub loop_playback: bool,
    pub volume: f64, // 0.0 to 1.0
    pub muted: bool,
    pub show_controls: bool,
}

impl MediaNode {
    pub fn new_video(
        id: impl Into<String>,
        name: impl Into<String>,
        r_id: impl Into<String>,
        part_path: impl Into<String>,
        format: MediaFormat,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            media_type: MediaType::Video,
            media_format: format,
            r_id: r_id.into(),
            part_path: part_path.into(),
            poster_r_id: None,
            poster_part_path: None,
            bounds: MediaBounds::default(),
            auto_play: false,
            loop_playback: false,
            volume: 1.0,
            muted: false,
            show_controls: true,
        }
    }

    pub fn new_audio(
        id: impl Into<String>,
        name: impl Into<String>,
        r_id: impl Into<String>,
        part_path: impl Into<String>,
        format: MediaFormat,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            media_type: MediaType::Audio,
            media_format: format,
            r_id: r_id.into(),
            part_path: part_path.into(),
            poster_r_id: None,
            poster_part_path: None,
            bounds: MediaBounds {
                width: 200.0,
                height: 48.0,
                ..Default::default()
            },
            auto_play: false,
            loop_playback: false,
            volume: 1.0,
            muted: false,
            show_controls: true,
        }
    }
}

/// Extracted media binary asset from the package.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaAsset {
    pub part_path: String,
    pub media_type: MediaType,
    pub format: MediaFormat,
    pub data: Vec<u8>,
}

pub const VIDEO_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/video";
pub const AUDIO_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/audio";
pub const MEDIA_RELATIONSHIP_TYPE: &str =
    "http://schemas.microsoft.com/office/2007/relationships/media";

/// Media Engine inspecting slide XML and relationship parts to extract DrawingML media nodes.
pub struct MediaEngine;

impl MediaEngine {
    /// Extracts media relationships from a slide's `Relationships` table.
    pub fn extract_media_relationships(
        rels: &Relationships,
    ) -> HashMap<String, (MediaType, String)> {
        let mut map = HashMap::new();
        for (id, rel) in &rels.relationships {
            let rel_type = &rel.rel_type;
            let is_video = rel_type == VIDEO_RELATIONSHIP_TYPE
                || (rel_type == MEDIA_RELATIONSHIP_TYPE && Self::is_video_target(&rel.target));
            let is_audio = rel_type == AUDIO_RELATIONSHIP_TYPE
                || (rel_type == MEDIA_RELATIONSHIP_TYPE && Self::is_audio_target(&rel.target));

            if is_video {
                map.insert(id.clone(), (MediaType::Video, rel.target.clone()));
            } else if is_audio {
                map.insert(id.clone(), (MediaType::Audio, rel.target.clone()));
            }
        }
        map
    }

    /// Determines if a target path points to a video file.
    pub fn is_video_target(target: &str) -> bool {
        let lower = target.to_ascii_lowercase();
        lower.ends_with(".mp4")
            || lower.ends_with(".m4v")
            || lower.ends_with(".webm")
            || lower.ends_with(".avi")
            || lower.ends_with(".wmv")
            || lower.ends_with(".mov")
    }

    /// Determines if a target path points to an audio file.
    pub fn is_audio_target(target: &str) -> bool {
        let lower = target.to_ascii_lowercase();
        lower.ends_with(".mp3")
            || lower.ends_with(".wav")
            || lower.ends_with(".wma")
            || lower.ends_with(".m4a")
            || lower.ends_with(".ogg")
    }

    /// Generates DrawingML XML for an embedded video node.
    pub fn serialize_video_shape_xml(node: &MediaNode) -> String {
        let emu_x = (node.bounds.x * 9525.0) as i64;
        let emu_y = (node.bounds.y * 9525.0) as i64;
        let emu_cx = (node.bounds.width * 9525.0) as i64;
        let emu_cy = (node.bounds.height * 9525.0) as i64;

        format!(
            r#"<p:pic xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:nvPicPr>
    <p:cNvPr id="{}" name="{}">
      <a:hlinkClick r:action="ppaction://media?action=play"/>
    </p:cNvPr>
    <p:cNvPicPr>
      <a:picLocks noChangeAspect="1"/>
    </p:cNvPicPr>
    <p:nvPr>
      <p:videoFile r:link="{}"/>
    </p:nvPr>
  </p:nvPicPr>
  <p:blipFill>
    <a:blip r:embed="{}"/>
    <a:stretch><a:fillRect/></a:stretch>
  </p:blipFill>
  <p:spPr>
    <a:xfrm>
      <a:off x="{}" y="{}"/>
      <a:ext cx="{}" cy="{}"/>
    </a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
  </p:spPr>
</p:pic>"#,
            node.id,
            node.name,
            node.r_id,
            node.poster_r_id.as_deref().unwrap_or(&node.r_id),
            emu_x,
            emu_y,
            emu_cx,
            emu_cy
        )
    }

    /// Generates DrawingML XML for an embedded audio node.
    pub fn serialize_audio_shape_xml(node: &MediaNode) -> String {
        let emu_x = (node.bounds.x * 9525.0) as i64;
        let emu_y = (node.bounds.y * 9525.0) as i64;

        format!(
            r#"<p:pic xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:nvPicPr>
    <p:cNvPr id="{}" name="{}">
      <a:hlinkClick r:action="ppaction://media?action=play"/>
    </p:cNvPr>
    <p:cNvPicPr/>
    <p:nvPr>
      <p:audioFile r:link="{}"/>
    </p:nvPr>
  </p:nvPicPr>
  <p:blipFill>
    <a:blip r:embed="{}"/>
    <a:stretch><a:fillRect/></a:stretch>
  </p:blipFill>
  <p:spPr>
    <a:xfrm>
      <a:off x="{}" y="{}"/>
      <a:ext cx="1905000" cy="457200"/>
    </a:xfrm>
    <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
  </p:spPr>
</p:pic>"#,
            node.id,
            node.name,
            node.r_id,
            node.poster_r_id.as_deref().unwrap_or(&node.r_id),
            emu_x,
            emu_y
        )
    }

    /// Registers a media file into a slide's relationships, returning relationship ID.
    pub fn register_media_relationship(
        rels: &mut Relationships,
        media_type: MediaType,
        target_path: &str,
    ) -> String {
        let rel_type = match media_type {
            MediaType::Video => VIDEO_RELATIONSHIP_TYPE,
            MediaType::Audio => AUDIO_RELATIONSHIP_TYPE,
        };
        let r_id = format!("rIdMedia{}", rels.relationships.len() + 1);
        rels.add(&r_id, rel_type, target_path, TargetMode::Internal);
        r_id
    }
}
