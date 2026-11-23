//! Vespera Client — Horizon icin native HUD ve ClickGUI modu.
//!
//! Sag Shift menuyu acar. HUD oyun sirasinda cizilir; ClickGUI oyunun kendi ekran
//! sistemi uzerinden cizilir, boylece fare ile suruklenir, kartlara tiklanir ve
//! tekerlek ile kayar.

mod modules;
mod theme;
mod ui;

use std::sync::Mutex;

use mod_api::{
    self, ClientHost, ClientTickPostEvent, ClientTickPreEvent, ComputeFogColorEvent, Corner,
    EntityOutlineEvent, Flow, GuiRenderPostEvent, Hooks, InputKeyEvent, KeyAction, KeyRegistration,
    Registrar, RegistrationContext, RenderFogEvent, RenderGuiPostEvent, ScreenCharTypedPreEvent,
    ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent, ScreenMouseButtonReleasedPreEvent,
    ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent, ScreenRenderPostEvent,
};

use modules::Modules;
use theme::Theme;
use ui::ClickGui;

/// `horizon-mod.toml` icindeki `id` ile ayni olmak zorunda: ekran kimligi ve tus
/// baglantisi buna gore kurulur.
const MOD_ID: &str = "vespera";

/// Kontroller menusune eklenen "ClickGUI" tusunun kimligi. Varsayilan: Sag Shift.
const KEY_GUI: &str = "clickgui";

pub struct Vespera {
    modules: Mutex<Modules>,
    gui: Mutex<ClickGui>,
    /// Sag Shift basili tutulurken tekrarlarin menuyu acip kapamasini engeller.
    gui_key_held: Mutex<bool>,
}

impl Default for Vespera {
    fn default() -> Self {
        Self {
            modules: Mutex::new(Modules::new()),
            gui: Mutex::new(ClickGui::new()),
            gui_key_held: Mutex::new(false),
        }
    }
}

impl Vespera {
    fn open_gui(&self, host: &mut dyn ClientHost) {
        let mut gui = self.gui.lock().unwrap();
        if !gui.is_open() {
            gui.open();
            host.mod_screen_open(MOD_ID);
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Vespera {
    /// Kontroller menusune "Vespera" basligi altinda ClickGUI baglantisini ekler.
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        registrar.register_key_mapping(
            KeyRegistration::new(KEY_GUI, "key.keyboard.right.shift").category("Vespera"),
        );
    }

    /// Tus baglantisinin tiklamasini tuketerek menuyu acar. Tus, Kontroller menusunden degistirilebilir.
    fn client_tick_pre(&self, host: &mut dyn ClientHost, _: &mut ClientTickPreEvent) {
        if host.consume_click(MOD_ID, KEY_GUI) && host.mod_screen().is_none() {
            self.open_gui(host);
        }
    }

    /// Ekran disaridan kapandiysa (olum ekrani, baska ekran) GUI durumunu da kapatir.
    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        let own = host.mod_screen() == Some(MOD_ID);
        let mut gui = self.gui.lock().unwrap();
        if gui.is_open() && !own {
            gui.close();
        } else if !gui.is_open() && own {
            // Kapatma yalnizca degistirilebilir host'tan yapilabilir; bu kanca odur.
            gui.close();
            host.mod_screen_close(MOD_ID);
        }
    }

    /// HUD: Dynamic Island, sol modul listesi, sag watermark. F1 acikken oyun bunu cagirmaz.
    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        let [w, _] = event.draw.size();
        let (mark_x, mark_y) = event.corners.place(Corner::TopRight, 70.0, 16.0);
        let modules = self.modules.lock().unwrap();
        ui::draw_hud(host, event.draw, &modules, w, mark_x, mark_y);
    }

    /// Fullbright: sis rengini gunduz beyazina ceker, gece karanligini kaldirir.
    /// Yalnizca bu istemcinin goruntusudur, sunucuya bir sey gitmez.
    fn compute_fog_color(&self, _: &dyn ClientHost, event: &mut ComputeFogColorEvent<'_>) {
        if self.modules.lock().unwrap().enabled("Fullbright") {
            *event.rgb = [1.0, 1.0, 1.0];
        }
    }

    /// NoFog: sis mesafesini gorus mesafesinin otesine iter.
    fn render_fog(&self, _: &dyn ClientHost, event: &mut RenderFogEvent<'_>) {
        if self.modules.lock().unwrap().enabled("NoFog") {
            event.fog.environmental_start = 100_000.0;
            event.fog.environmental_end = 100_000.0;
        }
    }

    /// PlayerESP: oyunculari duvar arkasindan Vespera moruyla isaretler.
    fn entity_outline(&self, _: &dyn ClientHost, event: &mut EntityOutlineEvent<'_>) {
        if self.modules.lock().unwrap().enabled("PlayerESP") && event.kind == "minecraft:player" {
            *event.outline = Some(Theme::ACCENT_RGB);
        }
    }

    /// Sag Shift'e ilk basista menuyu acar. Tus Kontroller menusunden degistirildiyse
    /// `client_tick_pre` icindeki baglanti devreye girer.
    fn input_key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        if event.key != "key.keyboard.right.shift" {
            return;
        }
        let mut held = self.gui_key_held.lock().unwrap();
        match event.action {
            KeyAction::Release => *held = false,
            KeyAction::Press if !*held => {
                *held = true;
                drop(held);
                if host.mod_screen().is_none() {
                    self.open_gui(host);
                }
            }
            _ => {}
        }
    }

    /// ClickGUI, oyunun ekran ciziminin uzerine cizilir.
    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        if host.mod_screen() != Some(MOD_ID) {
            return;
        }
        let [w, h] = event.draw.size();
        let modules = self.modules.lock().unwrap();
        self.gui
            .lock()
            .unwrap()
            .draw(host, event.draw, &modules, w, h, event.screen.mouse);
    }

    /// F1 acikken HUD gizlenir; menu acikken en azindan saat gorunsun diye Dynamic Island burada cizilir.
    fn gui_render_post(&self, host: &dyn ClientHost, event: &mut GuiRenderPostEvent<'_>) {
        if !event.focused || !event.gui_hidden || host.mod_screen().is_some() {
            return;
        }
        let [w, _] = event.batch.size();
        let modules = self.modules.lock().unwrap();
        ui::draw_hud(host, event.batch, &modules, w, w, 0.0);
    }

    fn screen_key_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) || event.repeat {
            return Flow::Continue;
        }
        let mut gui = self.gui.lock().unwrap();
        if gui.on_key(event.key) {
            // Ekranin kendisi bir sonraki tick'te `client_tick_post` ile kapanir:
            // bu kancanin host'u salt okunur.
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_char_typed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenCharTypedPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        self.gui.lock().unwrap().on_char(event.c);
        Flow::Cancel
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        self.gui.lock().unwrap().on_press(
            &mut self.modules.lock().unwrap(),
            event.screen.mouse,
            event.button,
        );
        Flow::Cancel
    }

    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        self.gui.lock().unwrap().on_release(event.button);
        Flow::Cancel
    }

    fn screen_mouse_dragged_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseDraggedPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        self.gui
            .lock()
            .unwrap()
            .on_drag(event.screen.mouse, event.button);
        Flow::Cancel
    }

    fn screen_mouse_scrolled_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseScrolledPreEvent<'_>,
    ) -> Flow {
        if host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        self.gui.lock().unwrap().on_scroll(event.scroll[1]);
        Flow::Cancel
    }
}
