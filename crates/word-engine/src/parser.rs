//! OpenOfficeRS Word Engine - Streaming WordprocessingML Parser.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Streaming XML parser using `quick-xml` for `document.xml`, headers, footers,
//! footnotes, endnotes, and comments.

use crate::model::*;
use crate::sub_documents::*;
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::Cursor;

fn read_element_text<R: std::io::BufRead>(reader: &mut Reader<R>, end_tag: &str) -> Result<String> {
    let mut text = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Text(e) => {
                let unesc = e.unescape().map_err(EngineError::Xml)?;
                text.push_str(&unesc);
            }
            Event::CData(e) => {
                let s = String::from_utf8_lossy(&e.into_inner()).to_string();
                text.push_str(&s);
            }
            Event::End(ref e) => {
                if local_name(e.name().into_inner()) == end_tag {
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

/// Parses a full WordprocessingML `word/document.xml` part.
pub fn parse_document(xml_bytes: &[u8]) -> Result<Document> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut doc = Document::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "document" => {
                extract_namespaces_and_attributes(e, &mut doc.namespaces, &mut doc.ignorable)?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "body" => {
                doc.body = parse_body(&mut reader)?;
            }
            Event::Start(ref e) => {
                doc.unparsed_nodes.push(parse_raw_xml_node(&mut reader, e)?);
            }
            Event::Empty(ref e) => {
                doc.unparsed_nodes.push(parse_empty_raw_node(e)?);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(doc)
}

/// Parses a header part: `<w:hdr>`.
pub fn parse_header(xml_bytes: &[u8]) -> Result<HeaderDocument> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut header = HeaderDocument::default();
    let mut buf = Vec::new();
    let mut dummy_ignorable = None;

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "hdr" => {
                extract_namespaces_and_attributes(e, &mut header.namespaces, &mut dummy_ignorable)?;
                header.body = parse_body(&mut reader)?;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(header)
}

/// Parses a footer part: `<w:ftr>`.
pub fn parse_footer(xml_bytes: &[u8]) -> Result<FooterDocument> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut footer = FooterDocument::default();
    let mut buf = Vec::new();
    let mut dummy_ignorable = None;

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "ftr" => {
                extract_namespaces_and_attributes(e, &mut footer.namespaces, &mut dummy_ignorable)?;
                footer.body = parse_body(&mut reader)?;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(footer)
}

/// Parses a footnotes part: `<w:footnotes>`.
pub fn parse_footnotes(xml_bytes: &[u8]) -> Result<FootnotesDocument> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut footnotes_doc = FootnotesDocument::default();
    let mut buf = Vec::new();
    let mut dummy_ignorable = None;

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "footnotes" => {
                extract_namespaces_and_attributes(
                    e,
                    &mut footnotes_doc.namespaces,
                    &mut dummy_ignorable,
                )?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "footnote" => {
                let mut id = 0i64;
                let mut fn_type = None;
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    if local_name(key.as_bytes()) == "id" {
                        id = val.parse::<i64>().unwrap_or(0);
                    } else if local_name(key.as_bytes()) == "type" {
                        fn_type = Some(val);
                    }
                }
                let body = parse_body_elements_until(&mut reader, "footnote")?;
                footnotes_doc.footnotes.push(Footnote {
                    id,
                    footnote_type: fn_type,
                    body: Body {
                        elements: body,
                        sect_pr: None,
                    },
                });
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(footnotes_doc)
}

/// Parses an endnotes part: `<w:endnotes>`.
pub fn parse_endnotes(xml_bytes: &[u8]) -> Result<EndnotesDocument> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut endnotes_doc = EndnotesDocument::default();
    let mut buf = Vec::new();
    let mut dummy_ignorable = None;

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "endnotes" => {
                extract_namespaces_and_attributes(
                    e,
                    &mut endnotes_doc.namespaces,
                    &mut dummy_ignorable,
                )?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "endnote" => {
                let mut id = 0i64;
                let mut en_type = None;
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    if local_name(key.as_bytes()) == "id" {
                        id = val.parse::<i64>().unwrap_or(0);
                    } else if local_name(key.as_bytes()) == "type" {
                        en_type = Some(val);
                    }
                }
                let body = parse_body_elements_until(&mut reader, "endnote")?;
                endnotes_doc.endnotes.push(Endnote {
                    id,
                    endnote_type: en_type,
                    body: Body {
                        elements: body,
                        sect_pr: None,
                    },
                });
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(endnotes_doc)
}

/// Parses a comments part: `<w:comments>`.
pub fn parse_comments(xml_bytes: &[u8]) -> Result<CommentsDocument> {
    let mut reader = Reader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(false);

    let mut comments_doc = CommentsDocument::default();
    let mut buf = Vec::new();
    let mut dummy_ignorable = None;

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "comments" => {
                extract_namespaces_and_attributes(
                    e,
                    &mut comments_doc.namespaces,
                    &mut dummy_ignorable,
                )?;
            }
            Event::Start(ref e) if local_name(e.name().into_inner()) == "comment" => {
                let mut id = String::new();
                let mut author = String::new();
                let mut initials = None;
                let mut date = None;

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    match local_name(key.as_bytes()) {
                        "id" => id = val,
                        "author" => author = val,
                        "initials" => initials = Some(val),
                        "date" => date = Some(val),
                        _ => {}
                    }
                }

                let body = parse_body_elements_until(&mut reader, "comment")?;
                comments_doc.comments.push(Comment {
                    id,
                    author,
                    initials,
                    date,
                    body: Body {
                        elements: body,
                        sect_pr: None,
                    },
                });
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(comments_doc)
}

// --- Body Parsing ---

fn parse_body<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<Body> {
    let mut elements = Vec::new();
    let mut sect_pr = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "p" => elements.push(BodyElement::Paragraph(parse_paragraph(reader, e)?)),
                    "tbl" => elements.push(BodyElement::Table(parse_table(reader, e)?)),
                    "sectPr" => sect_pr = Some(parse_section_properties(reader, e)?),
                    _ => elements.push(BodyElement::Raw(parse_raw_xml_node(reader, e)?)),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "p" => elements.push(BodyElement::Paragraph(parse_empty_paragraph(e)?)),
                    "sectPr" => sect_pr = Some(parse_empty_section_properties(e)?),
                    _ => elements.push(BodyElement::Raw(parse_empty_raw_node(e)?)),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "body" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(Body { elements, sect_pr })
}

fn parse_body_elements_until<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    end_tag: &str,
) -> Result<Vec<BodyElement>> {
    let mut elements = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "p" => elements.push(BodyElement::Paragraph(parse_paragraph(reader, e)?)),
                    "tbl" => elements.push(BodyElement::Table(parse_table(reader, e)?)),
                    _ => elements.push(BodyElement::Raw(parse_raw_xml_node(reader, e)?)),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "p" => elements.push(BodyElement::Paragraph(parse_empty_paragraph(e)?)),
                    _ => elements.push(BodyElement::Raw(parse_empty_raw_node(e)?)),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(elements)
}

// --- Paragraph Parsing ---

fn parse_paragraph<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<Paragraph> {
    let mut para = Paragraph::default();
    extract_paragraph_attributes(start_tag, &mut para);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pPr" => para.properties = Some(parse_paragraph_properties(reader)?),
                    "r" => para
                        .children
                        .push(ParagraphChild::Run(parse_run(reader, e)?)),
                    "hyperlink" => para
                        .children
                        .push(ParagraphChild::Hyperlink(parse_hyperlink(reader, e)?)),
                    "ins" => para
                        .children
                        .push(ParagraphChild::Inserted(parse_tracked_change(
                            reader, e, "ins",
                        )?)),
                    "del" => para
                        .children
                        .push(ParagraphChild::Deleted(parse_tracked_change(
                            reader, e, "del",
                        )?)),
                    _ => para
                        .children
                        .push(ParagraphChild::Raw(parse_raw_xml_node(reader, e)?)),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pPr" => para.properties = Some(parse_empty_paragraph_properties(e)?),
                    "r" => para.children.push(ParagraphChild::Run(parse_empty_run(e)?)),
                    "bookmarkStart" => {
                        para.children
                            .push(ParagraphChild::BookmarkStart(parse_bookmark_start(e)?));
                    }
                    "bookmarkEnd" => {
                        para.children
                            .push(ParagraphChild::BookmarkEnd(parse_bookmark_end(e)?));
                    }
                    "commentRangeStart" => {
                        if let Some(id) = get_attr_val(e, "id") {
                            para.children.push(ParagraphChild::CommentRangeStart(id));
                        }
                    }
                    "commentRangeEnd" => {
                        if let Some(id) = get_attr_val(e, "id") {
                            para.children.push(ParagraphChild::CommentRangeEnd(id));
                        }
                    }
                    "commentReference" => {
                        if let Some(id) = get_attr_val(e, "id") {
                            para.children.push(ParagraphChild::CommentReference(id));
                        }
                    }
                    "footnoteReference" => {
                        if let Some(id) = get_attr_val(e, "id") {
                            para.children.push(ParagraphChild::FootnoteReference(id));
                        }
                    }
                    "endnoteReference" => {
                        if let Some(id) = get_attr_val(e, "id") {
                            para.children.push(ParagraphChild::EndnoteReference(id));
                        }
                    }
                    _ => para
                        .children
                        .push(ParagraphChild::Raw(parse_empty_raw_node(e)?)),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "p" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(para)
}

fn parse_empty_paragraph(e: &BytesStart) -> Result<Paragraph> {
    let mut para = Paragraph::default();
    extract_paragraph_attributes(e, &mut para);
    Ok(para)
}

fn extract_paragraph_attributes(e: &BytesStart, para: &mut Paragraph) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match local_name(key.as_bytes()) {
            "paraId" => para.para_id = Some(val),
            "textId" => para.text_id = Some(val),
            "rsidR" => para.rsid_r = Some(val),
            "rsidRDefault" => para.rsid_r_default = Some(val),
            _ => para.raw_attrs.push((key, val)),
        }
    }
}

pub(crate) fn parse_paragraph_properties<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<ParagraphProperties> {
    let mut p_pr = ParagraphProperties::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "rPr" => p_pr.r_pr = Some(parse_run_properties(reader)?),
                    "numPr" => {
                        let mut num_id = 0;
                        let mut ilvl = 0;
                        let mut np_buf = Vec::new();
                        loop {
                            match reader
                                .read_event_into(&mut np_buf)
                                .map_err(EngineError::Xml)?
                            {
                                Event::Empty(ref ne) => {
                                    let nname = local_name(ne.name().into_inner());
                                    if nname == "numId" {
                                        num_id = get_attr_val(ne, "val")
                                            .and_then(|s| s.parse::<u32>().ok())
                                            .unwrap_or(0);
                                    } else if nname == "ilvl" {
                                        ilvl = get_attr_val(ne, "val")
                                            .and_then(|s| s.parse::<u32>().ok())
                                            .unwrap_or(0);
                                    }
                                }
                                Event::End(ref ne)
                                    if local_name(ne.name().into_inner()) == "numPr" =>
                                {
                                    break
                                }
                                Event::Eof => break,
                                _ => {}
                            }
                            np_buf.clear();
                        }
                        p_pr.num_pr = Some(NumberingProperty { num_id, ilvl });
                    }
                    _ => p_pr.raw_elements.push(parse_raw_xml_node(reader, e)?),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "pStyle" => p_pr.style = get_attr_val(e, "val"),
                    "jc" => p_pr.justification = get_attr_val(e, "val"),
                    "spacing" => p_pr.spacing = Some(parse_spacing(e)),
                    "ind" => p_pr.indentation = Some(parse_indentation(e)),
                    "rPr" => p_pr.r_pr = Some(parse_empty_run_properties(e)),
                    "widowControl" => p_pr.widow_control = Some(parse_boolean_attr(e)),
                    "keepNext" => p_pr.keep_next = parse_boolean_attr(e),
                    "keepLines" => p_pr.keep_lines = parse_boolean_attr(e),
                    "pageBreakBefore" => p_pr.page_break_before = parse_boolean_attr(e),
                    "kinsoku" => p_pr.kinsoku = Some(parse_boolean_attr(e)),
                    _ => p_pr.raw_elements.push(parse_empty_raw_node(e)?),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "pPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(p_pr)
}

fn parse_empty_paragraph_properties(e: &BytesStart) -> Result<ParagraphProperties> {
    Ok(ParagraphProperties {
        style: get_attr_val(e, "val"),
        ..Default::default()
    })
}

fn parse_spacing(e: &BytesStart) -> Spacing {
    let before = get_attr_val(e, "before").and_then(|s| s.parse::<u32>().ok());
    let after = get_attr_val(e, "after").and_then(|s| s.parse::<u32>().ok());
    let line = get_attr_val(e, "line").and_then(|s| s.parse::<u32>().ok());
    let line_rule = get_attr_val(e, "lineRule");
    Spacing {
        before,
        after,
        line,
        line_rule,
    }
}

fn parse_indentation(e: &BytesStart) -> Indentation {
    let left = get_attr_val(e, "left").and_then(|s| s.parse::<i32>().ok());
    let right = get_attr_val(e, "right").and_then(|s| s.parse::<i32>().ok());
    let first_line = get_attr_val(e, "firstLine").and_then(|s| s.parse::<i32>().ok());
    let hanging = get_attr_val(e, "hanging").and_then(|s| s.parse::<i32>().ok());
    Indentation {
        left,
        right,
        first_line,
        hanging,
    }
}

// --- Run Parsing ---

fn parse_run<R: std::io::BufRead>(reader: &mut Reader<R>, start_tag: &BytesStart) -> Result<Run> {
    let mut run = Run::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match local_name(key.as_bytes()) {
            "rsidRPr" => run.rsid_r_pr = Some(val),
            "rsidR" => run.rsid_r = Some(val),
            _ => run.raw_attrs.push((key, val)),
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "rPr" => run.properties = Some(parse_run_properties(reader)?),
                    "t" => {
                        let preserve = is_xml_space_preserve(e);
                        let text = read_element_text(reader, "t")?;
                        run.content.push(RunContent::Text(text, preserve));
                    }
                    "delText" => {
                        let preserve = is_xml_space_preserve(e);
                        let text = read_element_text(reader, "delText")?;
                        run.content.push(RunContent::DeletedText(text, preserve));
                    }
                    _ => run
                        .content
                        .push(RunContent::Raw(parse_raw_xml_node(reader, e)?)),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "rPr" => run.properties = Some(parse_empty_run_properties(e)),
                    "t" => run
                        .content
                        .push(RunContent::Text(String::new(), is_xml_space_preserve(e))),
                    "br" => run.content.push(RunContent::Break(get_attr_val(e, "type"))),
                    "tab" => run.content.push(RunContent::Tab),
                    "softHyphen" => run.content.push(RunContent::SoftHyphen),
                    "noBreakHyphen" => run.content.push(RunContent::NoBreakHyphen),
                    "separator" => run.content.push(RunContent::Separator),
                    "continuationSeparator" => run.content.push(RunContent::ContinuationSeparator),
                    "footnoteRef" => run.content.push(RunContent::FootnoteRef),
                    "endnoteRef" => run.content.push(RunContent::EndnoteRef),
                    _ => run.content.push(RunContent::Raw(parse_empty_raw_node(e)?)),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "r" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(run)
}

fn parse_empty_run(e: &BytesStart) -> Result<Run> {
    let mut run = Run::default();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        run.raw_attrs.push((key, val));
    }
    Ok(run)
}

pub(crate) fn parse_run_properties<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<RunProperties> {
    let mut r_pr = RunProperties::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                r_pr.raw_elements.push(parse_raw_xml_node(reader, e)?);
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "b" => r_pr.bold = parse_boolean_attr(e),
                    "i" => r_pr.italic = parse_boolean_attr(e),
                    "u" => {
                        r_pr.underline =
                            get_attr_val(e, "val").or_else(|| Some("single".to_string()))
                    }
                    "strike" => r_pr.strike = parse_boolean_attr(e),
                    "dstrike" => r_pr.double_strike = parse_boolean_attr(e),
                    "color" => r_pr.color = get_attr_val(e, "val"),
                    "sz" => {
                        r_pr.size_half_points =
                            get_attr_val(e, "val").and_then(|s| s.parse::<u32>().ok())
                    }
                    "highlight" => r_pr.highlight = get_attr_val(e, "val"),
                    "rFonts" => {
                        r_pr.font_ascii = get_attr_val(e, "ascii");
                        r_pr.font_h_ansi = get_attr_val(e, "hAnsi");
                        r_pr.font_east_asia = get_attr_val(e, "eastAsia");
                        r_pr.font_cs = get_attr_val(e, "cs");
                    }
                    _ => r_pr.raw_elements.push(parse_empty_raw_node(e)?),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "rPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(r_pr)
}

fn parse_empty_run_properties(e: &BytesStart) -> RunProperties {
    let mut r_pr = RunProperties::default();
    if local_name(e.name().into_inner()) == "b" {
        r_pr.bold = true;
    }
    r_pr
}

// --- Hyperlinks, Bookmarks & Revisions ---

fn parse_hyperlink<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<Hyperlink> {
    let mut h = Hyperlink::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match local_name(key.as_bytes()) {
            "id" => h.id = Some(val),
            "anchor" => h.anchor = Some(val),
            "tooltip" => h.tooltip = Some(val),
            "history" => h.history = val != "0" && val != "false",
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "r" => {
                h.children.push(ParagraphChild::Run(parse_run(reader, e)?));
            }
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "r" => {
                h.children.push(ParagraphChild::Run(parse_empty_run(e)?));
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "hyperlink" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(h)
}

fn parse_tracked_change<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
    end_tag: &str,
) -> Result<TrackedChange> {
    let mut tc = TrackedChange::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match local_name(key.as_bytes()) {
            "id" => tc.id = val,
            "author" => tc.author = val,
            "date" => tc.date = val,
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "r" => {
                tc.children.push(ParagraphChild::Run(parse_run(reader, e)?));
            }
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "r" => {
                tc.children.push(ParagraphChild::Run(parse_empty_run(e)?));
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(tc)
}

fn parse_bookmark_start(e: &BytesStart) -> Result<BookmarkStart> {
    let id = get_attr_val(e, "id").unwrap_or_default();
    let name = get_attr_val(e, "name").unwrap_or_default();
    let col_first = get_attr_val(e, "colFirst").and_then(|s| s.parse::<u32>().ok());
    let col_last = get_attr_val(e, "colLast").and_then(|s| s.parse::<u32>().ok());
    Ok(BookmarkStart {
        id,
        name,
        col_first,
        col_last,
    })
}

fn parse_bookmark_end(e: &BytesStart) -> Result<BookmarkEnd> {
    let id = get_attr_val(e, "id").unwrap_or_default();
    Ok(BookmarkEnd { id })
}

// --- Table Parsing ---

fn parse_table<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<Table> {
    let mut table = Table::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        table.raw_attrs.push((key, val));
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "tblPr" => table.properties = Some(parse_table_properties(reader)?),
                    "tblGrid" => table.grid = Some(parse_table_grid(reader)?),
                    "tr" => table.rows.push(parse_table_row(reader, e)?),
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "tblPr" => table.properties = Some(TableProperties::default()),
                    "tr" => table.rows.push(parse_empty_table_row(e)?),
                    _ => {}
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tbl" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(table)
}

fn parse_table_properties<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<TableProperties> {
    let mut tbl_pr = TableProperties::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                tbl_pr.raw_elements.push(parse_raw_xml_node(reader, e)?);
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "tblW" => {
                        let w = get_attr_val(e, "w")
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);
                        let w_type = get_attr_val(e, "type").unwrap_or_else(|| "auto".to_string());
                        tbl_pr.width = Some(TableWidth {
                            width: w,
                            width_type: w_type,
                        });
                    }
                    "jc" => tbl_pr.justification = get_attr_val(e, "val"),
                    _ => tbl_pr.raw_elements.push(parse_empty_raw_node(e)?),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tblPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(tbl_pr)
}

fn parse_table_grid<R: std::io::BufRead>(reader: &mut Reader<R>) -> Result<TableGrid> {
    let mut grid = TableGrid::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Empty(ref e) if local_name(e.name().into_inner()) == "gridCol" => {
                let w = get_attr_val(e, "w")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
                grid.columns.push(w);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tblGrid" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(grid)
}

fn parse_table_row<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<TableRow> {
    let mut row = TableRow::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match local_name(key.as_bytes()) {
            "rsidR" => row.rsid_r = Some(val),
            "rsidTr" => row.rsid_tr = Some(val),
            _ => row.raw_attrs.push((key, val)),
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) if local_name(e.name().into_inner()) == "tc" => {
                row.cells.push(parse_table_cell(reader, e)?);
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(row)
}

fn parse_empty_table_row(start_tag: &BytesStart) -> Result<TableRow> {
    let mut row = TableRow::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        row.raw_attrs.push((key, val));
    }
    Ok(row)
}

fn parse_table_cell_properties<R: std::io::BufRead>(
    reader: &mut Reader<R>,
) -> Result<Option<TableWidth>> {
    let mut width = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) | Event::Empty(ref e)
                if local_name(e.name().into_inner()) == "tcW" =>
            {
                let w = get_attr_val(e, "w")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
                let w_type = get_attr_val(e, "type").unwrap_or_else(|| "dxa".to_string());
                width = Some(TableWidth {
                    width: w,
                    width_type: w_type,
                });
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tcPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(width)
}

fn parse_table_cell<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<TableCell> {
    let mut cell = TableCell::default();
    for attr in start_tag.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        cell.raw_attrs.push((key, val));
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "tcPr" => {
                        cell.width = parse_table_cell_properties(reader)?;
                    }
                    "p" => cell
                        .elements
                        .push(BodyElement::Paragraph(parse_paragraph(reader, e)?)),
                    "tbl" => cell
                        .elements
                        .push(BodyElement::Table(parse_table(reader, e)?)),
                    _ => cell
                        .elements
                        .push(BodyElement::Raw(parse_raw_xml_node(reader, e)?)),
                }
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                if name == "p" {
                    cell.elements
                        .push(BodyElement::Paragraph(parse_empty_paragraph(e)?));
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "tc" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(cell)
}

// --- Section Properties Parsing ---

fn parse_section_properties<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    _start_tag: &BytesStart,
) -> Result<SectionProperties> {
    let mut sect_pr = SectionProperties::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                sect_pr.raw_elements.push(parse_raw_xml_node(reader, e)?);
            }
            Event::Empty(ref e) => {
                let name = local_name(e.name().into_inner());
                match name {
                    "headerReference" => {
                        let ref_type =
                            get_attr_val(e, "type").unwrap_or_else(|| "default".to_string());
                        let rel_id = get_attr_val(e, "id").unwrap_or_default();
                        sect_pr
                            .header_refs
                            .push(HeaderFooterRef { ref_type, rel_id });
                    }
                    "footerReference" => {
                        let ref_type =
                            get_attr_val(e, "type").unwrap_or_else(|| "default".to_string());
                        let rel_id = get_attr_val(e, "id").unwrap_or_default();
                        sect_pr
                            .footer_refs
                            .push(HeaderFooterRef { ref_type, rel_id });
                    }
                    "pgSz" => {
                        let width = get_attr_val(e, "w")
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);
                        let height = get_attr_val(e, "h")
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);
                        let orientation = get_attr_val(e, "orient");
                        sect_pr.page_size = Some(PageSize {
                            width,
                            height,
                            orientation,
                        });
                    }
                    "pgMar" => {
                        sect_pr.page_margin = Some(PageMargin {
                            top: get_attr_val(e, "top")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            right: get_attr_val(e, "right")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            bottom: get_attr_val(e, "bottom")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            left: get_attr_val(e, "left")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            header: get_attr_val(e, "header")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            footer: get_attr_val(e, "footer")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                            gutter: get_attr_val(e, "gutter")
                                .and_then(|s| s.parse::<i32>().ok())
                                .unwrap_or(0),
                        });
                    }
                    "cols" => {
                        sect_pr.columns = Some(Columns {
                            num: get_attr_val(e, "num").and_then(|s| s.parse::<u32>().ok()),
                            space: get_attr_val(e, "space").and_then(|s| s.parse::<u32>().ok()),
                        });
                    }
                    "docGrid" => {
                        sect_pr.doc_grid = Some(DocGrid {
                            line_pitch: get_attr_val(e, "linePitch")
                                .and_then(|s| s.parse::<u32>().ok()),
                        });
                    }
                    _ => sect_pr.raw_elements.push(parse_empty_raw_node(e)?),
                }
            }
            Event::End(ref e) if local_name(e.name().into_inner()) == "sectPr" => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(sect_pr)
}

