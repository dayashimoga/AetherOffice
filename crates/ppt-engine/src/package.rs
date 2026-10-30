#![forbid(unsafe_code)]
//! PresentationML Package (`PresentationPackage`)
//!
//! Implements ECMA-376 Part 2 Open Packaging Conventions (OPC) integration for PowerPoint `.pptx`:
//! - Reading and writing `ppt/presentation.xml`
//! - Discovers and parses `slideMasters`, `slideLayouts`, `slides`, `notesSlides`, and `theme`
//! - Resolves relationship graphs linking slides -> layouts -> masters -> theme
//! - Preserves unmodeled OPC parts in `UnparsedPartStore` for zero-loss round-trip fidelity

use ooxml_opc::content_types::ContentTypes;
use ooxml_opc::error::{EngineError, Result};
use ooxml_opc::package::{PackageReader, PackageWriter};
use ooxml_opc::relationships::{Relationships, TargetMode};
use ooxml_opc::unparsed::UnparsedPartStore;
use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Seek, Write};

use crate::cascade::{InheritanceResolver, ResolvedShape};
use crate::model::*;
use crate::parser::*;
use crate::serializer::*;

// =========================================================================
// Content Types & Relationship Constants
// =========================================================================

pub const PRESENTATION_MAIN_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml";
pub const SLIDE_MASTER_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml";
pub const SLIDE_LAYOUT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";
pub const SLIDE_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";
pub const NOTES_SLIDE_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml";
pub const THEME_TYPE: &str = "application/vnd.openxmlformats-officedocument.theme+xml";

pub const OFFICE_DOCUMENT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
pub const SLIDE_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
pub const SLIDE_MASTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
pub const SLIDE_LAYOUT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
pub const THEME_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

// =========================================================================
// PresentationPackage Struct
// =========================================================================

/// Represents a complete PowerPoint `.pptx` presentation package.
#[derive(Debug, Clone)]
pub struct PresentationPackage {
    pub presentation: Presentation,
    pub slide_masters: BTreeMap<String, SlideMaster>, // e.g. "ppt/slideMasters/slideMaster1.xml"
    pub slide_layouts: BTreeMap<String, SlideLayout>, // e.g. "ppt/slideLayouts/slideLayout1.xml"
    pub slides: Vec<(String, Slide)>,                 // ordered list of slides (part_name, Slide)
    pub notes_slides: BTreeMap<String, NotesSlide>,
    pub themes: BTreeMap<String, Theme>, // e.g. "ppt/theme/theme1.xml"

    // OPC Infrastructure
    pub unparsed_store: UnparsedPartStore,
    pub content_types: ContentTypes,
    pub package_relationships: Relationships,
    pub part_relationships: BTreeMap<String, Relationships>,
}

impl Default for PresentationPackage {
    fn default() -> Self {
        Self::create_empty()
    }
}

