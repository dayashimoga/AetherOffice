//! Subphase 19.1.1 Integration Tests: DrawingML Embedded Audio/Video Media Engine.

use ooxml_opc::relationships::{Relationships, TargetMode};
use ooxml_opc::unparsed::UnparsedPart;
use ppt_engine::media::{
    MediaBounds, MediaEngine, MediaFormat, MediaNode, MediaType, AUDIO_RELATIONSHIP_TYPE,
    MEDIA_RELATIONSHIP_TYPE, VIDEO_RELATIONSHIP_TYPE,
};
use ppt_engine::package::PresentationPackage;

#[test]
fn test_01_media_format_and_mime_types() {
    assert_eq!(MediaFormat::from_extension("mp4"), MediaFormat::Mp4);
    assert_eq!(MediaFormat::from_extension("MP4"), MediaFormat::Mp4);
    assert_eq!(MediaFormat::from_extension("m4v"), MediaFormat::Mp4);
    assert_eq!(MediaFormat::from_extension("webm"), MediaFormat::WebM);
    assert_eq!(MediaFormat::from_extension("mp3"), MediaFormat::Mp3);
    assert_eq!(MediaFormat::from_extension("wav"), MediaFormat::Wav);
    assert_eq!(MediaFormat::from_extension("ogg"), MediaFormat::Ogg);

    assert_eq!(MediaFormat::Mp4.mime_type(), "video/mp4");
    assert_eq!(MediaFormat::WebM.mime_type(), "video/webm");
    assert_eq!(MediaFormat::Mp3.mime_type(), "audio/mpeg");
    assert_eq!(MediaFormat::Wav.mime_type(), "audio/wav");
    assert_eq!(MediaFormat::Ogg.mime_type(), "audio/ogg");

    let unknown = MediaFormat::from_extension("flac");
    assert_eq!(unknown, MediaFormat::Unknown("flac".to_string()));
    assert_eq!(unknown.mime_type(), "application/octet-stream");
}

#[test]
fn test_02_media_node_video_and_audio_defaults() {
    let video = MediaNode::new_video(
        "vid1",
        "Sample Video",
        "rIdMedia1",
        "ppt/media/sample.mp4",
        MediaFormat::Mp4,
    );
    assert_eq!(video.id, "vid1");
    assert_eq!(video.name, "Sample Video");
    assert_eq!(video.media_type, MediaType::Video);
    assert_eq!(video.media_format, MediaFormat::Mp4);
    assert_eq!(video.r_id, "rIdMedia1");
    assert_eq!(video.part_path, "ppt/media/sample.mp4");
    assert_eq!(video.volume, 1.0);
    assert!(!video.muted);
    assert!(!video.auto_play);
    assert!(!video.loop_playback);
    assert_eq!(video.bounds.width, 480.0);
    assert_eq!(video.bounds.height, 270.0);

    let audio = MediaNode::new_audio(
        "aud1",
        "Podcast Audio",
        "rIdMedia2",
        "ppt/media/sample.mp3",
        MediaFormat::Mp3,
    );
    assert_eq!(audio.id, "aud1");
    assert_eq!(audio.name, "Podcast Audio");
    assert_eq!(audio.media_type, MediaType::Audio);
    assert_eq!(audio.bounds.width, 200.0);
    assert_eq!(audio.bounds.height, 48.0);
}

#[test]
fn test_03_media_relationship_extraction() {
    let mut rels = Relationships::new();
    rels.add(
        "rId1",
        VIDEO_RELATIONSHIP_TYPE,
        "media/video1.mp4",
        TargetMode::Internal,
    );
    rels.add(
        "rId2",
        AUDIO_RELATIONSHIP_TYPE,
        "media/audio1.mp3",
        TargetMode::Internal,
    );
    // Generic media relationship with video extension
    rels.add(
        "rId3",
        MEDIA_RELATIONSHIP_TYPE,
        "media/clip.webm",
        TargetMode::Internal,
    );
    // Generic media relationship with audio extension
    rels.add(
        "rId4",
        MEDIA_RELATIONSHIP_TYPE,
        "media/song.wav",
        TargetMode::Internal,
    );
    // Non-media relationship
    rels.add(
        "rId5",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout",
        "../slideLayouts/slideLayout1.xml",
        TargetMode::Internal,
    );

    let media_map = MediaEngine::extract_media_relationships(&rels);
    assert_eq!(media_map.len(), 4);
    assert_eq!(
        media_map.get("rId1"),
        Some(&(MediaType::Video, "media/video1.mp4".to_string()))
    );
    assert_eq!(
        media_map.get("rId2"),
        Some(&(MediaType::Audio, "media/audio1.mp3".to_string()))
    );
    assert_eq!(
        media_map.get("rId3"),
        Some(&(MediaType::Video, "media/clip.webm".to_string()))
    );
    assert_eq!(
        media_map.get("rId4"),
        Some(&(MediaType::Audio, "media/song.wav".to_string()))
    );
    assert!(!media_map.contains_key("rId5"));
}

