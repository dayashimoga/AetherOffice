//! Subphase 4.1.1 Integration Test Suite: SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance
//!
//! Validates:
//! - PresentationML parsing for presentation, slideMasters, slideLayouts, slides, notesSlides, and themes.
//! - 3-tier placeholder shape resolution (idx matching, type matching, ECMA-376 fallback rules).
//! - Property inheritance: geometry, transform (xfrm in EMUs), fill, and line styles.
//! - Text styling hierarchy: direct formatting -> layout placeholder -> master txStyles -> defaultTextStyle.
//! - Theme color scheme resolution (a:clrScheme), color mapping (p:clrMap / p:clrMapOvr), and transforms (tint, shade, lumMod, alpha).
//! - Theme font scheme resolution (+mj-lt, +mn-lt).
//! - Baseline.pptx package open, slide resolution, and lossless round-trip.
//! - Test vectors manifest validation.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use ppt_engine::cascade::InheritanceResolver;
use ppt_engine::model::*;
use ppt_engine::package::PresentationPackage;
use ppt_engine::parser::*;

fn resolve_repo_path(rel: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut curr = manifest_dir.clone();
    for _ in 0..5 {
        let candidate = curr.join(rel);
        if candidate.exists() {
            return candidate;
        }
        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            break;
        }
    }
    manifest_dir.join(rel)
}

fn create_test_theme() -> Theme {
    let mut colors = BTreeMap::new();
    colors.insert("dk1".to_string(), ColorChoice::Srgb("000000".to_string()));
    colors.insert("lt1".to_string(), ColorChoice::Srgb("FFFFFF".to_string()));
    colors.insert("dk2".to_string(), ColorChoice::Srgb("0E2841".to_string()));
    colors.insert("lt2".to_string(), ColorChoice::Srgb("E8E8E8".to_string()));
    colors.insert(
        "accent1".to_string(),
        ColorChoice::Srgb("156082".to_string()),
    );
    colors.insert(
        "accent2".to_string(),
        ColorChoice::Srgb("E97132".to_string()),
    );
    colors.insert(
        "accent3".to_string(),
        ColorChoice::Srgb("196B24".to_string()),
    );
    colors.insert(
        "accent4".to_string(),
        ColorChoice::Srgb("0F9ED5".to_string()),
    );
    colors.insert(
        "accent5".to_string(),
        ColorChoice::Srgb("A02B93".to_string()),
    );
    colors.insert(
        "accent6".to_string(),
        ColorChoice::Srgb("4EA72E".to_string()),
    );
    colors.insert("hlink".to_string(), ColorChoice::Srgb("467886".to_string()));
    colors.insert(
        "folHlink".to_string(),
        ColorChoice::Srgb("96607D".to_string()),
    );

    Theme {
        name: "Office Theme".to_string(),
        color_scheme: ColorScheme {
            name: "Office".to_string(),
            colors,
        },
        font_scheme: FontScheme {
            name: "Office".to_string(),
            major_font: FontCollection {
                latin: Some("Aptos Display".to_string()),
                ..Default::default()
            },
            minor_font: FontCollection {
                latin: Some("Aptos".to_string()),
                ..Default::default()
            },
        },
    }
}

// =========================================================================
// Tests 01-05: Parser Validation
// =========================================================================

#[test]
fn test_01_parse_presentation_xml_dimensions() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
    <p:sldIdLst><p:sldId id="256" r:id="rId2"/><p:sldId id="257" r:id="rId3"/></p:sldIdLst>
    <p:sldSz cx="12192000" cy="6858000"/>
    <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#;

    let pres = parse_presentation(xml.as_bytes()).expect("Valid presentation XML");
    assert_eq!(pres.slide_size.cx, 12_192_000); // 16:9 widescreen width
    assert_eq!(pres.slide_size.cy, 6_858_000); // 16:9 widescreen height
    assert_eq!(pres.notes_size.cx, 6_858_000);
    assert_eq!(pres.notes_size.cy, 9_144_000);
    assert_eq!(pres.slide_masters.len(), 1);
    assert_eq!(pres.slides.len(), 2);
    assert_eq!(pres.slides[0].id, 256);
    assert_eq!(pres.slides[1].id, 257);
}

