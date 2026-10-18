//! Define New Bullet (Home › Paragraph › Bullets, #2786): the bullet character (picked in the
//! Symbol dialog), its font (in the app's Font dialog) and its alignment, with a preview. It
//! ends in one command, `para.defineBullet`, so agents get the same result without the dialog.

use egui::{Ui, vec2};
use serde::Serialize;
use serde_json::json;
use wordcraft_doc::props::Align;

use crate::WordApp;
use crate::dialogs::Dialog;
use crate::dialogs_insert::{InsertDialog, SymbolForm};
use crate::theme::{Tokens, regular, semibold};

/// The alignments the dialog offers: `para.defineBullet`'s `align` and their names.
pub const ALIGNMENTS: [(&str, &str); 3] = [("left", "Left"), ("center", "Center"), ("right", "Right")];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineBulletForm {
    /// The bullet character.
    pub ch: String,
    /// The bullet's own font; empty = the text's.
    pub font: String,
    /// The bullet's size in points; empty = the text's.
    pub size: String,
    pub bold: bool,
    pub italic: bool,
    /// `RRGGBB`; empty or `auto` = the text's colour.
    pub color: String,
    /// `left`, `center` or `right`.
    pub align: String,
}

impl DefineBulletForm {
    /// Starts from the bullet the caret's paragraph has (its level's character, font and
    /// alignment), or a plain • when it has none.
    pub fn new(app: &WordApp) -> DefineBulletForm {
        let doc = &app.session.doc;
        let level = doc
            .para_at(&app.session.sel.focus)
            .and_then(|p| doc.styles.resolve_para(&p.props).numbering)
            .filter(|n| n.num != 0)
            .and_then(|n| doc.numbering.level(n.num, n.level))
            .filter(|l| l.format == wordcraft_doc::section::NumFormat::Bullet && l.text.chars().count() == 1);
        match level {
            Some(l) => DefineBulletForm {
                ch: l.text.clone(),
                font: l.chr.font.clone().unwrap_or_default(),
                size: l.chr.size.map(|s| s.to_string()).unwrap_or_default(),
                bold: l.chr.bold.unwrap_or(false),
                italic: l.chr.italic.unwrap_or(false),
                color: match l.chr.color {
                    Some(wordcraft_doc::props::TextColor::Rgb(c)) => c.hex(),
                    _ => String::new(),
                },
                align: match l.align {
                    Align::Center => "center",
                    Align::Right => "right",
                    _ => "left",
                }
                .into(),
            },
            None => DefineBulletForm::plain("•"),
        }
    }

    /// A bullet `ch` in the text's own formatting, left aligned.
    pub fn plain(ch: &str) -> DefineBulletForm {
        DefineBulletForm {
            ch: ch.into(),
            font: String::new(),
            size: String::new(),
            bold: false,
            italic: false,
            color: String::new(),
            align: "left".into(),
        }
    }

    /// The bullet's formatting as the preview draws it (the preview fits its size).
    pub fn chr(&self) -> wordcraft_doc::props::CharProps {
        wordcraft_doc::props::CharProps {
            font: (!self.font.is_empty()).then(|| self.font.clone()),
            bold: self.bold.then_some(true),
            italic: self.italic.then_some(true),
            color: wordcraft_doc::Rgb::parse(&self.color).map(wordcraft_doc::props::TextColor::Rgb),
            ..Default::default()
        }
    }

    /// The `para.defineBullet` params this form stands for.
    pub fn params(&self) -> serde_json::Value {
        let mut v = json!({"char": self.ch, "font": self.font, "align": self.align, "bold": self.bold, "italic": self.italic});
        if let Some(pt) = self.size.trim().parse::<f64>().ok().filter(|pt| (1.0..=1638.0).contains(pt)) {
            v["size"] = json!(pt);
        }
        v["color"] = json!(if wordcraft_doc::Rgb::parse(&self.color).is_some() { self.color.as_str() } else { "auto" });
        v
    }
}

