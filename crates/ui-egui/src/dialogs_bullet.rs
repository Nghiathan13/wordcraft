//! Define New Bullet (Home › Paragraph › Bullets, #2786): the bullet character (picked in the
//! Symbol dialog), its font and its alignment, with a preview laid out by the layout engine. It
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
pub const ALIGNMENTS: [(&str, &str); 3] = [("left", "Left"), ("center", "Centered"), ("right", "Right")];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefineBulletForm {
    /// The bullet character.
    pub ch: String,
    /// The bullet's own font; empty = the text's.
    pub font: String,
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
                align: match l.align {
                    Align::Center => "center",
                    Align::Right => "right",
                    _ => "left",
                }
                .into(),
            },
            None => DefineBulletForm { ch: "•".into(), font: String::new(), align: "left".into() },
        }
    }

    /// The `para.defineBullet` params this form stands for.
    pub fn params(&self) -> serde_json::Value {
        json!({"char": self.ch, "font": self.font, "align": self.align})
    }
}

/// Draws the dialog; returns true to close it (OK, Cancel, or Symbol…, which hands the dialog
/// over to the Symbol dialog and gets it back when a character is picked).
pub fn body(app: &mut WordApp, ui: &mut Ui, f: &mut DefineBulletForm) -> bool {
    let t = Tokens::get(ui.ctx());
    ui.set_width(340.0);
    ui.label(egui::RichText::new(tl!("Bullet character")).font(semibold(12.0)));
    ui.add_space(4.0);
    let mut to_symbol = false;
    ui.horizontal(|ui| {
        // The current character, in its font, beside the button that changes it.
        let (r, _) = ui.allocate_exact_size(vec2(34.0, 34.0), egui::Sense::hover());
        ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
        crate::previews::glyph(app, ui, r.shrink(4.0), &f.ch, &f.font);
        if ui.button(tl!("Symbol…")).clicked() {
            to_symbol = true;
        }
    });
    if to_symbol {
        let form = f.clone();
        app.dialog = Some(Dialog::Insert(Box::new(InsertDialog::Symbol(SymbolForm::for_bullet(app, form)))));
        return true;
    }
    ui.add_space(6.0);
    egui::Grid::new("define_bullet_grid").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
        ui.label(tl!("Font:"));
        let normal = tl!("(normal text)").to_string();
        let mut items = vec![normal.clone()];
        items.extend(app.previews.families());
        let shown = if f.font.is_empty() { normal.clone() } else { f.font.clone() };
        if let Some(v) = crate::widgets::combo(ui, "define_bullet_font", 200.0, &shown, &items, None) {
            f.font = if v == normal || v.trim().is_empty() { String::new() } else { v.trim().to_string() };
        }
        ui.end_row();
        ui.label(tl!("Alignment:"));
        let current = ALIGNMENTS.iter().find(|(k, _)| *k == f.align).map_or("Left", |(_, n)| *n);
        egui::ComboBox::from_id_salt("define_bullet_align").selected_text(tl!(current)).width(200.0).show_ui(ui, |ui| {
            for (k, n) in ALIGNMENTS {
                if ui.selectable_label(f.align == k, tl!(n)).clicked() {
                    f.align = k.to_string();
                }
            }
        });
        ui.end_row();
    });
    ui.add_space(8.0);
    ui.label(egui::RichText::new(tl!("Preview")).font(regular(12.0)).color(t.text_dim));
    crate::previews::bullet_preview(app, ui, &f.ch, &f.font, &f.align, vec2(340.0, 92.0));
    ui.add_space(6.0);
    let (ok, cancel) = crate::dialogs_insert::buttons(ui, "OK", "Cancel", f.ch.chars().count() == 1);
    if ok {
        let _ = app.run("para.defineBullet", f.params());
    }
    ok || cancel
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
            (app, DefineBulletForm { ch: "†".into(), font: "Segoe UI Symbol".into(), align: "right".into() }),
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
        let form = DefineBulletForm { ch: "▪".into(), font: String::new(), align: "center".into() };
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