fn parse_empty_section_properties(_e: &BytesStart) -> Result<SectionProperties> {
    Ok(SectionProperties::default())
}

// --- Raw / Unmodeled XML Node Preservation ---

pub fn parse_raw_xml_node<R: std::io::BufRead>(
    reader: &mut Reader<R>,
    start_tag: &BytesStart,
) -> Result<RawXmlNode> {
    let tag = String::from_utf8_lossy(start_tag.name().as_ref()).to_string();
    let mut node = RawXmlNode::new(tag);

    for attr in start_tag.attributes().flatten() {
        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let v = String::from_utf8_lossy(&attr.value).to_string();
        node.attributes.push((k, v));
    }

    let end_target = node.tag.clone();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(EngineError::Xml)? {
            Event::Start(ref e) => {
                node.children.push(parse_raw_xml_node(reader, e)?);
            }
            Event::Empty(ref e) => {
                node.children.push(parse_empty_raw_node(e)?);
            }
            Event::Text(ref t) => {
                let s = t.unescape().map_err(EngineError::Xml)?.to_string();
                node.content = Some(s);
            }
            Event::End(ref e) => {
                let e_name = String::from_utf8_lossy(e.name().into_inner());
                if e_name == end_target {
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(node)
}

pub fn parse_empty_raw_node(e: &BytesStart) -> Result<RawXmlNode> {
    let tag = String::from_utf8_lossy(e.name().into_inner()).to_string();
    let mut node = RawXmlNode::new(tag);
    for attr in e.attributes().flatten() {
        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let v = String::from_utf8_lossy(&attr.value).to_string();
        node.attributes.push((k, v));
    }
    Ok(node)
}

// --- Utilities ---

#[inline]
fn local_name(qname: &[u8]) -> &str {
    if let Some(pos) = qname.iter().rposition(|&b| b == b':') {
        std::str::from_utf8(&qname[pos + 1..]).unwrap_or("")
    } else {
        std::str::from_utf8(qname).unwrap_or("")
    }
}

fn get_attr_val(e: &BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if local_name(key) == name {
            return Some(String::from_utf8_lossy(&attr.value).to_string());
        }
    }
    None
}

fn parse_boolean_attr(e: &BytesStart) -> bool {
    if let Some(val) = get_attr_val(e, "val") {
        val != "0" && val != "false" && val != "off"
    } else {
        true
    }
}

fn is_xml_space_preserve(e: &BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref());
        if key == "xml:space" {
            let val = String::from_utf8_lossy(&attr.value);
            return val == "preserve";
        }
    }
    false
}

fn extract_namespaces_and_attributes(
    e: &BytesStart,
    namespaces: &mut Vec<(String, String)>,
    ignorable: &mut Option<String>,
) -> Result<()> {
    for attr in e.attributes().flatten() {
        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let v = String::from_utf8_lossy(&attr.value).to_string();
        if k.starts_with("xmlns") {
            namespaces.push((k, v));
        } else if k == "mc:Ignorable" {
            *ignorable = Some(v);
        } else {
            namespaces.push((k, v));
        }
    }
    Ok(())
}
