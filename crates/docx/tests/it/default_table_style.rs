//! #5056: a table without `w:tblStyle` is formatted with the styles part's default table style.
//! Word for Windows, opening such a file, reports that style as the table's `Style` and draws its
//! header row part: a bottom border and bottom-aligned cells.
use std::io::Write;
use wordcraft_doc::props::VAlign;
use wordcraft_doc::styles::TablePart;

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn package(styles: &str, body: &str) -> Vec<u8> {
    let root = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let doc = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{W}"><w:body>{body}</w:body></w:document>"#);
    let styles = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="{W}">{styles}</w:styles>"#);
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, body) in
        [("_rels/.rels", root), ("word/document.xml", doc.as_str()), ("word/_rels/document.xml.rels", rels), ("word/styles.xml", styles.as_str())]
    {
        zw.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        zw.write_all(body.as_bytes()).unwrap();
    }
    zw.finish().unwrap().into_inner()
}

/// A table style whose header row part draws a bottom rule and bottom-aligns its cells.
fn ruled(id: &str, default: &str) -> String {
    format!(
        r#"<w:style w:type="table" w:default="{default}" w:styleId="{id}"><w:name w:val="{id}"/><w:tblStylePr w:type="firstRow"><w:tcPr><w:vAlign w:val="bottom"/><w:tcBorders><w:bottom w:val="single" w:sz="4"/></w:tcBorders></w:tcPr></w:tblStylePr></w:style>"#
    )
}

/// One row, two cells, its own borders all off: what an exam header table looks like.
fn table(tbl_style: &str) -> String {
    let nil = ["top", "left", "bottom", "right", "insideH", "insideV"].map(|e| format!(r#"<w:{e} w:val="nil"/>"#)).concat();
    format!(
        r#"<w:tbl><w:tblPr>{tbl_style}<w:tblLook w:firstRow="1" w:noVBand="1"/><w:tblBorders>{nil}</w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Left</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Right</w:t></w:r></w:p><w:p><w:r><w:t>Code</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#
    )
}

fn style_of(d: &wordcraft_doc::Document) -> Option<String> {
    d.body[0].as_table().unwrap().props.style.clone()
}

#[test]
fn a_table_without_a_style_takes_the_default_table_style() {
    let d = wordcraft_docx::read(&package(&ruled("Rule", "1"), &table(""))).unwrap();
    assert_eq!(style_of(&d).as_deref(), Some("Rule"));
    let head = d.styles.table_style("Rule").unwrap().part(TablePart::FirstRow);
    assert_eq!(head.v_align, Some(VAlign::Bottom));
    assert!(head.borders.and_then(|b| b.bottom).is_some_and(|b| b.width > 0.0), "{head:?}");

    // Saved, the table names the style; reopened, it still has it.
    let back = wordcraft_docx::read(&wordcraft_docx::write(&d).unwrap()).unwrap();
    assert_eq!(style_of(&back).as_deref(), Some("Rule"));
}

#[test]
fn a_named_style_wins_over_the_default() {
    let styles = format!("{}{}", ruled("Rule", "1"), ruled("Other", "0"));
    let d = wordcraft_docx::read(&package(&styles, &table(r#"<w:tblStyle w:val="Other"/>"#))).unwrap();
    assert_eq!(style_of(&d).as_deref(), Some("Other"));
}

#[test]
fn the_default_flag_reads_as_on_off() {
    for (flag, want) in [("true", Some("Rule")), ("on", Some("Rule")), ("0", None), ("false", None)] {
        let d = wordcraft_docx::read(&package(&ruled("Rule", flag), &table(""))).unwrap();
        assert_eq!(style_of(&d).as_deref(), want, "w:default=\"{flag}\"");
    }
}

#[test]
fn without_a_default_table_style_the_table_has_none() {
    let paragraph = r#"<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#;
    let d = wordcraft_docx::read(&package(paragraph, &table(""))).unwrap();
    assert_eq!(style_of(&d), None);
}

/// Word's usual default, Normal Table, is what a table with no style already is: tables keep no
/// style, so saving (which writes Normal Table as the default) and reopening changes nothing.
#[test]
fn word_s_usual_normal_table_leaves_tables_unstyled() {
    let normal = r#"<w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblCellMar><w:left w:w="108" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>"#;
    let d = wordcraft_docx::read(&package(normal, &table(""))).unwrap();
    assert_eq!(style_of(&d), None);
    let back = wordcraft_docx::read(&wordcraft_docx::write(&d).unwrap()).unwrap();
    assert_eq!(style_of(&back), None);
}
