//! OpenOfficeRS Word Engine - Lossless WordprocessingML Serializer.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).
//! Serializes WordprocessingML documents, headers, footers, footnotes, endnotes,
//! and comments to clean, deterministic ECMA-376 XML.

use crate::model::*;
use crate::sub_documents::*;
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::io::Cursor;

/// Serializes a full `word/document.xml` part.
pub fn serialize_document(doc: &Document) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut doc_tag = BytesStart::new("w:document");
    for (k, v) in &doc.namespaces {
        doc_tag.push_attribute((k.as_str(), v.as_str()));
    }
    if let Some(ref ignorable) = doc.ignorable {
        doc_tag.push_attribute(("mc:Ignorable", ignorable.as_str()));
    }

    writer
        .write_event(Event::Start(doc_tag))
        .map_err(EngineError::Io)?;

    serialize_body(&mut writer, &doc.body)?;

    for node in &doc.unparsed_nodes {
        serialize_raw_node(&mut writer, node)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:document")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

/// Serializes a header part: `<w:hdr>`.
pub fn serialize_header(header: &HeaderDocument) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut hdr_tag = BytesStart::new("w:hdr");
    for (k, v) in &header.namespaces {
        hdr_tag.push_attribute((k.as_str(), v.as_str()));
    }

    writer
        .write_event(Event::Start(hdr_tag))
        .map_err(EngineError::Io)?;

    for elem in &header.body.elements {
        serialize_body_element(&mut writer, elem)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:hdr")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

/// Serializes a footer part: `<w:ftr>`.
pub fn serialize_footer(footer: &FooterDocument) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut ftr_tag = BytesStart::new("w:ftr");
    for (k, v) in &footer.namespaces {
        ftr_tag.push_attribute((k.as_str(), v.as_str()));
    }

    writer
        .write_event(Event::Start(ftr_tag))
        .map_err(EngineError::Io)?;

    for elem in &footer.body.elements {
        serialize_body_element(&mut writer, elem)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:ftr")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

/// Serializes a footnotes part: `<w:footnotes>`.
pub fn serialize_footnotes(footnotes: &FootnotesDocument) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut fn_tag = BytesStart::new("w:footnotes");
    for (k, v) in &footnotes.namespaces {
        fn_tag.push_attribute((k.as_str(), v.as_str()));
    }

    writer
        .write_event(Event::Start(fn_tag))
        .map_err(EngineError::Io)?;

    for f in &footnotes.footnotes {
        let mut f_tag = BytesStart::new("w:footnote");
        let id_str = f.id.to_string();
        f_tag.push_attribute(("w:id", id_str.as_str()));
        if let Some(ref fn_type) = f.footnote_type {
            f_tag.push_attribute(("w:type", fn_type.as_str()));
        }

        writer
            .write_event(Event::Start(f_tag))
            .map_err(EngineError::Io)?;

        for elem in &f.body.elements {
            serialize_body_element(&mut writer, elem)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("w:footnote")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:footnotes")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

/// Serializes an endnotes part: `<w:endnotes>`.
pub fn serialize_endnotes(endnotes: &EndnotesDocument) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut en_tag = BytesStart::new("w:endnotes");
    for (k, v) in &endnotes.namespaces {
        en_tag.push_attribute((k.as_str(), v.as_str()));
    }

    writer
        .write_event(Event::Start(en_tag))
        .map_err(EngineError::Io)?;

    for e in &endnotes.endnotes {
        let mut e_tag = BytesStart::new("w:endnote");
        let id_str = e.id.to_string();
        e_tag.push_attribute(("w:id", id_str.as_str()));
        if let Some(ref en_type) = e.endnote_type {
            e_tag.push_attribute(("w:type", en_type.as_str()));
        }

        writer
            .write_event(Event::Start(e_tag))
            .map_err(EngineError::Io)?;

        for elem in &e.body.elements {
            serialize_body_element(&mut writer, elem)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("w:endnote")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:endnotes")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

/// Serializes a comments part: `<w:comments>`.
pub fn serialize_comments(comments: &CommentsDocument) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(EngineError::Io)?;

    let mut cm_tag = BytesStart::new("w:comments");
    for (k, v) in &comments.namespaces {
        cm_tag.push_attribute((k.as_str(), v.as_str()));
    }

    writer
        .write_event(Event::Start(cm_tag))
        .map_err(EngineError::Io)?;

    for c in &comments.comments {
        let mut c_tag = BytesStart::new("w:comment");
        c_tag.push_attribute(("w:id", c.id.as_str()));
        c_tag.push_attribute(("w:author", c.author.as_str()));
        if let Some(ref initials) = c.initials {
            c_tag.push_attribute(("w:initials", initials.as_str()));
        }
        if let Some(ref date) = c.date {
            c_tag.push_attribute(("w:date", date.as_str()));
        }

        writer
            .write_event(Event::Start(c_tag))
            .map_err(EngineError::Io)?;

        for elem in &c.body.elements {
            serialize_body_element(&mut writer, elem)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("w:comment")))
            .map_err(EngineError::Io)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:comments")))
        .map_err(EngineError::Io)?;

    Ok(writer.into_inner().into_inner())
}

// --- Body Serialization ---

fn serialize_body<W: std::io::Write>(writer: &mut Writer<W>, body: &Body) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("w:body")))
        .map_err(EngineError::Io)?;

    for elem in &body.elements {
        serialize_body_element(writer, elem)?;
    }

    if let Some(ref sect_pr) = body.sect_pr {
        serialize_section_properties(writer, sect_pr)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:body")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_body_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    elem: &BodyElement,
) -> Result<()> {
    match elem {
        BodyElement::Paragraph(p) => serialize_paragraph(writer, p),
        BodyElement::Table(tbl) => serialize_table(writer, tbl),
        BodyElement::SectionProperties(sp) => serialize_section_properties(writer, sp),
        BodyElement::Raw(raw) => serialize_raw_node(writer, raw),
    }
}

// --- Paragraph Serialization ---

fn serialize_paragraph<W: std::io::Write>(writer: &mut Writer<W>, para: &Paragraph) -> Result<()> {
    let mut p_tag = BytesStart::new("w:p");
    if let Some(ref pid) = para.para_id {
        p_tag.push_attribute(("w14:paraId", pid.as_str()));
    }
    if let Some(ref tid) = para.text_id {
        p_tag.push_attribute(("w14:textId", tid.as_str()));
    }
    if let Some(ref rsid) = para.rsid_r {
        p_tag.push_attribute(("w:rsidR", rsid.as_str()));
    }
    if let Some(ref rsid_def) = para.rsid_r_default {
        p_tag.push_attribute(("w:rsidRDefault", rsid_def.as_str()));
    }
    for (k, v) in &para.raw_attrs {
        p_tag.push_attribute((k.as_str(), v.as_str()));
    }

    if para.properties.is_none() && para.children.is_empty() {
        writer
            .write_event(Event::Empty(p_tag))
            .map_err(EngineError::Io)?;
        return Ok(());
    }

    writer
        .write_event(Event::Start(p_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref p_pr) = para.properties {
        serialize_paragraph_properties(writer, p_pr)?;
    }

    for child in &para.children {
        serialize_paragraph_child(writer, child)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:p")))
        .map_err(EngineError::Io)?;

    Ok(())
}

pub(crate) fn serialize_paragraph_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    p_pr: &ParagraphProperties,
) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("w:pPr")))
        .map_err(EngineError::Io)?;

    if let Some(ref style) = p_pr.style {
        let mut st = BytesStart::new("w:pStyle");
        st.push_attribute(("w:val", style.as_str()));
        writer
            .write_event(Event::Empty(st))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref jc) = p_pr.justification {
        let mut jt = BytesStart::new("w:jc");
        jt.push_attribute(("w:val", jc.as_str()));
        writer
            .write_event(Event::Empty(jt))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref sp) = p_pr.spacing {
        let mut sp_tag = BytesStart::new("w:spacing");
        if let Some(b) = sp.before {
            let s = b.to_string();
            sp_tag.push_attribute(("w:before", s.as_str()));
        }
        if let Some(a) = sp.after {
            let s = a.to_string();
            sp_tag.push_attribute(("w:after", s.as_str()));
        }
        if let Some(l) = sp.line {
            let s = l.to_string();
            sp_tag.push_attribute(("w:line", s.as_str()));
        }
        if let Some(ref lr) = sp.line_rule {
            sp_tag.push_attribute(("w:lineRule", lr.as_str()));
        }
        writer
            .write_event(Event::Empty(sp_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref ind) = p_pr.indentation {
        let mut ind_tag = BytesStart::new("w:ind");
        if let Some(l) = ind.left {
            let s = l.to_string();
            ind_tag.push_attribute(("w:left", s.as_str()));
        }
        if let Some(r) = ind.right {
            let s = r.to_string();
            ind_tag.push_attribute(("w:right", s.as_str()));
        }
        if let Some(fl) = ind.first_line {
            let s = fl.to_string();
            ind_tag.push_attribute(("w:firstLine", s.as_str()));
        }
        if let Some(h) = ind.hanging {
            let s = h.to_string();
            ind_tag.push_attribute(("w:hanging", s.as_str()));
        }
        writer
            .write_event(Event::Empty(ind_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref num) = p_pr.num_pr {
        writer
            .write_event(Event::Start(BytesStart::new("w:numPr")))
            .map_err(EngineError::Io)?;
        let mut ilvl_tag = BytesStart::new("w:ilvl");
        let ilvl_str = num.ilvl.to_string();
        ilvl_tag.push_attribute(("w:val", ilvl_str.as_str()));
        writer
            .write_event(Event::Empty(ilvl_tag))
            .map_err(EngineError::Io)?;

        let mut num_tag = BytesStart::new("w:numId");
        let num_str = num.num_id.to_string();
        num_tag.push_attribute(("w:val", num_str.as_str()));
        writer
            .write_event(Event::Empty(num_tag))
            .map_err(EngineError::Io)?;

        writer
            .write_event(Event::End(BytesEnd::new("w:numPr")))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref r_pr) = p_pr.r_pr {
        serialize_run_properties(writer, r_pr)?;
    }

    if let Some(wc) = p_pr.widow_control {
        let mut wc_tag = BytesStart::new("w:widowControl");
        if !wc {
            wc_tag.push_attribute(("w:val", "0"));
        }
        writer
            .write_event(Event::Empty(wc_tag))
            .map_err(EngineError::Io)?;
    }

    if p_pr.keep_next {
        writer
            .write_event(Event::Empty(BytesStart::new("w:keepNext")))
            .map_err(EngineError::Io)?;
    }

    if p_pr.keep_lines {
        writer
            .write_event(Event::Empty(BytesStart::new("w:keepLines")))
            .map_err(EngineError::Io)?;
    }

    if p_pr.page_break_before {
        writer
            .write_event(Event::Empty(BytesStart::new("w:pageBreakBefore")))
            .map_err(EngineError::Io)?;
    }

    if let Some(k) = p_pr.kinsoku {
        let mut k_tag = BytesStart::new("w:kinsoku");
        if !k {
            k_tag.push_attribute(("w:val", "0"));
        }
        writer
            .write_event(Event::Empty(k_tag))
            .map_err(EngineError::Io)?;
    }

    for raw in &p_pr.raw_elements {
        serialize_raw_node(writer, raw)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:pPr")))
        .map_err(EngineError::Io)?;

    Ok(())
}

fn serialize_paragraph_child<W: std::io::Write>(
    writer: &mut Writer<W>,
    child: &ParagraphChild,
) -> Result<()> {
    match child {
        ParagraphChild::Run(r) => serialize_run(writer, r),
        ParagraphChild::BookmarkStart(bm) => {
            let mut bm_tag = BytesStart::new("w:bookmarkStart");
            bm_tag.push_attribute(("w:id", bm.id.as_str()));
            bm_tag.push_attribute(("w:name", bm.name.as_str()));
            if let Some(cf) = bm.col_first {
                let s = cf.to_string();
                bm_tag.push_attribute(("w:colFirst", s.as_str()));
            }
            if let Some(cl) = bm.col_last {
                let s = cl.to_string();
                bm_tag.push_attribute(("w:colLast", s.as_str()));
            }
            writer
                .write_event(Event::Empty(bm_tag))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::BookmarkEnd(bme) => {
            let mut bme_tag = BytesStart::new("w:bookmarkEnd");
            bme_tag.push_attribute(("w:id", bme.id.as_str()));
            writer
                .write_event(Event::Empty(bme_tag))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::Hyperlink(h) => {
            let mut h_tag = BytesStart::new("w:hyperlink");
            if let Some(ref id) = h.id {
                h_tag.push_attribute(("r:id", id.as_str()));
            }
            if let Some(ref anchor) = h.anchor {
                h_tag.push_attribute(("w:anchor", anchor.as_str()));
            }
            if let Some(ref tooltip) = h.tooltip {
                h_tag.push_attribute(("w:tooltip", tooltip.as_str()));
            }
            if h.history {
                h_tag.push_attribute(("w:history", "1"));
            }
            writer
                .write_event(Event::Start(h_tag))
                .map_err(EngineError::Io)?;
            for c in &h.children {
                serialize_paragraph_child(writer, c)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("w:hyperlink")))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::Inserted(tc) => {
            let mut ins_tag = BytesStart::new("w:ins");
            ins_tag.push_attribute(("w:id", tc.id.as_str()));
            ins_tag.push_attribute(("w:author", tc.author.as_str()));
            ins_tag.push_attribute(("w:date", tc.date.as_str()));
            writer
                .write_event(Event::Start(ins_tag))
                .map_err(EngineError::Io)?;
            for c in &tc.children {
                serialize_paragraph_child(writer, c)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("w:ins")))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::Deleted(tc) => {
            let mut del_tag = BytesStart::new("w:del");
            del_tag.push_attribute(("w:id", tc.id.as_str()));
            del_tag.push_attribute(("w:author", tc.author.as_str()));
            del_tag.push_attribute(("w:date", tc.date.as_str()));
            writer
                .write_event(Event::Start(del_tag))
                .map_err(EngineError::Io)?;
            for c in &tc.children {
                serialize_paragraph_child(writer, c)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("w:del")))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::CommentRangeStart(id) => {
            let mut t = BytesStart::new("w:commentRangeStart");
            t.push_attribute(("w:id", id.as_str()));
            writer
                .write_event(Event::Empty(t))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::CommentRangeEnd(id) => {
            let mut t = BytesStart::new("w:commentRangeEnd");
            t.push_attribute(("w:id", id.as_str()));
            writer
                .write_event(Event::Empty(t))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::CommentReference(id) => {
            let mut t = BytesStart::new("w:commentReference");
            t.push_attribute(("w:id", id.as_str()));
            writer
                .write_event(Event::Empty(t))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::FootnoteReference(id) => {
            let mut t = BytesStart::new("w:footnoteReference");
            t.push_attribute(("w:id", id.as_str()));
            writer
                .write_event(Event::Empty(t))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::EndnoteReference(id) => {
            let mut t = BytesStart::new("w:endnoteReference");
            t.push_attribute(("w:id", id.as_str()));
            writer
                .write_event(Event::Empty(t))
                .map_err(EngineError::Io)?;
            Ok(())
        }
        ParagraphChild::Raw(raw) => serialize_raw_node(writer, raw),
    }
}

// --- Run Serialization ---

fn serialize_run<W: std::io::Write>(writer: &mut Writer<W>, run: &Run) -> Result<()> {
    let mut r_tag = BytesStart::new("w:r");
    if let Some(ref rsid_pr) = run.rsid_r_pr {
        r_tag.push_attribute(("w:rsidRPr", rsid_pr.as_str()));
    }
    if let Some(ref rsid) = run.rsid_r {
        r_tag.push_attribute(("w:rsidR", rsid.as_str()));
    }
    for (k, v) in &run.raw_attrs {
        r_tag.push_attribute((k.as_str(), v.as_str()));
    }

    if run.properties.is_none() && run.content.is_empty() {
        writer
            .write_event(Event::Empty(r_tag))
            .map_err(EngineError::Io)?;
        return Ok(());
    }

    writer
        .write_event(Event::Start(r_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref r_pr) = run.properties {
        serialize_run_properties(writer, r_pr)?;
    }

    for item in &run.content {
        match item {
            RunContent::Text(text, preserve) => {
                let mut t_tag = BytesStart::new("w:t");
                if *preserve {
                    t_tag.push_attribute(("xml:space", "preserve"));
                }
                writer
                    .write_event(Event::Start(t_tag))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::Text(BytesText::new(text)))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::End(BytesEnd::new("w:t")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::DeletedText(text, preserve) => {
                let mut dt_tag = BytesStart::new("w:delText");
                if *preserve {
                    dt_tag.push_attribute(("xml:space", "preserve"));
                }
                writer
                    .write_event(Event::Start(dt_tag))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::Text(BytesText::new(text)))
                    .map_err(EngineError::Io)?;
                writer
                    .write_event(Event::End(BytesEnd::new("w:delText")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::Break(br_type) => {
                let mut b_tag = BytesStart::new("w:br");
                if let Some(ref t) = br_type {
                    b_tag.push_attribute(("w:type", t.as_str()));
                }
                writer
                    .write_event(Event::Empty(b_tag))
                    .map_err(EngineError::Io)?;
            }
            RunContent::Tab => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:tab")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::SoftHyphen => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:softHyphen")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::NoBreakHyphen => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:noBreakHyphen")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::Separator => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:separator")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::ContinuationSeparator => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:continuationSeparator")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::FootnoteRef => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:footnoteRef")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::EndnoteRef => {
                writer
                    .write_event(Event::Empty(BytesStart::new("w:endnoteRef")))
                    .map_err(EngineError::Io)?;
            }
            RunContent::Raw(raw) => serialize_raw_node(writer, raw)?,
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:r")))
        .map_err(EngineError::Io)?;
    Ok(())
}

pub(crate) fn serialize_run_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    r_pr: &RunProperties,
) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("w:rPr")))
        .map_err(EngineError::Io)?;

    if r_pr.font_ascii.is_some()
        || r_pr.font_h_ansi.is_some()
        || r_pr.font_east_asia.is_some()
        || r_pr.font_cs.is_some()
    {
        let mut f_tag = BytesStart::new("w:rFonts");
        if let Some(ref a) = r_pr.font_ascii {
            f_tag.push_attribute(("w:ascii", a.as_str()));
        }
        if let Some(ref h) = r_pr.font_h_ansi {
            f_tag.push_attribute(("w:hAnsi", h.as_str()));
        }
        if let Some(ref ea) = r_pr.font_east_asia {
            f_tag.push_attribute(("w:eastAsia", ea.as_str()));
        }
        if let Some(ref cs) = r_pr.font_cs {
            f_tag.push_attribute(("w:cs", cs.as_str()));
        }
        writer
            .write_event(Event::Empty(f_tag))
            .map_err(EngineError::Io)?;
    }

    if r_pr.bold {
        writer
            .write_event(Event::Empty(BytesStart::new("w:b")))
            .map_err(EngineError::Io)?;
    }
    if r_pr.italic {
        writer
            .write_event(Event::Empty(BytesStart::new("w:i")))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref u) = r_pr.underline {
        let mut u_tag = BytesStart::new("w:u");
        u_tag.push_attribute(("w:val", u.as_str()));
        writer
            .write_event(Event::Empty(u_tag))
            .map_err(EngineError::Io)?;
    }
    if r_pr.strike {
        writer
            .write_event(Event::Empty(BytesStart::new("w:strike")))
            .map_err(EngineError::Io)?;
    }
    if r_pr.double_strike {
        writer
            .write_event(Event::Empty(BytesStart::new("w:dstrike")))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref color) = r_pr.color {
        let mut c_tag = BytesStart::new("w:color");
        c_tag.push_attribute(("w:val", color.as_str()));
        writer
            .write_event(Event::Empty(c_tag))
            .map_err(EngineError::Io)?;
    }
    if let Some(sz) = r_pr.size_half_points {
        let mut s_tag = BytesStart::new("w:sz");
        let s = sz.to_string();
        s_tag.push_attribute(("w:val", s.as_str()));
        writer
            .write_event(Event::Empty(s_tag))
            .map_err(EngineError::Io)?;
    }
    if let Some(ref hl) = r_pr.highlight {
        let mut h_tag = BytesStart::new("w:highlight");
        h_tag.push_attribute(("w:val", hl.as_str()));
        writer
            .write_event(Event::Empty(h_tag))
            .map_err(EngineError::Io)?;
    }

    for raw in &r_pr.raw_elements {
        serialize_raw_node(writer, raw)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:rPr")))
        .map_err(EngineError::Io)?;

    Ok(())
}

// --- Table Serialization ---

fn serialize_table<W: std::io::Write>(writer: &mut Writer<W>, tbl: &Table) -> Result<()> {
    let mut tbl_tag = BytesStart::new("w:tbl");
    for (k, v) in &tbl.raw_attrs {
        tbl_tag.push_attribute((k.as_str(), v.as_str()));
    }
    writer
        .write_event(Event::Start(tbl_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref tbl_pr) = tbl.properties {
        writer
            .write_event(Event::Start(BytesStart::new("w:tblPr")))
            .map_err(EngineError::Io)?;
        if let Some(ref width) = tbl_pr.width {
            let mut w_tag = BytesStart::new("w:tblW");
            let w_str = width.width.to_string();
            w_tag.push_attribute(("w:w", w_str.as_str()));
            w_tag.push_attribute(("w:type", width.width_type.as_str()));
            writer
                .write_event(Event::Empty(w_tag))
                .map_err(EngineError::Io)?;
        }
        if let Some(ref jc) = tbl_pr.justification {
            let mut j_tag = BytesStart::new("w:jc");
            j_tag.push_attribute(("w:val", jc.as_str()));
            writer
                .write_event(Event::Empty(j_tag))
                .map_err(EngineError::Io)?;
        }
        for raw in &tbl_pr.raw_elements {
            serialize_raw_node(writer, raw)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("w:tblPr")))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref grid) = tbl.grid {
        writer
            .write_event(Event::Start(BytesStart::new("w:tblGrid")))
            .map_err(EngineError::Io)?;
        for col_w in &grid.columns {
            let mut c_tag = BytesStart::new("w:gridCol");
            let s = col_w.to_string();
            c_tag.push_attribute(("w:w", s.as_str()));
            writer
                .write_event(Event::Empty(c_tag))
                .map_err(EngineError::Io)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("w:tblGrid")))
            .map_err(EngineError::Io)?;
    }

    for row in &tbl.rows {
        serialize_table_row(writer, row)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:tbl")))
        .map_err(EngineError::Io)?;
    Ok(())
}

fn serialize_table_row<W: std::io::Write>(writer: &mut Writer<W>, row: &TableRow) -> Result<()> {
    let mut tr_tag = BytesStart::new("w:tr");
    if let Some(ref rsid) = row.rsid_r {
        tr_tag.push_attribute(("w:rsidR", rsid.as_str()));
    }
    if let Some(ref rsid_tr) = row.rsid_tr {
        tr_tag.push_attribute(("w:rsidTr", rsid_tr.as_str()));
    }
    for (k, v) in &row.raw_attrs {
        tr_tag.push_attribute((k.as_str(), v.as_str()));
    }
    writer
        .write_event(Event::Start(tr_tag))
        .map_err(EngineError::Io)?;

    for cell in &row.cells {
        serialize_table_cell(writer, cell)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:tr")))
        .map_err(EngineError::Io)?;
    Ok(())
}

fn serialize_table_cell<W: std::io::Write>(writer: &mut Writer<W>, cell: &TableCell) -> Result<()> {
    let mut tc_tag = BytesStart::new("w:tc");
    for (k, v) in &cell.raw_attrs {
        tc_tag.push_attribute((k.as_str(), v.as_str()));
    }
    writer
        .write_event(Event::Start(tc_tag))
        .map_err(EngineError::Io)?;

    if let Some(ref width) = cell.width {
        writer
            .write_event(Event::Start(BytesStart::new("w:tcPr")))
            .map_err(EngineError::Io)?;
        let mut w_tag = BytesStart::new("w:tcW");
        let w_str = width.width.to_string();
        w_tag.push_attribute(("w:w", w_str.as_str()));
        w_tag.push_attribute(("w:type", width.width_type.as_str()));
        writer
            .write_event(Event::Empty(w_tag))
            .map_err(EngineError::Io)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:tcPr")))
            .map_err(EngineError::Io)?;
    }

    for elem in &cell.elements {
        serialize_body_element(writer, elem)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:tc")))
        .map_err(EngineError::Io)?;
    Ok(())
}

// --- Section Properties Serialization ---

fn serialize_section_properties<W: std::io::Write>(
    writer: &mut Writer<W>,
    sect_pr: &SectionProperties,
) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("w:sectPr")))
        .map_err(EngineError::Io)?;

    for hr in &sect_pr.header_refs {
        let mut h_tag = BytesStart::new("w:headerReference");
        h_tag.push_attribute(("w:type", hr.ref_type.as_str()));
        h_tag.push_attribute(("r:id", hr.rel_id.as_str()));
        writer
            .write_event(Event::Empty(h_tag))
            .map_err(EngineError::Io)?;
    }

    for fr in &sect_pr.footer_refs {
        let mut f_tag = BytesStart::new("w:footerReference");
        f_tag.push_attribute(("w:type", fr.ref_type.as_str()));
        f_tag.push_attribute(("r:id", fr.rel_id.as_str()));
        writer
            .write_event(Event::Empty(f_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref sz) = sect_pr.page_size {
        let mut sz_tag = BytesStart::new("w:pgSz");
        let w_str = sz.width.to_string();
        let h_str = sz.height.to_string();
        sz_tag.push_attribute(("w:w", w_str.as_str()));
        sz_tag.push_attribute(("w:h", h_str.as_str()));
        if let Some(ref orient) = sz.orientation {
            sz_tag.push_attribute(("w:orient", orient.as_str()));
        }
        writer
            .write_event(Event::Empty(sz_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref mar) = sect_pr.page_margin {
        let mut m_tag = BytesStart::new("w:pgMar");
        let top = mar.top.to_string();
        let right = mar.right.to_string();
        let bottom = mar.bottom.to_string();
        let left = mar.left.to_string();
        let header = mar.header.to_string();
        let footer = mar.footer.to_string();
        let gutter = mar.gutter.to_string();

        m_tag.push_attribute(("w:top", top.as_str()));
        m_tag.push_attribute(("w:right", right.as_str()));
        m_tag.push_attribute(("w:bottom", bottom.as_str()));
        m_tag.push_attribute(("w:left", left.as_str()));
        m_tag.push_attribute(("w:header", header.as_str()));
        m_tag.push_attribute(("w:footer", footer.as_str()));
        m_tag.push_attribute(("w:gutter", gutter.as_str()));
        writer
            .write_event(Event::Empty(m_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref cols) = sect_pr.columns {
        let mut col_tag = BytesStart::new("w:cols");
        if let Some(n) = cols.num {
            let s = n.to_string();
            col_tag.push_attribute(("w:num", s.as_str()));
        }
        if let Some(sp) = cols.space {
            let s = sp.to_string();
            col_tag.push_attribute(("w:space", s.as_str()));
        }
        writer
            .write_event(Event::Empty(col_tag))
            .map_err(EngineError::Io)?;
    }

    if let Some(ref dg) = sect_pr.doc_grid {
        let mut dg_tag = BytesStart::new("w:docGrid");
        if let Some(lp) = dg.line_pitch {
            let s = lp.to_string();
            dg_tag.push_attribute(("w:linePitch", s.as_str()));
        }
        writer
            .write_event(Event::Empty(dg_tag))
            .map_err(EngineError::Io)?;
    }

    for raw in &sect_pr.raw_elements {
        serialize_raw_node(writer, raw)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:sectPr")))
        .map_err(EngineError::Io)?;

    Ok(())
}

// --- Raw Node Serialization ---

fn serialize_raw_node<W: std::io::Write>(writer: &mut Writer<W>, node: &RawXmlNode) -> Result<()> {
    let mut tag = BytesStart::new(&node.tag);
    for (k, v) in &node.attributes {
        tag.push_attribute((k.as_str(), v.as_str()));
    }

    if node.children.is_empty() && node.content.is_none() {
        writer
            .write_event(Event::Empty(tag))
            .map_err(EngineError::Io)?;
        return Ok(());
    }

    writer
        .write_event(Event::Start(tag))
        .map_err(EngineError::Io)?;

    if let Some(ref c) = node.content {
        writer
            .write_event(Event::Text(BytesText::new(c)))
            .map_err(EngineError::Io)?;
    }

    for child in &node.children {
        serialize_raw_node(writer, child)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new(&node.tag)))
        .map_err(EngineError::Io)?;

    Ok(())
}
