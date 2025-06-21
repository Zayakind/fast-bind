use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Auto,    // Следовать системной теме
    Light,   // Светлая тема
    Dark,    // Тёмная тема
}

pub struct AppTheme {
    pub mode: ThemeMode,
}

impl AppTheme {
    pub fn new() -> Self {
        Self {
            mode: ThemeMode::Auto,
        }
    }
    
    pub fn apply(&self, ctx: &egui::Context) {
        match self.mode {
            ThemeMode::Auto => ctx.set_visuals(egui::Visuals::default()),
            ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
            ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
    }
    
    pub fn is_dark_mode(&self, ctx: &egui::Context) -> bool {
        ctx.style().visuals.dark_mode
    }
    
    // Унифицированная цветовая схема
    pub fn colors(&self, ctx: &egui::Context) -> ThemeColors {
        if self.is_dark_mode(ctx) {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        }
    }
}

pub struct ThemeColors {
    pub header: egui::Color32,
    pub text: egui::Color32,
    pub panel_bg: egui::Color32,
    pub central_bg: egui::Color32,
    pub button_bg: egui::Color32,
    pub button_border: egui::Color32,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            header: egui::Color32::WHITE,
            text: egui::Color32::from_gray(220),
            panel_bg: egui::Color32::from_gray(30),
            central_bg: egui::Color32::from_gray(35),
            button_bg: egui::Color32::from_gray(50),
            button_border: egui::Color32::from_gray(80),
        }
    }
    
    pub fn light() -> Self {
        Self {
            header: egui::Color32::BLACK,
            text: egui::Color32::from_gray(60),
            panel_bg: egui::Color32::from_gray(240),
            central_bg: egui::Color32::WHITE,
            button_bg: egui::Color32::from_gray(220),
            button_border: egui::Color32::from_gray(160),
        }
    }
    
    pub fn panel_shadow(&self) -> egui::Shadow {
        if self.panel_bg == ThemeColors::dark().panel_bg {
            // Тёмная тема
            egui::Shadow {
                offset: [2, 0],
                blur: 4,
                spread: 0,
                color: egui::Color32::from_black_alpha(80),
            }
        } else {
            // Светлая тема
            egui::Shadow {
                offset: [2, 0],
                blur: 6,
                spread: 0,
                color: egui::Color32::from_black_alpha(30),
            }
        }
    }
} 