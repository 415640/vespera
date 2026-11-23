//! Vespera'nin iki cizim yuzeyi: oyun ici HUD ve ClickGUI.
//!
//! Cizim yalnizca `DrawContext` uzerinden yapilir (`fill`, `fill_gradient`, `text`,
//! `text_width`), yani native ve wasm ayni yolu kullanir. Koordinatlar GUI pikselidir.

use mod_api::{ClientHost, DrawContext};

use crate::modules::{Category, Module, Modules};
use crate::theme::Theme;

// ---------------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------------

/// Ustte Dynamic Island, solda acik moduller, sagda watermark.
pub fn draw_hud(
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    modules: &Modules,
    screen_w: f32,
    watermark_x: f32,
    watermark_y: f32,
) {
    draw_island(host, draw, screen_w);
    // watermark_y == 0, F1 acikken yalnizca saat isteniyor demektir.
    if watermark_y > 0.0 {
        draw_module_list(draw, modules);
        draw_watermark(draw, watermark_x, watermark_y);
    }
}

fn session(host: &dyn ClientHost) -> (String, String, String) {
    let session = host.client_session();
    let server = session
        .client_server()
        .and_then(|s| s.name.or(s.address))
        .unwrap_or_else(|| "Singleplayer".to_string());
    let ping = session
        .client_latency()
        .map(|ms| format!("{ms} ms"))
        .unwrap_or_else(|| "-- ms".to_string());
    let fps = session
        .client_frame_stats()
        .map(|s| format!("{} FPS", s.fps.round() as u32))
        .unwrap_or_else(|| "-- FPS".to_string());
    (server, ping, fps)
}

fn clock() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Turkiye saati (UTC+3). Sistem saati okunamaz; epoch'tan turetilir.
    let local = (secs + 3 * 3600) % 86400;
    format!("{:02}:{:02}", local / 3600, (local % 3600) / 60)
}

/// Ekranin tepesinde ortalanmis kapsul: saat, sunucu, ping, FPS.
fn draw_island(host: &dyn ClientHost, draw: &mut dyn DrawContext, screen_w: f32) {
    let (server, ping, fps) = session(host);
    let time = clock();
    let sep = " \u{00b7} ";

    let pieces = [
        (draw.text_width("V"), true),
        (5.0, false),
        (draw.text_width(&time), false),
        (draw.text_width(sep), false),
        (draw.text_width(&server), false),
        (draw.text_width(sep), false),
        (draw.text_width(&ping), false),
        (draw.text_width(sep), false),
        (draw.text_width(&fps), false),
    ];
    let content: f32 = pieces.iter().map(|(w, _)| w).sum();

    let pad = 10.0;
    let w = content + pad * 2.0;
    let h = 18.0;
    let x = ((screen_w - w) / 2.0).round();
    let y = 6.0;
    draw.fill([x, y, w, h], Theme::HUD_BG);

    let ty = y + (h - 9.0) / 2.0;
    let mut cx = x + pad;
    draw.text("V", [cx, ty], Theme::ACCENT, true);
    cx += pieces[0].0 + pieces[1].0;
    draw.text(&time, [cx, ty], Theme::TEXT, true);
    cx += pieces[2].0;
    draw.text(sep, [cx, ty], Theme::TEXT_DIM, false);
    cx += pieces[3].0;
    draw.text(&server, [cx, ty], Theme::TEXT, true);
    cx += pieces[4].0;
    draw.text(sep, [cx, ty], Theme::TEXT_DIM, false);
    cx += pieces[5].0;
    draw.text(&ping, [cx, ty], Theme::TEXT, true);
    cx += pieces[6].0;
    draw.text(sep, [cx, ty], Theme::TEXT_DIM, false);
    cx += pieces[7].0;
    draw.text(&fps, [cx, ty], Theme::TEXT, true);
}

/// Solda, acik modullerin kapsul listesi.
fn draw_module_list(draw: &mut dyn DrawContext, modules: &Modules) {
    let x = 6.0;
    let mut y = 8.0;
    for name in modules.enabled_names() {
        let w = draw.text_width(name) + 22.0;
        draw.fill([x, y, w, 15.0], Theme::HUD_BG);
        draw.fill([x + 5.0, y + 5.5, 4.0, 4.0], Theme::ACCENT);
        draw.text(name, [x + 13.0, y + 3.0], Theme::TEXT, true);
        y += 18.0;
    }
}

