#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod notes;
mod ssh;
mod error;
mod config;
mod app;

use eframe::egui;
use app::App;
use error::AppError;

fn main() -> Result<(), AppError> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
            .with_position([0.0, 0.0])
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "Mimir Notes",
        options,
        Box::new(|cc| {
            // Настройка стиля приложения
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.window_rounding = 5.0.into();
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 40);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(80, 80, 80);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(100, 100, 100);
            cc.egui_ctx.set_style(style);
            
            Box::new(App::new(cc))
        })
    ).map_err(|e| AppError::Config(format!("Failed to start application: {}", e)))
}