impl PresentationPackage {
    /// Creates an empty 16:9 widescreen presentation package with default master, layout, and theme.
    pub fn create_empty() -> Self {
        let mut content_types = ContentTypes::new();
        content_types.add_override("ppt/presentation.xml", PRESENTATION_MAIN_TYPE);
        content_types.add_override("ppt/slideMasters/slideMaster1.xml", SLIDE_MASTER_TYPE);
        content_types.add_override("ppt/slideLayouts/slideLayout1.xml", SLIDE_LAYOUT_TYPE);
        content_types.add_override("ppt/slides/slide1.xml", SLIDE_TYPE);
        content_types.add_override("ppt/theme/theme1.xml", THEME_TYPE);

        let mut package_relationships = Relationships::new();
        package_relationships.add(
            "rId1",
            OFFICE_DOCUMENT_REL,
            "ppt/presentation.xml",
            TargetMode::Internal,
        );

        let mut pres_rels = Relationships::new();
        pres_rels.add(
            "rId1",
            SLIDE_MASTER_REL,
            "slideMasters/slideMaster1.xml",
            TargetMode::Internal,
        );
        pres_rels.add("rId2", SLIDE_REL, "slides/slide1.xml", TargetMode::Internal);

        let mut master_rels = Relationships::new();
        master_rels.add(
            "rId1",
            SLIDE_LAYOUT_REL,
            "../slideLayouts/slideLayout1.xml",
            TargetMode::Internal,
        );
        master_rels.add(
            "rId2",
            THEME_REL,
            "../theme/theme1.xml",
            TargetMode::Internal,
        );

        let mut layout_rels = Relationships::new();
        layout_rels.add(
            "rId1",
            SLIDE_MASTER_REL,
            "../slideMasters/slideMaster1.xml",
            TargetMode::Internal,
        );

        let mut slide_rels = Relationships::new();
        slide_rels.add(
            "rId1",
            SLIDE_LAYOUT_REL,
            "../slideLayouts/slideLayout1.xml",
            TargetMode::Internal,
        );

        let mut part_relationships = BTreeMap::new();
        part_relationships.insert("ppt/presentation.xml".to_string(), pres_rels);
        part_relationships.insert("ppt/slideMasters/slideMaster1.xml".to_string(), master_rels);
        part_relationships.insert("ppt/slideLayouts/slideLayout1.xml".to_string(), layout_rels);
        part_relationships.insert("ppt/slides/slide1.xml".to_string(), slide_rels);

        let presentation = Presentation {
            slide_masters: vec![SlideMasterIdRef {
                id: 2_147_483_648,
                r_id: "rId1".to_string(),
            }],
            slides: vec![SlideIdRef {
                id: 256,
                r_id: "rId2".to_string(),
            }],
            slide_size: SlideSize::default(),
            notes_size: NotesSize::default(),
            ..Default::default()
        };

        let master = SlideMaster::default();
        let layout = SlideLayout {
            layout_type: Some("title".to_string()),
            ..Default::default()
        };
        let slide = Slide::default();
        let theme = Theme {
            name: "Office Theme".to_string(),
            color_scheme: ColorScheme {
                name: "Office".to_string(),
                colors: BTreeMap::new(),
            },
            font_scheme: FontScheme {
                name: "Office".to_string(),
                major_font: FontCollection {
                    latin: Some("Calibri".to_string()),
                    ..Default::default()
                },
                minor_font: FontCollection {
                    latin: Some("Calibri".to_string()),
                    ..Default::default()
                },
            },
        };

        let mut slide_masters = BTreeMap::new();
        slide_masters.insert("ppt/slideMasters/slideMaster1.xml".to_string(), master);

        let mut slide_layouts = BTreeMap::new();
        slide_layouts.insert("ppt/slideLayouts/slideLayout1.xml".to_string(), layout);

        let slides = vec![("ppt/slides/slide1.xml".to_string(), slide)];

        let mut themes = BTreeMap::new();
        themes.insert("ppt/theme/theme1.xml".to_string(), theme);

        Self {
            presentation,
            slide_masters,
            slide_layouts,
            slides,
            notes_slides: BTreeMap::new(),
            themes,
            unparsed_store: UnparsedPartStore::new(),
            content_types,
            package_relationships,
            part_relationships,
        }
    }

    /// Opens an ECMA-376 `.pptx` presentation package from a seekable reader.
    pub fn open<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut pkg_reader = PackageReader::open(reader)?;

        // 1. Read ppt/presentation.xml
        let pres_part_name = if pkg_reader.contains_part("ppt/presentation.xml") {
            "ppt/presentation.xml"
        } else if pkg_reader.contains_part("/ppt/presentation.xml") {
            "/ppt/presentation.xml"
        } else {
            return Err(EngineError::PartNotFound(
                "ppt/presentation.xml".to_string(),
            ));
        };

        let pres_bytes = pkg_reader.read_part(pres_part_name)?;
        let presentation = parse_presentation(&pres_bytes)?;

        // 2. Discover and parse presentation components
        let part_names = pkg_reader.part_names().to_vec();
        let mut slide_masters = BTreeMap::new();
        let mut slide_layouts = BTreeMap::new();
        let mut raw_slides = HashMap::new();
        let mut notes_slides = BTreeMap::new();
        let mut themes = BTreeMap::new();

        let mut modeled_parts = vec![
            "ppt/presentation.xml".to_string(),
            "/ppt/presentation.xml".to_string(),
        ];