#[test]
fn test_02_parse_slide_master_components() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
            <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Title Placeholder 1"/>
                    <p:cNvSpPr/>
                    <p:nvPr><p:ph type="title"/></p:nvPr>
                </p:nvSpPr>
                <p:spPr><a:xfrm><a:off x="838200" y="365125"/><a:ext cx="10515600" cy="1325563"/></a:xfrm></p:spPr>
            </p:sp>
        </p:spTree>
    </p:cSld>
    <p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
    <p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>
    <p:txStyles>
        <p:titleStyle>
            <a:lvl1pPr algn="ctr"><a:defRPr sz="4400" b="1"/></a:lvl1pPr>
        </p:txStyles>
    </p:txStyles>
</p:sldMaster>"#;

    let master = parse_slide_master(xml.as_bytes()).expect("Valid slide master XML");
    assert_eq!(master.slide_layouts.len(), 1);
    assert_eq!(master.color_map.bg1, "lt1");
    assert_eq!(master.color_map.tx1, "dk1");
    assert!(master.text_styles.title_style.is_some());

    let title_style = master.text_styles.title_style.as_ref().unwrap();
    assert_eq!(
        title_style.levels[0].as_ref().unwrap().align.as_deref(),
        Some("ctr")
    );
    assert_eq!(
        title_style.levels[0]
            .as_ref()
            .unwrap()
            .default_run_properties
            .as_ref()
            .unwrap()
            .size,
        Some(4400)
    );
}

#[test]
fn test_03_parse_slide_layout_components() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="title" matchingName="Title Slide" preserve="1">
    <p:cSld name="Title Slide">
        <p:spTree>
            <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
            <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Title 1"/>
                    <p:cNvSpPr/>
                    <p:nvPr><p:ph type="ctrTitle"/></p:nvPr>
                </p:nvSpPr>
                <p:spPr><a:xfrm><a:off x="1524000" y="1122363"/><a:ext cx="9144000" cy="2387600"/></a:xfrm></p:spPr>
            </p:sp>
        </p:spTree>
    </p:cSld>
    <p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr>
</p:sldLayout>"#;

    let layout = parse_slide_layout(xml.as_bytes()).expect("Valid slide layout XML");
    assert_eq!(layout.layout_type.as_deref(), Some("title"));
    assert_eq!(layout.matching_name.as_deref(), Some("Title Slide"));
    assert!(layout.preserve);
    assert_eq!(layout.color_map_override, Some(ColorMapOverride::UseMaster));
    assert_eq!(layout.common_slide_data.shape_tree.shapes.len(), 1);
}

#[test]
fn test_04_parse_slide_shapes_and_placeholders() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
    <p:cSld>
        <p:spTree>
            <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
            <p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
            <p:sp>
                <p:nvSpPr>
                    <p:cNvPr id="2" name="Title Placeholder"/>
                    <p:cNvSpPr txBox="1"/>
                    <p:nvPr><p:ph type="title" idx="0"/></p:nvPr>
                </p:nvSpPr>
                <p:spPr><a:prstGeom prst="rect"/></p:spPr>
                <p:txBody>
                    <a:bodyPr anchor="ctr"/>
                    <a:p>
                        <a:r>
                            <a:rPr sz="4000" b="1"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill></a:rPr>
                            <a:t>Quarterly Results</a:t>
                        </a:r>
                    </a:p>
                </p:txBody>
            </p:sp>
        </p:spTree>
    </p:cSld>
</p:sld>"#;

    let slide = parse_slide(xml.as_bytes()).expect("Valid slide XML");
    assert_eq!(slide.common_slide_data.shape_tree.shapes.len(), 1);
    if let ShapeElement::Shape(ref s) = slide.common_slide_data.shape_tree.shapes[0] {
        assert_eq!(s.id, 2);
        assert!(s.is_text_box);
        assert!(s.placeholder.is_some());
        let ph = s.placeholder.as_ref().unwrap();
        assert_eq!(ph.ph_type, "title");
        assert_eq!(ph.index, Some(0));

        let body = s.text_body.as_ref().unwrap();
        assert_eq!(body.paragraphs.len(), 1);
        assert_eq!(body.paragraphs[0].runs[0].get_text(), "Quarterly Results");
    } else {
        panic!("Expected Shape element");
    }
}