fn draw_watermark(draw: &mut dyn DrawContext, x: f32, y: f32) {
    let text = "Vespera";
    let w = draw.text_width(text) + 16.0;
    draw.fill([x, y, w, 16.0], Theme::HUD_BG);
    draw.fill([x + 4.0, y + 4.0, 3.0, 8.0], Theme::ACCENT);
    draw.text(text, [x + 10.0, y + 3.5], Theme::TEXT, true);
}

// ---------------------------------------------------------------------------------
// ClickGUI
// ---------------------------------------------------------------------------------

const PANEL_W: f32 = 470.0;
const PANEL_H: f32 = 292.0;
const BAR_H: f32 = 34.0;
const RAIL_W: f32 = 34.0;
const RAIL_GAP: f32 = 6.0;
const CARD_H: f32 = 46.0;
const CARD_GAP: f32 = 5.0;
const COLS: usize = 3;
const FOOTER_H: f32 = 20.0;

/// Sag Shift ile acilan menu.
pub struct ClickGui {
    open: bool,
    category: usize,
    search: String,
    search_focus: bool,
    scroll: f32,
    /// Panelin sol ust kosesi. `None` ise ilk cizimde ortalanir.
    origin: Option<[f32; 2]>,
    /// Baslik cubugundan surukleniyorsa farenin panele gore ofseti.
    drag: Option<[f32; 2]>,
}

