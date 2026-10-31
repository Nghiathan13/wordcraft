//! #5056: an exam header table names no style, so Word formats it with the document's default
//! table style, whose header row part draws a rule under the row and bottom-aligns its cells.
//! WordCraft drew neither.
use std::io::Write;

use wordcraft_engine::Session;
use wordcraft_engine::layout::Placed;

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn package() -> Vec<u8> {
    let root = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let styles = format!(
        r#"<w:styles xmlns:w="{W}"><w:style w:type="table" w:default="1" w:styleId="Rule"><w:name w:val="Rule"/><w:tblStylePr w:type="firstRow"><w:tcPr><w:vAlign w:val="bottom"/><w:tcBorders><w:bottom w:val="single" w:sz="4"/></w:tcBorders></w:tcPr></w:tblStylePr></w:style></w:styles>"#
    );
    let nil = ["top", "left", "bottom", "right", "insideH", "insideV"].map(|e| format!(r#"<w:{e} w:val="nil"/>"#)).concat();
    let doc = format!(
        r#"<w:document xmlns:w="{W}"><w:body><w:tbl><w:tblPr><w:tblLook w:firstRow="1" w:noVBand="1"/><w:tblBorders>{nil}</w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Left</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Right</w:t></w:r></w:p><w:p><w:r><w:t>Code</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:body></w:document>"#
    );
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, body) in [("_rels/.rels", root), ("word/_rels/document.xml.rels", rels), ("word/styles.xml", &styles), ("word/document.xml", &doc)] {
        zw.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        zw.write_all(body.as_bytes()).unwrap();
    }
    zw.finish().unwrap().into_inner()
}

#[test]
fn the_default_table_style_rules_and_bottom_aligns_the_header_row() {
    let doc = wordcraft_engine::io::open_bytes("exam.docx", &package()).expect("opens");
    let mut s = Session::new(doc);
    let l = s.layout();
    let items = &l.pages[0].items;
    let rules: Vec<_> = items.iter().filter_map(|i| if let Placed::Rule { y0, y1, .. } = i { Some((*y0, *y1)) } else { None }).collect();
    assert!(!rules.is_empty() && rules.iter().all(|(a, b)| a == b), "a horizontal rule under the row: {rules:?}");

    // The top of the first line of paragraph `para` of cell `cell`.
    let top = |cell: u32, para: u32| {
        items.iter().find_map(|i| match i {
            Placed::Lines { path, y, .. } if path.0 == [0, 0, cell, para] => Some(*y),
            _ => None,
        })
    };
    let (left, right, code) = (top(0, 0).unwrap(), top(1, 0).unwrap(), top(1, 1).unwrap());
    assert!((left - code).abs() < 0.5 && left > right + 1.0, "the one-line cell sits at the bottom: {left} {right} {code}");
}
