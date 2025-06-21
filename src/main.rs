#![cfg_attr(all(not(debug_assertions), not(feature = "console")), windows_subsystem = "windows")]

mod notes;
mod error;
mod app;
mod state;
mod ui;
mod logging;
mod validation;
mod performance;

use eframe::egui;
use app::App;
use error::AppError;

fn main() -> Result<(), AppError> {
    // Инициализируем систему логирования
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();
    
    log::info!("Запуск приложения fast-bind");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
            .with_position([0.0, 0.0])
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(false),
        // Включаем поддержку IME для решения проблемы с кириллицей на Linux
        // Это помогает с обработкой UTF-8 символов в TextEdit виджетах
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    
    eframe::run_native(
        "Notes",
        options,
        Box::new(|cc| {
            // Настраиваем поддержку IME для решения проблемы с кириллицей
            #[cfg(target_os = "linux")]
            {
                // Включаем обработку UTF-8 событий клавиатуры
                cc.egui_ctx.input_mut(|i| {
                    // Принудительно включаем обработку text events
                    i.wants_raw_input = true;
                });
            }
            
            // Создаём приложение без принудительной установки темы
            Ok(Box::new(App::new(cc)))
        })
    ).map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to start application: {}", e))))
}