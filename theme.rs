//! Vespera renk paleti. Tum renkler duz alfa ARGB (`0xAARRGGBB`).

pub struct Theme;

impl Theme {
    /// Marka moru, opak.
    pub const ACCENT: u32 = 0xFF8A_5CF6;
    /// Marka moru, `0xRRGGBB` (entity outline bunu ister).
    pub const ACCENT_RGB: u32 = 0x008A_5CF6;
    /// Parlak mor (hover, secili kenar).
    pub const ACCENT_HI: u32 = 0xFFA7_8BFA;
    /// Panel govdesi: neredeyse opak antrasit.
    pub const PANEL: u32 = 0xF211_1216;
    /// Panel basligi.
    pub const PANEL_BAR: u32 = 0xFF16_171C;
    /// Kart govdesi.
    pub const CARD: u32 = 0xFF1C_1D24;
    /// Kart uzerine gelinmis.
    pub const CARD_HOVER: u32 = 0xFF26_2732;
    /// Acik modulun karti.
    pub const CARD_ON: u32 = 0xFF2A_2440;

    /// HUD kapsulu.
    pub const HUD_BG: u32 = 0xC011_1216;
    /// Arama kutusu ve ikon cubugu.
    pub const WELL: u32 = 0xFF22_232B;
    /// Ince ayirici cizgi.
    pub const LINE: u32 = 0xFF2E_2F38;
    /// Kapali anahtar izi.
    pub const TRACK_OFF: u32 = 0xFF3A_3B46;

    pub const TEXT: u32 = 0xFFF4_F4F5;
    pub const TEXT_DIM: u32 = 0xFFA1_A1AA;
    pub const TEXT_FAINT: u32 = 0xFF6E_6E78;

    /// Menunun arkasindaki karartma.
    pub const SCRIM: u32 = 0x9900_0000;

    /// Yumusak golge (panelin altina cizilir).
    pub const SHADOW: u32 = 0x6600_0000;
}