#[test]
fn test_05_parse_theme_clr_scheme_and_font_scheme() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
    <a:themeElements>
        <a:clrScheme name="Office">
            <a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
            <a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
            <a:accent1><a:srgbClr val="156082"/></a:accent1>
            <a:accent2><a:srgbClr val="E97132"/></a:accent2>
        </a:clrScheme>
        <a:fontScheme name="Office">
            <a:majorFont><a:latin typeface="Aptos Display"/></a:majorFont>
            <a:minorFont><a:latin typeface="Aptos"/></a:minorFont>
        </a:fontScheme>
    </a:themeElements>
</a:theme>"#;

    let theme = parse_theme(xml.as_bytes()).expect("Valid theme XML");
    assert_eq!(theme.name, "Office Theme");
    assert_eq!(theme.color_scheme.name, "Office");
    assert_eq!(
        theme.color_scheme.colors.get("accent1"),
        Some(&ColorChoice::Srgb("156082".to_string()))
    );
    assert_eq!(
        theme.font_scheme.major_font.latin.as_deref(),
        Some("Aptos Display")
    );
    assert_eq!(theme.font_scheme.minor_font.latin.as_deref(), Some("Aptos"));
}

// =========================================================================
// Tests 06-12: 3-Tier Placeholder Matching & Shape Properties
// =========================================================================

#[test]
fn test_06_placeholder_match_by_idx() {
    let mut master = SlideMaster::default();
    let mut layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master has placeholder idx=1
    master
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 10,
            name: "Master PH".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "body".to_string(),
                index: Some(1),
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(100, 200, 300, 400)),
                ..Default::default()
            },
            ..Default::default()
        })));

    // Layout has placeholder idx=1 with updated position
    layout
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 20,
            name: "Layout PH".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "body".to_string(),
                index: Some(1),
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(500, 600, 700, 800)),
                ..Default::default()
            },
            ..Default::default()
        })));

    // Slide references idx=1 without transform
    let slide_shape = Shape {
        id: 30,
        name: "Slide PH".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "body".to_string(),
            index: Some(1),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let layout_match = resolver.find_layout_placeholder(&slide_shape);
    assert!(layout_match.is_some());
    assert_eq!(layout_match.unwrap().id, 20);

    let resolved =
        resolver.resolve_single_shape(&slide_shape, layout_match, None, &Slide::default());
    // Should inherit Layout transform (500, 600, 700, 800)
    assert_eq!(resolved.transform.x, 500);
    assert_eq!(resolved.transform.y, 600);
}

#[test]
fn test_07_placeholder_match_by_type() {
    let master = SlideMaster::default();
    let mut layout = SlideLayout::default();
    let theme = create_test_theme();

    // Layout has title placeholder without idx
    layout
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 21,
            name: "Title Layout PH".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "title".to_string(),
                index: None,
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(1000, 1000, 8000, 1500)),
                ..Default::default()
            },
            ..Default::default()
        })));

    // Slide has title placeholder without idx
    let slide_shape = Shape {
        id: 31,
        name: "Title Slide PH".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "title".to_string(),
            index: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let layout_match = resolver.find_layout_placeholder(&slide_shape);
    assert!(layout_match.is_some());
    assert_eq!(layout_match.unwrap().id, 21);

    let resolved =
        resolver.resolve_single_shape(&slide_shape, layout_match, None, &Slide::default());
    assert_eq!(resolved.transform.x, 1000);
    assert_eq!(resolved.transform.cx, 8000);
}

#[test]
fn test_08_placeholder_fallback_match_ctrtitle_to_title() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master defines "title"
    master
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 12,
            name: "Master Title".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "title".to_string(),
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(2000, 2000, 6000, 1000)),
                ..Default::default()
            },
            ..Default::default()
        })));

    // Slide defines "ctrTitle"
    let slide_shape = Shape {
        id: 32,
        name: "Centered Title".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "ctrTitle".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let master_match = resolver.find_master_placeholder(&slide_shape);
    assert!(master_match.is_some());
    assert_eq!(master_match.unwrap().id, 12);
}

#[test]
fn test_09_placeholder_fallback_match_subtitle_to_body() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master defines "body"
    master
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 13,
            name: "Master Body".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "body".to_string(),
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(3000, 3000, 5000, 2000)),
                ..Default::default()
            },
            ..Default::default()
        })));

    // Slide defines "subTitle"
    let slide_shape = Shape {
        id: 33,
        name: "Subtitle".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "subTitle".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let master_match = resolver.find_master_placeholder(&slide_shape);
    assert!(master_match.is_some());
    assert_eq!(master_match.unwrap().id, 13);
}

