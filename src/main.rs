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
        "Notes",
        options,
        Box::new(|cc| {
            // Создаём приложение без принудительной установки темы
            Ok(Box::new(App::new(cc)))
        })
    ).map_err(|e| AppError::Config(format!("Failed to start application: {}", e)))
}