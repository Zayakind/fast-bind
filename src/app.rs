use eframe::egui;
use std::path::PathBuf;
use clipboard::{ClipboardContext, ClipboardProvider};

use crate::notes::NotesManager;
use crate::state::{AppState, UiState, LoadMode};
use crate::ui::{AppTheme, WindowManager, PanelManager, ThemeMode, SidePanelActions, CentralPanelActions, SettingsActions};
use uuid::Uuid;

/// Упрощенная главная структура приложения после рефакторинга
pub struct App {
    app_state: AppState,
    ui_state: UiState,
    theme: AppTheme,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Настраиваем поддержку IME для кириллицы на Linux
        #[cfg(target_os = "linux")]
        {
            _cc.egui_ctx.style_mut(|style| {
                style.interaction.tooltip_delay = 0.0;
                style.interaction.selectable_labels = true;
            });
            _cc.egui_ctx.set_ime_cursor_pos(egui::Vec2::ZERO);
        }
        
        // Создаем менеджер заметок
        let notes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".fast-bind")
            .join("data");
        
        let notes_manager = NotesManager::new(notes_dir);
        
        Self {
            app_state: AppState::new(notes_manager),
            ui_state: UiState::new(),
            theme: AppTheme::new(),
        }
    }
    
    /// Создает новую заметку
    fn create_note(&mut self) {
        match self.app_state.create_note(
            self.ui_state.new_note_title.clone(),
            self.ui_state.new_note_content.clone(),
            self.ui_state.new_note_group_id,
        ) {
            Ok(_note_id) => {
                self.ui_state.clear_note_form();
            }
            Err(e) => {
                eprintln!("Ошибка создания заметки: {}", e);
            }
        }
    }
    
    /// Удаляет выбранную заметку
    fn delete_selected_note(&mut self) {
        if let Some(idx) = self.ui_state.selected_note {
            if let Err(e) = self.app_state.delete_note(idx) {
                eprintln!("Ошибка удаления заметки: {}", e);
            } else {
                self.ui_state.selected_note = None;
            }
        }
    }
    
    /// Копирует заметку в буфер обмена
    fn copy_note_to_clipboard(&self) {
        if let Some(idx) = self.ui_state.selected_note {
            if let Some(content) = self.app_state.get_note_content(idx) {
                if let Ok(mut ctx) = ClipboardContext::new() {
                    let _ = ctx.set_contents(content);
                }
            }
        }
    }
    
    /// Копирует заметку в постоянное текстовое поле
    fn copy_note_to_persistent(&mut self) {
        if let Some(idx) = self.ui_state.selected_note {
            if let Err(e) = self.app_state.append_note_to_persistent(idx) {
                eprintln!("Ошибка копирования в постоянный текст: {}", e);
            }
        }
    }
    
    /// Сохраняет изменения заметки
    fn save_note_changes(&mut self) {
        if let Some(idx) = self.ui_state.selected_note {
            if let Err(e) = self.app_state.update_note(
                idx,
                None, // title не изменяем
                Some(self.ui_state.new_note_content.clone()),
            ) {
                eprintln!("Ошибка сохранения заметки: {}", e);
            } else {
                self.ui_state.stop_editing();
            }
        }
    }
    
    /// Переключает закрепление заметки
    fn toggle_pin(&mut self, index: usize) {
        if let Err(e) = self.app_state.toggle_pin(index) {
            eprintln!("Ошибка переключения закрепления: {}", e);
        }
    }
    
    /// Выбирает заметку для просмотра/редактирования
    fn select_note(&mut self, index: usize) {
        if let Some(note) = self.app_state.notes.get(index) {
            self.ui_state.selected_note = Some(index);
            self.ui_state.new_note_title = note.title.clone();
            self.ui_state.new_note_content = note.content.clone();
        }
    }
    
    /// Обновляет заголовок заметки
    fn update_note_title(&mut self, index: usize, title: String) {
        if let Err(e) = self.app_state.update_note(index, Some(title), None) {
            eprintln!("Ошибка обновления заголовка: {}", e);
        }
    }
    
    /// Переключает сворачивание группы
    fn toggle_group_collapsed(&mut self, group_id: Uuid) {
        if let Err(e) = self.app_state.toggle_group_collapsed(group_id) {
            eprintln!("Ошибка переключения группы: {}", e);
        }
    }
    
    /// Обрабатывает действия боковой панели
    fn handle_side_panel_actions(&mut self, actions: SidePanelActions) {
        if let Some(idx) = actions.selected_note {
            self.select_note(idx);
        }
        
        if actions.new_note_clicked {
            self.ui_state.clear_note_form();
        }
        
        if actions.create_group_clicked {
            self.ui_state.show_group_creation = true;
            self.ui_state.new_group_name.clear();
            self.ui_state.group_creation_selected_notes.clear();
        }
        
        if actions.show_settings_clicked {
            self.ui_state.show_settings = true;
        }
        
        if actions.show_group_editor_clicked {
            self.ui_state.show_group_editor = true;
        }
        
        if let Some(group_id) = actions.toggled_group {
            self.toggle_group_collapsed(group_id);
        }
        
        // Обработка запроса дозагрузки для ленивой загрузки
        if let Some((visible_start, visible_end)) = actions.load_more_requested {
            if let Err(e) = self.app_state.load_more_if_needed((visible_start, visible_end)) {
                eprintln!("Ошибка дозагрузки заметок: {}", e);
            }
        }
    }
    
    /// Обрабатывает действия центральной панели
    fn handle_central_panel_actions(&mut self, actions: CentralPanelActions) {
        if actions.save_note_clicked {
            self.save_note_changes();
        }
        
        if actions.delete_note_clicked {
            self.delete_selected_note();
        }
        
        if actions.copy_to_clipboard_clicked {
            self.copy_note_to_clipboard();
        }
        
        if actions.copy_to_persistent_clicked {
            self.copy_note_to_persistent();
        }
        
        if let Some(idx) = actions.toggle_pin {
            self.toggle_pin(idx);
        }
        
        if let Some((idx, title)) = actions.update_title {
            self.update_note_title(idx, title);
        }
        
        if actions.create_note_clicked {
            self.create_note();
        }
        
        if actions.persistent_text_changed {
            if let Err(e) = self.app_state.save_persistent_text() {
                eprintln!("Ошибка сохранения постоянного текста: {}", e);
            }
        }
    }
    
    /// Отображает основной UI через модульные компоненты
    fn show_main_ui(&mut self, ctx: &egui::Context) {
        let colors = self.theme.colors(ctx);
        
        // Устанавливаем фон контекста
        ctx.style_mut(|style| {
            style.visuals.panel_fill = colors.central_bg;
            style.visuals.window_fill = colors.central_bg;
            style.visuals.extreme_bg_color = colors.central_bg;
        });
        
        // Главная панель 
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: colors.central_bg,
                stroke: egui::Stroke::NONE,
                inner_margin: egui::Margin::ZERO,
                ..Default::default()
            })
            .show(ctx, |ui| {
                // Используем модульные компоненты вместо дублированного кода
                let side_actions = PanelManager::show_side_panel_simple(
                    &self.app_state,
                    &mut self.ui_state,
                    ui,
                    &colors,
                );
                
                let central_actions = PanelManager::show_central_panel_simple(
                    &mut self.app_state,
                    &mut self.ui_state,
                    ui,
                    &colors,
                );
                
                // Обрабатываем действия боковой панели
                self.handle_side_panel_actions(side_actions);
                
                // Обрабатываем действия центральной панели
                self.handle_central_panel_actions(central_actions);
            });
    }

    /// Отображает все окна приложения через WindowManager
    fn show_windows(&mut self, ctx: &egui::Context) {
        // Окно настроек - используем модульную версию
        let settings_actions = WindowManager::show_settings_window(
            &self.app_state,
            &mut self.ui_state,
            ctx,
        );
        self.handle_settings_actions(settings_actions);
        
        // Окно создания группы - используем модульную версию
        let mut group_creation_data: Option<(String, Option<Uuid>, Vec<Uuid>)> = None;
        
        WindowManager::show_group_creation_window(
            &self.app_state,
            &mut self.ui_state,
            ctx,
            |name, parent_id, selected_notes| {
                group_creation_data = Some((name, parent_id, selected_notes));
            },
        );
        
        // Обрабатываем создание группы после возврата управления
        if let Some((name, parent_id, selected_notes)) = group_creation_data {
            if let Err(e) = self.app_state.create_group(name, parent_id, selected_notes) {
                eprintln!("Ошибка создания группы: {}", e);
            }
        }
        
        // Окно редактора групп - используем модульную версию
        let (groups_to_save, groups_to_delete) = WindowManager::show_group_editor_window(
            &self.app_state,
            &mut self.ui_state,
            ctx,
        );
        
        // Обрабатываем изменения после возврата управления
        for group_id in groups_to_delete {
            if let Err(e) = self.app_state.delete_group(group_id) {
                eprintln!("Ошибка удаления группы: {}", e);
            }
        }
        
        for (group_id, name, parent_id) in groups_to_save {
            if let Err(e) = self.app_state.update_group(group_id, name, parent_id) {
                eprintln!("Ошибка обновления группы: {}", e);
            }
        }
    }
    
    /// Обрабатывает действия окна настроек
    fn handle_settings_actions(&mut self, actions: SettingsActions) {
        if let Some(theme) = actions.theme_changed {
            // Тема будет применена в update()
            self.ui_state.theme_mode = theme;
        }
        
        if let Some(load_mode) = actions.load_mode_changed {
            if let Err(e) = self.app_state.switch_load_mode(load_mode) {
                eprintln!("Ошибка переключения режима загрузки: {}", e);
            }
        }
        
        if let Some(show_stats) = actions.show_performance_stats_changed {
            self.ui_state.show_performance_stats = show_stats;
        }
        
        if actions.close_settings {
            self.ui_state.show_settings = false;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Применяем тему
        self.theme.mode = self.ui_state.theme_mode;
        self.theme.apply(ctx);
        
        // Отображаем основной UI
        self.show_main_ui(ctx);
        
        // Упрощенные окна без сложных замыканий
        self.show_windows(ctx);
    }
} 