impl ClickGui {
    pub fn new() -> Self {
        Self {
            open: false,
            category: 0,
            search: String::new(),
            search_focus: false,
            scroll: 0.0,
            origin: None,
            drag: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
        self.search_focus = false;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.drag = None;
    }

    pub fn draw(
        &mut self,
        host: &dyn ClientHost,
        draw: &mut dyn DrawContext,
        modules: &Modules,
        screen_w: f32,
        screen_h: f32,
        mouse: [f32; 2],
    ) {
        if !self.open {
            return;
        }
        draw.fill([0.0, 0.0, screen_w, screen_h], Theme::SCRIM);

        let [px, py] = *self.origin.get_or_insert([
            ((screen_w - width()) / 2.0).round(),
            ((screen_h - PANEL_H) / 2.0).round(),
        ]);

        draw.fill([px + 4.0, py + 6.0, width(), PANEL_H], Theme::SHADOW);
        draw_rail(draw, px, py, self.category, mouse);
        draw_panel(self, host, draw, modules, px + RAIL_W + RAIL_GAP, py, mouse);
    }

    /// Bir tusa basildi. `true` donerse tus oyuna iletilmesin.
    pub fn on_key(&mut self, key: &str) -> bool {
        if !self.open {
            return false;
        }
        match key {
            "key.keyboard.escape" | "key.keyboard.right.shift" => self.close(),
            "key.keyboard.backspace" if self.search_focus => {
                self.search.pop();
                self.scroll = 0.0;
            }
            _ => {}
        }
        true
    }

    pub fn on_char(&mut self, c: char) {
        if self.open && self.search_focus && !c.is_control() && self.search.chars().count() < 24 {
            self.search.push(c);
            self.scroll = 0.0;
        }
    }

    pub fn on_press(&mut self, modules: &mut Modules, mouse: [f32; 2], button: i32) {
        if !self.open || button != 0 {
            return;
        }
        let Some([px, py]) = self.origin else {
            return;
        };

        if inside(mouse, px, py, RAIL_W, PANEL_H) {
            let index = ((mouse[1] - (py + 8.0)) / 30.0).floor() as isize;
            if (0..Category::ALL.len() as isize).contains(&index) {
                self.category = index as usize;
                self.scroll = 0.0;
            }
            return;
        }

        let panel_x = px + RAIL_W + RAIL_GAP;

        if inside(mouse, panel_x, py, PANEL_W, BAR_H) {
            let search = search_box(panel_x, py);
            self.search_focus = inside(mouse, search[0], search[1], search[2], search[3]);
            if !self.search_focus {
                self.drag = Some([mouse[0] - px, mouse[1] - py]);
            }
            return;
        }
        self.search_focus = false;

        let shown = modules.visible(Category::ALL[self.category], &self.search);
        if let Some(slot) = card_slot(panel_x, py, self.scroll, mouse, shown.len()) {
            let index = shown[slot].0;
            modules.toggle(index);
        }
    }

    pub fn on_release(&mut self, button: i32) {
        if button == 0 {
            self.drag = None;
        }
    }

    pub fn on_drag(&mut self, mouse: [f32; 2], button: i32) {
        if button == 0 {
            if let Some([ox, oy]) = self.drag {
                self.origin = Some([mouse[0] - ox, mouse[1] - oy]);
            }
        }
    }

    pub fn on_scroll(&mut self, dy: f64) {
        if self.open {
            self.scroll = (self.scroll - dy as f32 * 16.0).max(0.0);
        }
    }
}

fn width() -> f32 {
    RAIL_W + RAIL_GAP + PANEL_W
}

fn inside(mouse: [f32; 2], x: f32, y: f32, w: f32, h: f32) -> bool {
    mouse[0] >= x && mouse[0] < x + w && mouse[1] >= y && mouse[1] < y + h
}

fn search_box(panel_x: f32, py: f32) -> [f32; 4] {
    [panel_x + 12.0, py + 7.0, 156.0, 20.0]
}

/// Kartlarin cizildigi alan: basligin altindan alt bilginin ustune kadar.
fn grid_top(py: f32) -> f32 {
    py + BAR_H + 8.0
}

fn grid_height() -> f32 {
    PANEL_H - BAR_H - 8.0 - FOOTER_H
}

fn card_size() -> (f32, f32) {
    let inner = PANEL_W - 24.0;
    let w = (inner - CARD_GAP * (COLS - 1) as f32) / COLS as f32;
    (w, CARD_H)
}

/// Farenin altindaki kartin sira numarasi.
fn card_slot(panel_x: f32, py: f32, scroll: f32, mouse: [f32; 2], count: usize) -> Option<usize> {
    let top = grid_top(py);
    if !inside(mouse, panel_x, top, PANEL_W, grid_height()) {
        return None;
    }
    let (cw, ch) = card_size();
    let col = ((mouse[0] - (panel_x + 12.0)) / (cw + CARD_GAP)).floor();
    let row = ((mouse[1] - top + scroll) / (ch + CARD_GAP)).floor();
    if col < 0.0 || col >= COLS as f32 || row < 0.0 {
        return None;
    }
    let slot = row as usize * COLS + col as usize;
    let x = panel_x + 12.0 + col * (cw + CARD_GAP);
    let y = top - scroll + row * (ch + CARD_GAP);
    if slot < count && inside(mouse, x, y, cw, ch) {
        Some(slot)
    } else {
        None
    }
}

/// Sol dikey ikon cubugu.
fn draw_rail(draw: &mut dyn DrawContext, x: f32, y: f32, active: usize, mouse: [f32; 2]) {
    draw.fill([x, y, RAIL_W, PANEL_H], Theme::PANEL_BAR);

    for (i, category) in Category::ALL.iter().enumerate() {
        let by = y + 8.0 + i as f32 * 30.0;
        let hovered = inside(mouse, x, by, RAIL_W, 28.0);
        if i == active {
            draw.fill([x, by + 4.0, 2.0, 18.0], Theme::ACCENT);
            draw.fill([x + 5.0, by + 2.0, RAIL_W - 8.0, 24.0], Theme::CARD_ON);
        } else if hovered {
            draw.fill([x + 5.0, by + 2.0, RAIL_W - 8.0, 24.0], Theme::WELL);
        }
        let glyph = category.glyph();
        let gx = x + (RAIL_W - draw.text_width(glyph)) / 2.0;
        let color = if i == active {
            Theme::ACCENT_HI
        } else {
            Theme::TEXT_DIM
        };
        draw.text(glyph, [gx, by + 9.0], color, false);
    }
}

fn draw_panel(
    gui: &ClickGui,
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    modules: &Modules,
    x: f32,
    y: f32,
    mouse: [f32; 2],
) {
    draw.fill_gradient([x, y, PANEL_W, PANEL_H], 0xF218_1920, Theme::PANEL);
    draw_header(gui, host, draw, x, y, mouse);
    draw_cards(gui, draw, modules, x, y, mouse);

    draw.fill(
        [x + 12.0, y + PANEL_H - FOOTER_H, PANEL_W - 24.0, 1.0],
        Theme::LINE,
    );
    draw.text(
        "Sol tik: ac/kapat     Tekerlek: kaydir     Baslik: surukle     Esc: kapat",
        [x + 12.0, y + PANEL_H - 13.0],
        Theme::TEXT_FAINT,
        false,
    );
}

fn draw_header(
    gui: &ClickGui,
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    x: f32,
    y: f32,
    mouse: [f32; 2],
) {
    draw.fill([x, y + BAR_H - 1.0, PANEL_W, 1.0], Theme::LINE);

    let [sx, sy, sw, sh] = search_box(x, y);
    let hovered = inside(mouse, sx, sy, sw, sh);
    draw.fill(
        [sx, sy, sw, sh],
        if gui.search_focus || hovered {
            Theme::CARD_HOVER
        } else {
            Theme::WELL
        },
    );
    if gui.search_focus {
        draw.fill([sx, sy + sh - 1.0, sw, 1.0], Theme::ACCENT);
    }
    let (shown, color) = if gui.search.is_empty() && !gui.search_focus {
        ("Ara...".to_string(), Theme::TEXT_FAINT)
    } else if gui.search_focus {
        (format!("{}_", gui.search), Theme::TEXT)
    } else {
        (gui.search.clone(), Theme::TEXT)
    };
    draw.text(&shown, [sx + 6.0, sy + 5.5], color, false);

    // Sag ust: saat ve baglanti durumu.
    let time = clock();
    let tw = draw.text_width(&time);
    draw.text(
        &time,
        [x + PANEL_W - tw - 12.0, y + 12.5],
        Theme::TEXT,
        true,
    );

    let online = host.client_session().client_server().is_some();
    let status = if online { "cevrimici" } else { "tek oyunculu" };
    let swidth = draw.text_width(status);
    draw.text(
        status,
        [x + PANEL_W - tw - swidth - 24.0, y + 12.5],
        Theme::TEXT_DIM,
        false,
    );
}

fn draw_cards(
    gui: &ClickGui,
    draw: &mut dyn DrawContext,
    modules: &Modules,
    x: f32,
    y: f32,
    mouse: [f32; 2],
) {
    let category = Category::ALL[gui.category];
    let shown = modules.visible(category, &gui.search);
    let (cw, ch) = card_size();
    let top = grid_top(y);

    // Baslik.
    let title = format!("{}  {}", category.title(), shown.len());
    draw.text(&title, [x + 12.0, top - 1.0], Theme::TEXT_DIM, false);

    let cards_top = top + 12.0;
    if draw
        .push_clip([x, cards_top, PANEL_W, grid_height() - 12.0])
        .is_err()
    {
        return;
    }
    for (slot, (_, module)) in shown.iter().enumerate() {
        let col = slot % COLS;
        let row = slot / COLS;
        let cx = x + 12.0 + col as f32 * (cw + CARD_GAP);
        let cy = cards_top - gui.scroll + row as f32 * (ch + CARD_GAP);
        if cy + ch < cards_top || cy > cards_top + grid_height() {
            continue;
        }
        let hovered = inside(mouse, cx, cy, cw, ch);
        draw_card(draw, module, cx, cy, cw, ch, hovered);
    }
    let _ = draw.pop_clip();

    if shown.is_empty() {
        draw.text(
            "Bu kategoride modul yok.",
            [x + 12.0, cards_top + 8.0],
            Theme::TEXT_FAINT,
            false,
        );
    }
}

fn draw_card(
    draw: &mut dyn DrawContext,
    module: &Module,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    hovered: bool,
) {
    let bg = if module.enabled {
        Theme::CARD_ON
    } else if hovered {
        Theme::CARD_HOVER
    } else {
        Theme::CARD
    };
    draw.fill([x, y, w, h], bg);
    if module.enabled {
        draw.fill([x, y, 2.0, h], Theme::ACCENT);
    }

    draw.text(module.name, [x + 9.0, y + 7.0], Theme::TEXT, true);

    let note = if module.live {
        module.description
    } else {
        "Yalnizca gorsel — etkisi yok"
    };
    draw.text(note, [x + 9.0, y + 20.0], Theme::TEXT_DIM, false);

    draw_switch(draw, x + w - 36.0, y + (h - 14.0) / 2.0, module.enabled);

    // Kartin sag altinda uc nokta, ekran goruntusundeki gibi.
    draw.text(
        "...",
        [x + w - 20.0, y + h - 13.0],
        Theme::TEXT_FAINT,
        false,
    );
}

/// Acik/kapali anahtar: 26x14 iz, icinde 10px top.
fn draw_switch(draw: &mut dyn DrawContext, x: f32, y: f32, on: bool) {
    let track = if on { Theme::ACCENT } else { Theme::TRACK_OFF };
    draw.fill([x, y, 26.0, 14.0], track);
    let knob_x = if on { x + 13.0 } else { x + 2.0 };
    draw.fill([knob_x, y + 2.0, 10.0, 10.0], Theme::TEXT);
}
