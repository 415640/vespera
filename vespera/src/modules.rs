//! Vespera'nin modul listesi. Tek sahip vardir: `Modules`. HUD acik olanlari okur,
//! ClickGUI acip kapatir.

/// Bir modulun menusunde durdugu sutun.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Combat,
    Movement,
    Render,
    Player,
    World,
    Misc,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Self::Combat,
        Self::Movement,
        Self::Render,
        Self::Player,
        Self::World,
        Self::Misc,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Combat => "Combat",
            Self::Movement => "Movement",
            Self::Render => "Render",
            Self::Player => "Player",
            Self::World => "World",
            Self::Misc => "Misc",
        }
    }

    /// Sol ikon cubugunda gosterilen tek harf.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Combat => "C",
            Self::Movement => "M",
            Self::Render => "R",
            Self::Player => "P",
            Self::World => "W",
            Self::Misc => "+",
        }
    }
}

pub struct Module {
    pub name: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub enabled: bool,
    /// `false` ise modul menude gorunur ama oyunda bir etkisi yoktur.
    /// Horizon'un acik API'si yalnizca gorsel kancalar verir; hareket ve savas
    /// modulleri sunucuya giden paketleri degistirmeden calisamaz, o yuzden
    /// bilerek bagli degildir.
    pub live: bool,
}

pub struct Modules {
    list: Vec<Module>,
}

impl Modules {
    pub fn new() -> Self {
        Self {
            list: vec![
                module(
                    "PlayerESP",
                    Category::Render,
                    "Oyunculari duvar arkasindan mor siluetle gosterir.",
                    true,
                    true,
                ),
                module(
                    "Fullbright",
                    Category::Render,
                    "Gece karanligini bu istemcide kaldirir.",
                    true,
                    true,
                ),
                module(
                    "NoFog",
                    Category::Render,
                    "Sis ve gorus pusunu kaldirir.",
                    true,
                    true,
                ),
                module("Zoom", Category::Render, "Tusla yaklasma.", false, false),
                module(
                    "Coordinates",
                    Category::Player,
                    "Koordinatlari ekranda gosterir.",
                    false,
                    false,
                ),
                module(
                    "ArmorHUD",
                    Category::Player,
                    "Zirh dayanikliligini gosterir.",
                    false,
                    false,
                ),
                module(
                    "Keystrokes",
                    Category::Player,
                    "Basilan tuslari gosterir.",
                    false,
                    false,
                ),
                module(
                    "ToggleSprint",
                    Category::Movement,
                    "Kosmayi acik tutar.",
                    false,
                    false,
                ),
                module(
                    "AutoReconnect",
                    Category::Misc,
                    "Baglanti kopunca yeniden dener.",
                    false,
                    false,
                ),
                module(
                    "TabList",
                    Category::Misc,
                    "Oyuncu listesini acik tutar.",
                    false,
                    false,
                ),
            ],
        }
    }

    pub fn enabled(&self, name: &str) -> bool {
        self.list.iter().any(|m| m.name == name && m.enabled)
    }

    pub fn enabled_names(&self) -> impl Iterator<Item = &str> {
        self.list.iter().filter(|m| m.enabled).map(|m| m.name)
    }

    /// Kategoriye ve arama metnine uyan moduller, orijinal indeksleriyle.
    pub fn visible(&self, category: Category, query: &str) -> Vec<(usize, &Module)> {
        let query = query.to_lowercase();
        self.list
            .iter()
            .enumerate()
            .filter(|(_, m)| m.category == category)
            .filter(|(_, m)| query.is_empty() || m.name.to_lowercase().contains(&query))
            .collect()
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some(module) = self.list.get_mut(index) {
            module.enabled = !module.enabled;
        }
    }
}

fn module(
    name: &'static str,
    category: Category,
    description: &'static str,
    enabled: bool,
    live: bool,
) -> Module {
    Module {
        name,
        category,
        description,
        enabled,
        live,
    }
}