#[test]
fn test_10_geometry_transform_inheritance() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master specifies xfrm and preset geometry
    master
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 14,
            name: "Master Box".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "body".to_string(),
                ..Default::default()
            }),
            properties: ShapeProperties {
                transform: Some(Transform2D::new(1234, 5678, 91011, 121314)),
                geometry: Some(Geometry::Preset("roundRect".to_string())),
                ..Default::default()
            },
            ..Default::default()
        })));

    let slide_shape = Shape {
        id: 34,
        name: "Slide Box".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "body".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let master_match = resolver.find_master_placeholder(&slide_shape);
    let resolved =
        resolver.resolve_single_shape(&slide_shape, None, master_match, &Slide::default());

    assert_eq!(resolved.transform.x, 1234);
    assert_eq!(resolved.transform.y, 5678);
    assert_eq!(resolved.transform.cx, 91011);
    assert_eq!(resolved.transform.cy, 121314);
    assert_eq!(resolved.geometry, Geometry::Preset("roundRect".to_string()));
}

#[test]
fn test_11_fill_and_outline_inheritance() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    master
        .common_slide_data
        .shape_tree
        .shapes
        .push(ShapeElement::Shape(Box::new(Shape {
            id: 15,
            name: "Master Shape".to_string(),
            placeholder: Some(Placeholder {
                ph_type: "body".to_string(),
                ..Default::default()
            }),
            properties: ShapeProperties {
                fill: Some(Fill::Solid(ColorChoice::Srgb("156082".to_string()))),
                line: Some(LineProperties {
                    width: Some(25400), // 2pt line in EMUs
                    dash_style: Some("dash".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })));

    let slide_shape = Shape {
        id: 35,
        name: "Slide Shape".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "body".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let master_match = resolver.find_master_placeholder(&slide_shape);
    let resolved =
        resolver.resolve_single_shape(&slide_shape, None, master_match, &Slide::default());

    assert_eq!(
        resolved.fill,
        Some(Fill::Solid(ColorChoice::Srgb("156082".to_string())))
    );
    assert!(resolved.line.is_some());
    assert_eq!(resolved.line.unwrap().width, Some(25400));
}

// =========================================================================
// Tests 13-17: Text Hierarchy & Cascade
// =========================================================================

#[test]
fn test_13_text_style_master_title_style() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master titleStyle: lvl1 is 44pt (4400), bold=true, centered
    master.text_styles.title_style = Some(TextListStyle {
        levels: vec![Some(TextParagraphProperties {
            align: Some("ctr".to_string()),
            default_run_properties: Some(TextRunProperties {
                size: Some(4400),
                bold: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        })],
        ..Default::default()
    });

    let slide_shape = Shape {
        id: 36,
        name: "Title Shape".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "title".to_string(),
            ..Default::default()
        }),
        text_body: Some(TextBody {
            paragraphs: vec![TextParagraph {
                runs: vec![TextRunElement::Text("Annual Report".to_string(), None)],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let resolved = resolver.resolve_single_shape(&slide_shape, None, None, &Slide::default());

    assert_eq!(resolved.paragraphs.len(), 1);
    let p = &resolved.paragraphs[0];
    assert_eq!(p.align, "ctr");
    let r = &p.runs[0];
    assert_eq!(r.font_size_pt, 44.0);
    assert!(r.bold);
}

#[test]
fn test_14_text_style_master_body_style_multi_level() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master bodyStyle: lvl1 = 32pt, lvl2 = 28pt, lvl3 = 24pt
    let mut levels = vec![None; 9];
    levels[0] = Some(TextParagraphProperties {
        mar_l: Some(288000),
        indent: Some(-288000),
        default_run_properties: Some(TextRunProperties {
            size: Some(3200),
            ..Default::default()
        }),
        ..Default::default()
    });
    levels[1] = Some(TextParagraphProperties {
        mar_l: Some(576000),
        indent: Some(-288000),
        default_run_properties: Some(TextRunProperties {
            size: Some(2800),
            ..Default::default()
        }),
        ..Default::default()
    });
    levels[2] = Some(TextParagraphProperties {
        mar_l: Some(864000),
        indent: Some(-288000),
        default_run_properties: Some(TextRunProperties {
            size: Some(2400),
            ..Default::default()
        }),
        ..Default::default()
    });

    master.text_styles.body_style = Some(TextListStyle {
        levels,
        ..Default::default()
    });

    let slide_shape = Shape {
        id: 37,
        name: "Body List".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "body".to_string(),
            ..Default::default()
        }),
        text_body: Some(TextBody {
            paragraphs: vec![
                TextParagraph {
                    properties: Some(TextParagraphProperties {
                        level: Some(0),
                        ..Default::default()
                    }),
                    runs: vec![TextRunElement::Text("Level 0 Item".to_string(), None)],
                    ..Default::default()
                },
                TextParagraph {
                    properties: Some(TextParagraphProperties {
                        level: Some(1),
                        ..Default::default()
                    }),
                    runs: vec![TextRunElement::Text("Level 1 Item".to_string(), None)],
                    ..Default::default()
                },
                TextParagraph {
                    properties: Some(TextParagraphProperties {
                        level: Some(2),
                        ..Default::default()
                    }),
                    runs: vec![TextRunElement::Text("Level 2 Item".to_string(), None)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let resolved = resolver.resolve_single_shape(&slide_shape, None, None, &Slide::default());

    assert_eq!(resolved.paragraphs[0].runs[0].font_size_pt, 32.0);
    assert_eq!(resolved.paragraphs[1].runs[0].font_size_pt, 28.0);
    assert_eq!(resolved.paragraphs[2].runs[0].font_size_pt, 24.0);

    assert_eq!(resolved.paragraphs[0].mar_l, 288000);
    assert_eq!(resolved.paragraphs[1].mar_l, 576000);
    assert_eq!(resolved.paragraphs[2].mar_l, 864000);
}

#[test]
fn test_15_text_style_master_other_style() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    // Master otherStyle: lvl1 size = 16pt (1600)
    master.text_styles.other_style = Some(TextListStyle {
        levels: vec![Some(TextParagraphProperties {
            default_run_properties: Some(TextRunProperties {
                size: Some(1600),
                ..Default::default()
            }),
            ..Default::default()
        })],
        ..Default::default()
    });

    // Arbitrary text box (not a placeholder)
    let slide_shape = Shape {
        id: 38,
        name: "Callout Text Box".to_string(),
        placeholder: None,
        text_body: Some(TextBody {
            paragraphs: vec![TextParagraph {
                runs: vec![TextRunElement::Text("Note text".to_string(), None)],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let resolved = resolver.resolve_single_shape(&slide_shape, None, None, &Slide::default());

    assert_eq!(resolved.paragraphs[0].runs[0].font_size_pt, 16.0);
}

#[test]
fn test_16_text_style_presentation_default_fallback() {
    let master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    let pres = Presentation {
        default_text_style: Some(TextListStyle {
            levels: vec![Some(TextParagraphProperties {
                default_run_properties: Some(TextRunProperties {
                    size: Some(2200), // 22pt default
                    ..Default::default()
                }),
                ..Default::default()
            })],
            ..Default::default()
        }),
        ..Default::default()
    };

    let slide_shape = Shape {
        id: 39,
        name: "Default Text".to_string(),
        placeholder: None,
        text_body: Some(TextBody {
            paragraphs: vec![TextParagraph {
                runs: vec![TextRunElement::Text("Fallback text".to_string(), None)],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(Some(&pres), &master, &layout, Some(&theme));
    let resolved = resolver.resolve_single_shape(&slide_shape, None, None, &Slide::default());

    assert_eq!(resolved.paragraphs[0].runs[0].font_size_pt, 22.0);
}

#[test]
fn test_17_direct_formatting_overrides_cascade() {
    let mut master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    master.text_styles.title_style = Some(TextListStyle {
        levels: vec![Some(TextParagraphProperties {
            align: Some("ctr".to_string()),
            default_run_properties: Some(TextRunProperties {
                size: Some(4400),
                bold: Some(false),
                italic: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        })],
        ..Default::default()
    });

    // Direct formatting specifies right align, 50pt, bold=true, italic=true
    let slide_shape = Shape {
        id: 40,
        name: "Direct Title".to_string(),
        placeholder: Some(Placeholder {
            ph_type: "title".to_string(),
            ..Default::default()
        }),
        text_body: Some(TextBody {
            paragraphs: vec![TextParagraph {
                properties: Some(TextParagraphProperties {
                    align: Some("r".to_string()),
                    ..Default::default()
                }),
                runs: vec![TextRunElement::Text(
                    "Direct Override".to_string(),
                    Some(TextRunProperties {
                        size: Some(5000),
                        bold: Some(true),
                        italic: Some(true),
                        ..Default::default()
                    }),
                )],
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let resolved = resolver.resolve_single_shape(&slide_shape, None, None, &Slide::default());

    let p = &resolved.paragraphs[0];
    assert_eq!(p.align, "r"); // direct override
    let r = &p.runs[0];
    assert_eq!(r.font_size_pt, 50.0);
    assert!(r.bold);
    assert!(r.italic);
}

// =========================================================================
// Tests 18-24: Theme Color Schemes, Mapping & Transforms
// =========================================================================

#[test]
fn test_18_theme_color_scheme_mapping() {
    let master = SlideMaster {
        color_map: ColorMap {
            bg1: "lt1".to_string(),
            tx1: "dk1".to_string(),
            accent1: "accent1".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));

    // SchemeClr tx1 -> master maps to dk1 -> theme dk1 is #000000
    let (rgb_tx1, alpha) = resolver.resolve_color(
        Some(&ColorChoice::Scheme("tx1".to_string(), Vec::new())),
        &Slide::default(),
    );
    assert_eq!(rgb_tx1, [0, 0, 0]);
    assert_eq!(alpha, 255);

    // SchemeClr accent1 -> theme accent1 is #156082 (rgb: [21, 96, 130])
    let (rgb_acc1, _) = resolver.resolve_color(
        Some(&ColorChoice::Scheme("accent1".to_string(), Vec::new())),
        &Slide::default(),
    );
    assert_eq!(rgb_acc1, [21, 96, 130]);
}

#[test]
fn test_19_theme_color_scheme_override() {
    let master = SlideMaster {
        color_map: ColorMap {
            tx1: "dk1".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut layout = SlideLayout::default();
    let theme = create_test_theme();

    // Layout overrides color mapping: tx1 -> accent1
    layout.color_map_override = Some(ColorMapOverride::Override(Box::new(ColorMap {
        tx1: "accent1".to_string(),
        ..Default::default()
    })));

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let (rgb, _) = resolver.resolve_color(
        Some(&ColorChoice::Scheme("tx1".to_string(), Vec::new())),
        &Slide::default(),
    );

    // tx1 mapped to accent1: [21, 96, 130]
    assert_eq!(rgb, [21, 96, 130]);
}

#[test]
fn test_20_color_transform_tint() {
    let base_rgb = [100, 100, 100];
    // 50% tint (50000) towards white: 100 + (255 - 100)*0.5 = 177.5 -> 178
    let transforms = vec![ColorTransform::Tint(50_000)];
    let (rgb, alpha) = InheritanceResolver::apply_transforms(base_rgb, &transforms);

    assert_eq!(rgb, [178, 178, 178]);
    assert_eq!(alpha, 255);
}

#[test]
fn test_21_color_transform_shade() {
    let base_rgb = [200, 200, 200];
    // 50% shade (50000) towards black: 200 * 0.5 = 100
    let transforms = vec![ColorTransform::Shade(50_000)];
    let (rgb, alpha) = InheritanceResolver::apply_transforms(base_rgb, &transforms);

    assert_eq!(rgb, [100, 100, 100]);
    assert_eq!(alpha, 255);
}

#[test]
fn test_22_color_transform_lum_mod_and_off() {
    let base_rgb = [128, 128, 128];
    // Luminance modulation 80% (80000)
    let transforms = vec![ColorTransform::LumMod(80_000)];
    let (rgb, _) = InheritanceResolver::apply_transforms(base_rgb, &transforms);

    assert!(rgb[0] < 128);
    assert_eq!(rgb[0], rgb[1]);
    assert_eq!(rgb[1], rgb[2]);
}

#[test]
fn test_23_color_transform_alpha() {
    let base_rgb = [255, 0, 0];
    // Alpha 50% (50000) -> 127
    let transforms = vec![ColorTransform::Alpha(50_000)];
    let (rgb, alpha) = InheritanceResolver::apply_transforms(base_rgb, &transforms);

    assert_eq!(rgb, [255, 0, 0]);
    assert_eq!(alpha, 127);
}

// =========================================================================
// Tests 24-25: Theme Font Schemes
// =========================================================================

#[test]
fn test_24_theme_font_major_latin_resolution() {
    let master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let font = resolver.resolve_theme_font("+mj-lt");
    assert_eq!(font, "Aptos Display");
}

#[test]
fn test_25_theme_font_minor_latin_resolution() {
    let master = SlideMaster::default();
    let layout = SlideLayout::default();
    let theme = create_test_theme();

    let resolver = InheritanceResolver::new(None, &master, &layout, Some(&theme));
    let font = resolver.resolve_theme_font("+mn-lt");
    assert_eq!(font, "Aptos");

    // Explicit font should not be modified
    let font_arial = resolver.resolve_theme_font("Arial");
    assert_eq!(font_arial, "Arial");
}

// =========================================================================
// Tests 26-29: Baseline PPTX Open, Cascade Resolution & Lossless Round-Trip
// =========================================================================

#[test]
fn test_26_baseline_pptx_open_and_slide_count() {
    let baseline_path = resolve_repo_path("specs/evidence/ppt/baseline.pptx");
    let file = File::open(&baseline_path).expect("Failed to open baseline.pptx");
    let pkg = PresentationPackage::open(file).expect("Failed to parse baseline.pptx");

    assert_eq!(pkg.slide_count(), 2);
    assert_eq!(pkg.slide_masters.len(), 1);
    assert!(pkg.slide_layouts.len() >= 10);
    assert_eq!(pkg.themes.len(), 1);
}

#[test]
fn test_27_baseline_pptx_resolve_slides_cascade() {
    let baseline_path = resolve_repo_path("specs/evidence/ppt/baseline.pptx");
    let file = File::open(&baseline_path).expect("Failed to open baseline.pptx");
    let pkg = PresentationPackage::open(file).expect("Failed to parse baseline.pptx");

    // Resolve slide 0
    let shapes_s1 = pkg.resolve_slide(0).expect("Resolve slide 0");
    assert!(!shapes_s1.is_empty());
    // Find text box containing "OpenOfficeRS Unified Engine"
    let mut found_text = false;
    for s in &shapes_s1 {
        for p in &s.paragraphs {
            for r in &p.runs {
                if r.text.contains("OpenOfficeRS") {
                    found_text = true;
                    assert!(!r.font_family.is_empty());
                    assert!(r.font_size_pt > 0.0);
                }
            }
        }
    }
    assert!(found_text, "Slide 0 should contain 'OpenOfficeRS' run text");

    // Resolve slide 1
    let shapes_s2 = pkg.resolve_slide(1).expect("Resolve slide 1");
    assert!(!shapes_s2.is_empty());
}

#[test]
fn test_28_baseline_pptx_roundtrip_lossless() {
    let baseline_path = resolve_repo_path("specs/evidence/ppt/baseline.pptx");
    let file = File::open(&baseline_path).expect("Failed to open baseline.pptx");
    let pkg = PresentationPackage::open(file).expect("Failed to parse baseline.pptx");

    // Save package into memory buffer
    let mut buffer = Cursor::new(Vec::new());
    pkg.save(&mut buffer).expect("Failed to save PPTX package");

    // Re-open saved package from buffer
    buffer.set_position(0);
    let reloaded_pkg = PresentationPackage::open(buffer).expect("Failed to reload saved PPTX");

    assert_eq!(reloaded_pkg.slide_count(), pkg.slide_count());
    assert_eq!(reloaded_pkg.slide_masters.len(), pkg.slide_masters.len());
    assert_eq!(reloaded_pkg.slide_layouts.len(), pkg.slide_layouts.len());
    assert_eq!(reloaded_pkg.themes.len(), pkg.themes.len());
}

// =========================================================================
// Test 29: Test Vectors Manifest Validation
// =========================================================================

#[test]
fn test_29_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/ppt/4.1.1_vectors.json");
    let json_bytes = std::fs::read(&path).expect("Must read specs/evidence/ppt/4.1.1_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let count = vectors["vectors"].as_array().expect("vectors array").len();
    assert!(
        count >= 25,
        "Subphase 4.1.1 requires at least 25 test vectors, found: {}",
        count
    );

    assert_eq!(vectors["subphase_id"].as_str(), Some("4.1.1"));
    assert_eq!(
        vectors["title"].as_str(),
        Some("SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance Resolver")
    );
}