/// Draws the dialog; returns true to close it (OK, Cancel, or Symbol… / Font…, which hand the
/// dialog over to the Symbol or Font dialog and get it back when it closes).
pub fn body(app: &mut WordApp, ui: &mut Ui, f: &mut DefineBulletForm) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.set_width(340.0);
    ui.label(egui::RichText::new(tl!("Bullet character")).font(semibold(12.0)));
    ui.add_space(4.0);
    let (mut to_symbol, mut to_font) = (false, false);
    ui.horizontal(|ui| {
        to_symbol = ui.button(tl!("Symbol…")).clicked();
        to_font = ui.button(tl!("Font…")).clicked();
    });
    if to_symbol {
        let form = f.clone();
        app.dialog = Some(Dialog::Insert(Box::new(InsertDialog::Symbol(SymbolForm::for_bullet(app, form)))));
        return true;
    }
    if to_font {
        // The app's own Font dialog, formatting the bullet and handing it back.
        if let Some(d) = crate::dialogs::font_for_bullet(app, f.clone()) {
            app.dialog = Some(d);
            return true;
        }
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(tl!("Alignment:"));
        ui.add_space(6.0);
        for (k, n) in ALIGNMENTS {
            ui.radio_value(&mut f.align, k.to_string(), tl!(n));
        }
    });
    ui.add_space(8.0);
    ui.label(egui::RichText::new(tl!("Preview")).font(regular(12.0)).color(t.text_dim));
    preview(app, ui, f);
    ui.add_space(6.0);
    let (ok, cancel) = crate::dialogs_insert::buttons(ui, "OK", "Cancel", f.ch.chars().count() == 1);
    if ok {
        let _ = app.run("para.defineBullet", f.params());
    }
    ok || cancel
}