#[test]
fn test_04_serialize_video_shape_xml() {
    let mut video = MediaNode::new_video(
        "v_100",
        "Corporate Intro",
        "rIdVideo1",
        "ppt/media/intro.mp4",
        MediaFormat::Mp4,
    );
    video.bounds = MediaBounds {
        x: 100.0,
        y: 150.0,
        width: 640.0,
        height: 360.0,
        rotation: 0.0,
    };
    video.poster_r_id = Some("rIdPoster1".to_string());

    let xml = MediaEngine::serialize_video_shape_xml(&video);
    assert!(xml.contains(r#"<p:cNvPr id="v_100" name="Corporate Intro">"#));
    assert!(xml.contains(r#"<p:videoFile r:link="rIdVideo1"/>"#));
    assert!(xml.contains(r#"<a:blip r:embed="rIdPoster1"/>"#));
    assert!(xml.contains(r#"<a:off x="952500" y="1428750"/>"#));
    assert!(xml.contains(r#"<a:ext cx="6096000" cy="3429000"/>"#));
}

#[test]
fn test_05_serialize_audio_shape_xml() {
    let mut audio = MediaNode::new_audio(
        "a_200",
        "Background Music",
        "rIdAudio1",
        "ppt/media/music.mp3",
        MediaFormat::Mp3,
    );
    audio.bounds.x = 50.0;
    audio.bounds.y = 80.0;

    let xml = MediaEngine::serialize_audio_shape_xml(&audio);
    assert!(xml.contains(r#"<p:cNvPr id="a_200" name="Background Music">"#));
    assert!(xml.contains(r#"<p:audioFile r:link="rIdAudio1"/>"#));
    assert!(xml.contains(r#"<a:blip r:embed="rIdAudio1"/>"#));
    assert!(xml.contains(r#"<a:off x="476250" y="762000"/>"#));
}

#[test]
fn test_06_register_media_relationship() {
    let mut rels = Relationships::new();
    let r_id_vid = MediaEngine::register_media_relationship(
        &mut rels,
        MediaType::Video,
        "ppt/media/video1.mp4",
    );
    let r_id_aud = MediaEngine::register_media_relationship(
        &mut rels,
        MediaType::Audio,
        "ppt/media/audio1.mp3",
    );

    assert_eq!(r_id_vid, "rIdMedia1");
    assert_eq!(r_id_aud, "rIdMedia2");

    let extracted = MediaEngine::extract_media_relationships(&rels);
    assert_eq!(
        extracted.get("rIdMedia1"),
        Some(&(MediaType::Video, "ppt/media/video1.mp4".to_string()))
    );
    assert_eq!(
        extracted.get("rIdMedia2"),
        Some(&(MediaType::Audio, "ppt/media/audio1.mp3".to_string()))
    );
}

#[test]
fn test_07_target_media_type_detection() {
    assert!(MediaEngine::is_video_target("clip.MP4"));
    assert!(MediaEngine::is_video_target("path/to/movie.webm"));
    assert!(MediaEngine::is_video_target("video.avi"));
    assert!(MediaEngine::is_video_target("demo.wmv"));
    assert!(MediaEngine::is_video_target("teaser.mov"));
    assert!(!MediaEngine::is_video_target("sound.mp3"));

    assert!(MediaEngine::is_audio_target("narration.mp3"));
    assert!(MediaEngine::is_audio_target("sound_fx.wav"));
    assert!(MediaEngine::is_audio_target("bgm.ogg"));
    assert!(MediaEngine::is_audio_target("track.m4a"));
    assert!(!MediaEngine::is_audio_target("clip.mp4"));
}

#[test]
fn test_08_extract_slide_media_from_package() {
    let mut pkg = PresentationPackage::create_empty();
    let slide_part = "ppt/slides/slide1.xml";

    let mut rels = Relationships::new();
    rels.add(
        "rIdMedia1",
        VIDEO_RELATIONSHIP_TYPE,
        "../media/vid.mp4",
        TargetMode::Internal,
    );
    rels.add(
        "rIdMedia2",
        AUDIO_RELATIONSHIP_TYPE,
        "../media/aud.wav",
        TargetMode::Internal,
    );
    pkg.part_relationships.insert(slide_part.to_string(), rels);

    pkg.unparsed_store.insert(UnparsedPart::new(
        "ppt/media/vid.mp4",
        vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p'],
        Some("video/mp4".to_string()),
        None,
    ));
    pkg.unparsed_store.insert(UnparsedPart::new(
        "ppt/media/aud.wav",
        vec![b'R', b'I', b'F', b'F', 0x24, 0x00, 0x00, 0x00],
        Some("audio/wav".to_string()),
        None,
    ));

    let assets = pkg.extract_slide_media(slide_part);
    assert_eq!(assets.len(), 2);

    let vid = assets
        .iter()
        .find(|a| a.media_type == MediaType::Video)
        .expect("video asset should be present in test");
    assert_eq!(vid.part_path, "ppt/media/vid.mp4");
    assert_eq!(vid.format, MediaFormat::Mp4);
    assert_eq!(vid.data.len(), 8);

    let aud = assets
        .iter()
        .find(|a| a.media_type == MediaType::Audio)
        .expect("audio asset should be present in test");
    assert_eq!(aud.part_path, "ppt/media/aud.wav");
    assert_eq!(aud.format, MediaFormat::Wav);
    assert_eq!(aud.data.len(), 8);
}

#[test]
fn test_09_serde_json_roundtrip() {
    let node = MediaNode::new_video(
        "v1",
        "Keynote Video",
        "rId3",
        "ppt/media/keynote.mp4",
        MediaFormat::Mp4,
    );

    let json = serde_json::to_string(&node).expect("Serialization must succeed in test");
    let deserialized: MediaNode =
        serde_json::from_str(&json).expect("Deserialization must succeed in test");

    assert_eq!(node, deserialized);
}

#[test]
fn test_10_poster_fallback_when_none() {
    let video = MediaNode::new_video(
        "v2",
        "Teaser",
        "rIdTeaser",
        "ppt/media/t.mp4",
        MediaFormat::Mp4,
    );
    assert!(video.poster_r_id.is_none());
    let xml = MediaEngine::serialize_video_shape_xml(&video);
    // When poster_r_id is None, it falls back to r_id ("rIdTeaser")
    assert!(xml.contains(r#"<a:blip r:embed="rIdTeaser"/>"#));
}