        for name in &part_names {
            let clean = name.trim_start_matches('/');
            if clean.starts_with("ppt/slideMasters/slideMaster") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let master = parse_slide_master(&bytes)?;
                slide_masters.insert(clean.to_string(), master);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean.starts_with("ppt/slideLayouts/slideLayout") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let layout = parse_slide_layout(&bytes)?;
                slide_layouts.insert(clean.to_string(), layout);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean.starts_with("ppt/slides/slide") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let slide = parse_slide(&bytes)?;
                raw_slides.insert(clean.to_string(), slide);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean.starts_with("ppt/notesSlides/notesSlide") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let notes = parse_notes_slide(&bytes)?;
                notes_slides.insert(clean.to_string(), notes);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            } else if clean.starts_with("ppt/theme/theme") && clean.ends_with(".xml") {
                let bytes = pkg_reader.read_part(name)?;
                let theme = parse_theme(&bytes)?;
                themes.insert(clean.to_string(), theme);
                modeled_parts.push(clean.to_string());
                modeled_parts.push(format!("/{clean}"));
            }
        }

        // 3. Read package & part relationships
        let package_relationships = pkg_reader
            .get_relationships(None)
            .cloned()
            .unwrap_or_default();

        let mut part_relationships = BTreeMap::new();
        for name in &part_names {
            let clean = name.trim_start_matches('/');
            if clean.ends_with(".xml") {
                if let Ok(rels) = pkg_reader.get_relationships(Some(clean)) {
                    if !rels.is_empty() {
                        part_relationships.insert(clean.to_string(), rels.clone());
                    }
                }
            }
        }

        // 4. Order slides according to presentation.xml <p:sldIdLst>
        let pres_rels = part_relationships
            .get("ppt/presentation.xml")
            .cloned()
            .unwrap_or_default();

        let mut slides = Vec::new();
        for s_ref in &presentation.slides {
            if let Some(rel) = pres_rels.relationships.get(&s_ref.r_id) {
                let slide_path = resolve_target_path("ppt/presentation.xml", &rel.target);
                if let Some(s) = raw_slides.remove(&slide_path) {
                    slides.push((slide_path, s));
                }
            }
        }

        // Add any remaining unreferenced slides in natural order
        let mut remaining_slides: Vec<_> = raw_slides.into_iter().collect();
        remaining_slides.sort_by(|a, b| a.0.cmp(&b.0));
        slides.extend(remaining_slides);

        // 5. Preserve unmodeled parts
        let content_types = pkg_reader.get_content_types()?.clone();
        let modeled_refs: Vec<&str> = modeled_parts.iter().map(|s| s.as_str()).collect();
        let unparsed_store = pkg_reader.extract_unparsed_store(&modeled_refs)?;

        Ok(Self {
            presentation,
            slide_masters,
            slide_layouts,
            slides,
            notes_slides,
            themes,
            unparsed_store,
            content_types,
            package_relationships,
            part_relationships,
        })
    }

    /// Saves the presentation package into an ECMA-376 `.pptx` stream.
    pub fn save<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut pkg_writer = PackageWriter::new(writer);

        pkg_writer.set_content_types(self.content_types.clone());
        pkg_writer.set_package_relationships(self.package_relationships.clone());
        for (source_part, rels) in &self.part_relationships {
            pkg_writer.set_part_relationships(source_part, rels.clone());
        }

        // Write unparsed parts
        pkg_writer.write_unparsed_store(&self.unparsed_store)?;

        // 1. Serialize ppt/presentation.xml
        let pres_xml = serialize_presentation(&self.presentation)?;
        pkg_writer.write_part(
            "ppt/presentation.xml",
            &pres_xml,
            Some(PRESENTATION_MAIN_TYPE),
        )?;

        // 2. Serialize slide masters
        for (name, master) in &self.slide_masters {
            let xml = serialize_slide_master(master)?;
            let clean = name.trim_start_matches('/');
            pkg_writer.write_part(clean, &xml, Some(SLIDE_MASTER_TYPE))?;
        }

        // 3. Serialize slide layouts
        for (name, layout) in &self.slide_layouts {
            let xml = serialize_slide_layout(layout)?;
            let clean = name.trim_start_matches('/');
            pkg_writer.write_part(clean, &xml, Some(SLIDE_LAYOUT_TYPE))?;
        }

        // 4. Serialize slides
        for (name, slide) in &self.slides {
            let xml = serialize_slide(slide)?;
            let clean = name.trim_start_matches('/');
            pkg_writer.write_part(clean, &xml, Some(SLIDE_TYPE))?;
        }

        // 5. Serialize notes slides
        for (name, notes) in &self.notes_slides {
            let xml = serialize_notes_slide(notes)?;
            let clean = name.trim_start_matches('/');
            pkg_writer.write_part(clean, &xml, Some(NOTES_SLIDE_TYPE))?;
        }

        // 6. Serialize themes
        for (name, theme) in &self.themes {
            let xml = serialize_theme(theme)?;
            let clean = name.trim_start_matches('/');
            pkg_writer.write_part(clean, &xml, Some(THEME_TYPE))?;
        }

        pkg_writer.finish()?;
        Ok(())
    }

    /// Total number of slides in presentation.
    pub fn slide_count(&self) -> usize {
        self.slides.len()
    }

    /// Access slide by 0-based index.
    pub fn get_slide(&self, index: usize) -> Option<&Slide> {
        self.slides.get(index).map(|(_, s)| s)
    }

    /// Resolves all shapes on slide at `index` using the 3-tier cascade solver.
    pub fn resolve_slide(&self, index: usize) -> Result<Vec<ResolvedShape>> {
        let (slide_path, slide) = self.slides.get(index).ok_or_else(|| {
            EngineError::InvalidOperation(format!("Slide index {index} out of bounds"))
        })?;

        // Find associated layout
        let slide_rels = self.part_relationships.get(slide_path);
        let layout_path = slide_rels.and_then(|rels| {
            rels.find_by_type(SLIDE_LAYOUT_REL)
                .first()
                .map(|r| resolve_target_path(slide_path, &r.target))
        });

        let layout = layout_path
            .as_deref()
            .and_then(|p| self.slide_layouts.get(p))
            .or_else(|| self.slide_layouts.values().next())
            .ok_or_else(|| {
                EngineError::InvalidOperation("No slide layout available".to_string())
            })?;

        // Find associated master
        let master_path = layout_path.as_deref().and_then(|lp| {
            self.part_relationships.get(lp).and_then(|rels| {
                rels.find_by_type(SLIDE_MASTER_REL)
                    .first()
                    .map(|r| resolve_target_path(lp, &r.target))
            })
        });

        let master = master_path
            .as_deref()
            .and_then(|p| self.slide_masters.get(p))
            .or_else(|| self.slide_masters.values().next())
            .ok_or_else(|| {
                EngineError::InvalidOperation("No slide master available".to_string())
            })?;

        // Find associated theme
        let theme_path = master_path.as_deref().and_then(|mp| {
            self.part_relationships.get(mp).and_then(|rels| {
                rels.find_by_type(THEME_REL)
                    .first()
                    .map(|r| resolve_target_path(mp, &r.target))
            })
        });

        let theme = theme_path
            .as_deref()
            .and_then(|p| self.themes.get(p))
            .or_else(|| self.themes.values().next());

        let resolver = InheritanceResolver::new(Some(&self.presentation), master, layout, theme);
        Ok(resolver.resolve_slide(slide))
    }

    /// Extracts embedded audio and video media assets linked from a slide.
    pub fn extract_slide_media(&self, slide_part_name: &str) -> Vec<crate::media::MediaAsset> {
        let mut assets = Vec::new();
        if let Some(rels) = self.part_relationships.get(slide_part_name) {
            let media_map = crate::media::MediaEngine::extract_media_relationships(rels);
            for (_r_id, (m_type, target)) in media_map {
                let resolved_path = resolve_target_path(slide_part_name, &target);
                if let Some(part) = self.unparsed_store.get(&resolved_path) {
                    let ext = resolved_path.split('.').next_back().unwrap_or("");
                    let format = crate::media::MediaFormat::from_extension(ext);
                    assets.push(crate::media::MediaAsset {
                        part_path: resolved_path,
                        media_type: m_type,
                        format,
                        data: part.content.clone(),
                    });
                }
            }
        }
        assets
    }
}

// =========================================================================
// Helpers
// =========================================================================

pub fn resolve_target_path(source_part: &str, target: &str) -> String {
    if target.starts_with('/') {
        return target.trim_start_matches('/').to_string();
    }
    let source_dir = match source_part.rfind('/') {
        Some(pos) => &source_part[..pos],
        None => "",
    };

    let mut parts: Vec<&str> = if source_dir.is_empty() {
        Vec::new()
    } else {
        source_dir.split('/').collect()
    };

    for segment in target.split('/') {
        if segment == ".." {
            parts.pop();
        } else if segment != "." && !segment.is_empty() {
            parts.push(segment);
        }
    }

    parts.join("/")
}