/// The preview, drawn like Define New Multilevel List's: two plain lines of text, three bulleted
/// ones and two plain lines again, as grey bars on a white page. The bullet sits at the first
/// level's aligned-at position (18 pt), aligned there as chosen, and its text starts at the text
/// indent (36 pt); the bullet itself is laid out by the layout engine, so a symbol font shows the
/// character the page will.
fn preview(app: &mut WordApp, ui: &mut Ui, f: &DefineBulletForm) {
    let t = Tokens::get(ui.ctx());
    let row = 17.0;
    let (rect, _) = ui.allocate_exact_size(vec2(340.0, row * 7.0 + 10.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 2.0, egui::Color32::WHITE);
    p.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, t.border_strong), egui::StrokeKind::Inside);
    // Positions in points, scaled so 4.5 inches fit, as in Define New Multilevel List.
    let scale = (rect.width() - 16.0) / 324.0;
    let left = rect.left() + 8.0;
    let (aligned_at, text_at) = (left + 18.0 * scale, left + 36.0 * scale);
    let bar = |y: f32, from: f32, gray: u8| {
        let r = egui::Rect::from_min_max(egui::pos2(from, y - 2.0), egui::pos2(rect.right() - 8.0, y + 2.0));
        p.rect_filled(r, 1.0, egui::Color32::from_gray(gray));
    };
    let glyph = 14.0;
    for i in 0..7 {
        let y = rect.top() + 5.0 + row * i as f32 + row / 2.0;
        if !(2..5).contains(&i) {
            bar(y, left, 205);
            continue;
        }
        let x = match f.align.as_str() {
            "center" => aligned_at - glyph / 2.0,
            "right" => aligned_at - glyph,
            _ => aligned_at,
        };
        let r = egui::Rect::from_min_size(egui::pos2(x, y - glyph / 2.0), vec2(glyph, glyph));
        crate::previews::glyph(app, ui, r, &f.ch, &f.chr(), egui::Color32::WHITE);
        bar(y, text_at.max(r.right() + 2.0), 120);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use wordcraft_engine::Session;

    /// The interface fonts take effect from the frame after they're installed: the first frame
    /// installs them and draws nothing.
    fn fonts_ready(ui: &Ui) -> bool {
        let id = egui::Id::new("define_bullet_test_fonts");
        let ready = ui.ctx().data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
        if !ready {
            crate::theme::install_fonts(ui.ctx());
            ui.ctx().data_mut(|d| d.insert_temp(id, true));
        }
        ready
    }

    fn app_with_bullet() -> WordApp {
        let mut a = WordApp::new(Session::new(wordcraft_doc::Document::from_text("One\nTwo")), crate::Services::default());
        a.run("para.bullets", json!({"kind": "▪"})).unwrap();
        a
    }

    fn level_at(app: &WordApp, i: usize) -> wordcraft_doc::numbering::Level {
        let doc = &app.session.doc;
        let n = doc.para_at(&wordcraft_doc::Pos::body(i, 0)).unwrap().props.numbering.unwrap();
        doc.numbering.level(n.num, n.level).unwrap().clone()
    }

    /// The form starts from the caret's bullet; OK ends in `para.defineBullet` with the
    /// character, its font and the alignment.
    #[test]
    fn ok_defines_the_bullet_the_form_shows() {
        let app = app_with_bullet();
        let f = DefineBulletForm::new(&app);
        let font = level_at(&app, 0).chr.font.unwrap_or_default();
        assert_eq!((f.ch.as_str(), f.font.as_str(), f.align.as_str()), ("▪", font.as_str(), "left"));
        let mut h = egui_kittest::Harness::builder().with_size(vec2(500.0, 500.0)).build_ui_state(
            |ui, (app, f): &mut (WordApp, DefineBulletForm)| {
                if fonts_ready(ui) {
                    body(app, ui, f);
                }
            },
            (app, DefineBulletForm { font: "Segoe UI Symbol".into(), align: "right".into(), ..DefineBulletForm::plain("†") }),
        );
        h.run_steps(2);
        h.get_by_label("OK").click();
        h.run_steps(2);
        let l = level_at(&h.state().0, 0);
        assert_eq!((l.text.as_str(), l.chr.font.as_deref(), l.align), ("†", Some("Segoe UI Symbol"), Align::Right));
    }

    /// Symbol… hands the form to the Symbol dialog, whose OK brings it back with the character
    /// picked there; Cancel brings it back unchanged.
    #[test]
    fn symbol_picks_the_character_and_returns() {
        let form = DefineBulletForm { align: "center".into(), ..DefineBulletForm::plain("▪") };
        let mut h = egui_kittest::Harness::builder().with_size(vec2(500.0, 500.0)).build_ui_state(
            |ui, (app, f): &mut (WordApp, DefineBulletForm)| {
                if fonts_ready(ui) {
                    body(app, ui, f);
                }
            },
            (app_with_bullet(), form.clone()),
        );
        h.run_steps(2);
        h.get_by_label("Symbol…").click();
        h.run_steps(2);
        let mut app = std::mem::replace(&mut h.state_mut().0, app_with_bullet());
        drop(h);
        let Some(Dialog::Insert(d)) = app.dialog.take() else { panic!("the Symbol dialog is open") };
        let InsertDialog::Symbol(mut sym) = *d else { panic!("the Symbol dialog is open") };
        assert_eq!(sym.bullet.as_deref().map(|b| b.align.as_str()), Some("center"), "it carries the form");
        sym.selected = Some('†');
        let mut h = egui_kittest::Harness::builder().with_size(vec2(700.0, 700.0)).build_ui_state(
            |ui, (app, d): &mut (WordApp, InsertDialog)| {
                if fonts_ready(ui) {
                    crate::dialogs_insert::body(app, ui, d);
                }
            },
            (app, InsertDialog::Symbol(sym)),
        );
        h.run_steps(2);
        h.get_by_label("OK").click();
        h.run_steps(2);
        let back = match h.state_mut().0.dialog.take() {
            Some(Dialog::Insert(d)) => match *d {
                InsertDialog::DefineBullet(f) => f,
                _ => panic!("back to Define New Bullet"),
            },
            _ => panic!("back to Define New Bullet"),
        };
        assert_eq!((back.ch.as_str(), back.align.as_str()), ("†", "center"));
        // Nothing was inserted into the text.
        assert_eq!(h.state().0.session.doc.para_at(&wordcraft_doc::Pos::body(0, 0)).unwrap().text, "One");
    }
}
