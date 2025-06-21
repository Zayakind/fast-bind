use eframe::egui;
use uuid::Uuid;

use crate::state::{AppState, UiState};
use crate::ui::{theme::ThemeColors, UiComponents};
use crate::ui::panel_actions::{SidePanelActions, CentralPanelActions};

/// Управление панелями интерфейса
pub struct PanelManager;

impl PanelManager {
    /// Упрощенная боковая панель без замыканий (возвращает действия)
    pub fn show_side_panel_simple(
        app_state: &AppState,
        ui_state: &mut UiState,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
    ) -> SidePanelActions {
        let mut actions = SidePanelActions::new();
        let panel_width = ui.available_width() * 0.30;
        
        egui::SidePanel::left("notes_panel")
            .frame(egui::Frame {
                fill: colors.panel_bg,
                stroke: egui::Stroke::NONE,
                inner_margin: egui::Margin::same(8),
                shadow: colors.panel_shadow(),
                ..Default::default()
            })
            .resizable(false)
            .min_width(panel_width)
            .max_width(panel_width)
            .show_inside(ui, |ui| {
                let scroll_area = egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                    .auto_shrink([false, false]);
                
                let scroll_output = scroll_area.show(ui, |ui| {
                    // Заголовок с кнопками и статистикой
                    ui.horizontal(|ui| {
                        // Показываем статистику загрузки
                        let loaded_count = app_state.notes.len();
                        let total_count = app_state.total_notes_count();
                        
                        let title_text = if loaded_count == total_count {
                            format!("Заметки ({})", total_count)
                        } else {
                            format!("Заметки ({}/{})", loaded_count, total_count)
                        };
                        
                        ui.add(egui::Label::new(
                            egui::RichText::new(title_text)
                                .size(18.0)
                                .strong()
                                .color(colors.header)
                        ));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if UiComponents::icon_button(
                                ui, "⚙", "Настройки приложения",
                                colors.button_bg, colors.button_border
                            ).clicked() {
                                actions.show_settings();
                            }
                            
                            if UiComponents::icon_button(
                                ui, "📁", "Редактор групп",
                                colors.button_bg, colors.button_border
                            ).clicked() {
                                actions.show_group_editor();
                            }
                            
                            // Индикатор режима загрузки
                            if ui_state.show_performance_stats {
                                let mode_icon = match app_state.load_mode {
                                    crate::state::LoadMode::Auto => "🤖",
                                    crate::state::LoadMode::Eager => "📋", 
                                    crate::state::LoadMode::Lazy => "⚡",
                                };
                                ui.add(egui::Label::new(mode_icon).sense(egui::Sense::hover()))
                                    .on_hover_text(format!("Режим загрузки: {:?}", app_state.load_mode));
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    
                    // Кнопки управления
                    if ui.add_sized([panel_width - 12.0, 32.0], egui::Button::new("+ Новая заметка")).clicked() {
                        actions.new_note();
                    }
                    
                    ui.add_space(6.0);
                    
                    if ui.add_sized([panel_width - 12.0, 32.0], egui::Button::new("Создать группу")).clicked() {
                        actions.create_group();
                    }
                    
                    ui.add_space(10.0);

                    // Отображение групп и заметок с подсчетом видимых элементов
                    let notes_start_y = ui.cursor().top();
                    
                    Self::show_groups_and_notes_simple(
                        app_state, ui_state, ui, colors, panel_width, &mut actions
                    );
                    
                    let notes_end_y = ui.cursor().top();
                    
                    // Индикатор загрузки для ленивого режима
                    if app_state.load_mode == crate::state::LoadMode::Lazy && 
                       app_state.notes.len() < app_state.total_notes_count() {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space((panel_width - 200.0) / 2.0);
                            ui.add(egui::Spinner::new());
                            ui.add_space(8.0);
                            ui.label("Загружается...");
                        });
                        ui.add_space(10.0);
                    }
                    
                    (notes_start_y, notes_end_y)
                });
                
                // Проверяем необходимость дозагрузки при ленивой загрузке
                if app_state.load_mode == crate::state::LoadMode::Lazy {
                    let scroll_offset = scroll_output.state.offset.y;
                    let viewport_height = scroll_output.inner_rect.height();
                    let content_height = scroll_output.content_size.y;
                    
                    // Если пользователь приблизился к концу контента (остается менее 20% для прокрутки)
                    let scroll_threshold = 0.8;
                    if content_height > 0.0 && 
                       (scroll_offset + viewport_height) / content_height > scroll_threshold {
                        
                        // Примерно вычисляем видимый диапазон заметок
                        let total_notes = app_state.total_notes_count();
                        if total_notes > 0 {
                            let visible_start = (scroll_offset / 40.0) as usize; // Примерно 40px на заметку
                            let visible_count = (viewport_height / 40.0) as usize + 5; // Буфер 5 заметок
                            let visible_end = (visible_start + visible_count).min(total_notes);
                            
                            actions.request_load_more(visible_start, visible_end);
                        }
                    }
                }
            });
            
        actions
    }
    
    /// Упрощенная центральная панель без замыканий (возвращает действия)
    pub fn show_central_panel_simple(
        app_state: &mut AppState,
        ui_state: &mut UiState,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
    ) -> CentralPanelActions {
        let mut actions = CentralPanelActions::new();
        
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: colors.central_bg,
                stroke: egui::Stroke::NONE,
                inner_margin: egui::Margin::same(8),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    if let Some(idx) = ui_state.selected_note {
                        if idx < app_state.notes.len() {
                            Self::show_note_editor_simple(
                                app_state, ui_state, ui, idx, colors, &mut actions
                            );
                        }
                    } else {
                        Self::show_note_creation_form_simple(app_state, ui_state, ui, &mut actions);
                    }
                    
                    // Постоянное текстовое поле
                    ui.add_space(12.0);
                    Self::show_persistent_text_field_simple(app_state, ui, &mut actions);
                });
            });
            
        actions
    }
    
    /// Упрощенное отображение групп и заметок
    fn show_groups_and_notes_simple(
        app_state: &AppState,
        ui_state: &UiState,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
        panel_width: f32,
        actions: &mut SidePanelActions,
    ) {
        // Показываем корневые группы
        let root_groups: Vec<(Uuid, String, bool)> = app_state.groups.iter()
            .filter(|g| g.parent_id.is_none())
            .map(|g| (g.id, g.name.clone(), g.collapsed))
            .collect();
        
        for (group_id, group_name, collapsed) in root_groups {
            if let Some(toggle_id) = Self::show_group_with_hierarchy_simple(
                app_state, ui_state, ui, colors, group_id, &group_name, 
                collapsed, 0, panel_width, actions
            ) {
                actions.toggle_group(toggle_id);
            }
        }

        // Заметки без группы
        Self::show_ungrouped_notes_simple(app_state, ui_state, ui, colors, panel_width, actions);
    }
    
    /// Упрощенное отображение группы с иерархией
    fn show_group_with_hierarchy_simple(
        app_state: &AppState,
        ui_state: &UiState,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
        group_id: Uuid,
        group_name: &str,
        collapsed: bool,
        depth: usize,
        panel_width: f32,
        actions: &mut SidePanelActions,
    ) -> Option<Uuid> {
        let mut toggled_group: Option<Uuid> = None;
        
        // Отступ для вложенности
        if depth > 0 {
            ui.add_space(depth as f32 * 15.0);
        }
        
        let note_indices: Vec<usize> = app_state.notes.iter().enumerate()
            .filter(|(_, n)| n.group_id == Some(group_id))
            .map(|(i, _)| i)
            .collect();
        
        let header = egui::CollapsingHeader::new(
            egui::RichText::new(group_name)
                .color(colors.header)
                .strong()
        )
            .id_salt(group_id)
            .default_open(!collapsed)
            .show(ui, |ui| {
                // Показываем заметки в этой группе
                for idx in note_indices {
                    if let Some(note) = app_state.notes.get(idx) {
                        let is_selected = ui_state.selected_note == Some(idx);
                        let mut title = String::new();
                        if note.pinned { 
                            title.push_str("📌 "); 
                        }
                        title.push_str(&note.title);
                        
                        let label = egui::SelectableLabel::new(is_selected, 
                            egui::RichText::new(title).color(colors.text)
                        );
                        
                        if ui.add_sized([panel_width - 20.0, 28.0], label).clicked() {
                            actions.select_note(idx);
                        }
                        ui.add_space(4.0);
                    }
                }
                
                // Показываем подгруппы
                let subgroups: Vec<(Uuid, String, bool)> = app_state.groups.iter()
                    .filter(|g| g.parent_id == Some(group_id))
                    .map(|g| (g.id, g.name.clone(), g.collapsed))
                    .collect();
                
                for (sub_id, sub_name, sub_collapsed) in subgroups {
                    if let Some(sub_toggle) = Self::show_group_with_hierarchy_simple(
                        app_state, ui_state, ui, colors, sub_id, &sub_name, 
                        sub_collapsed, depth + 1, panel_width, actions
                    ) {
                        toggled_group = Some(sub_toggle);
                    }
                }
            });
        
        if header.header_response.clicked() {
            return Some(group_id);
        }
        
        toggled_group
    }
    
    /// Упрощенное отображение заметок без группы
    fn show_ungrouped_notes_simple(
        app_state: &AppState,
        ui_state: &UiState,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
        panel_width: f32,
        actions: &mut SidePanelActions,
    ) {
        let no_group_notes: Vec<_> = app_state.notes.iter().enumerate()
            .filter(|(_, n)| n.group_id.is_none())
            .collect();
        
        if !no_group_notes.is_empty() {
            egui::CollapsingHeader::new(
                egui::RichText::new("Без группы")
                    .color(colors.header)
                    .strong()
            )
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, note) in no_group_notes {
                        let is_selected = ui_state.selected_note == Some(idx);
                        let mut title = String::new();
                        if note.pinned { 
                            title.push_str("📌 "); 
                        }
                        title.push_str(&note.title);
                        
                        let label = egui::SelectableLabel::new(is_selected, 
                            egui::RichText::new(title).color(colors.text)
                        );
                        
                        if ui.add_sized([panel_width - 20.0, 28.0], label).clicked() {
                            actions.select_note(idx);
                        }
                        ui.add_space(4.0);
                    }
                });
        }
    }
    
    /// Упрощенный редактор заметки
    fn show_note_editor_simple(
        app_state: &AppState,
        ui_state: &mut UiState,
        ui: &mut egui::Ui,
        idx: usize,
        colors: &ThemeColors,
        actions: &mut CentralPanelActions,
    ) {
        let note = &app_state.notes[idx];
        
        ui.add_space(6.0);
        
        // Заголовок заметки с возможностью редактирования
        ui.horizontal(|ui| {
            // Кнопка закрепить/открепить
            let pin_icon = if note.pinned { "📌" } else { "📍" };
            if ui.add(egui::Label::new(pin_icon).sense(egui::Sense::click()))
                .on_hover_text(if note.pinned { "Открепить" } else { "Закрепить" })
                .clicked() 
            {
                actions.toggle_pin(idx);
            }
            
            if ui_state.editing_title == Some(idx) {
                let mut title = note.title.clone();
                let response = UiComponents::single_line_text_edit(
                    ui, 
                    &mut title, 
                    "Заголовок заметки", 
                    ui.available_width() - 36.0, 
                    32.0
                );
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || 
                   response.lost_focus() && !response.has_focus() 
                {
                    if !title.trim().is_empty() {
                        actions.update_title(idx, title.clone());
                    }
                    ui_state.editing_title = None;
                }
                
                if ui.add(egui::Button::new("✔️")).on_hover_text("Сохранить").clicked() {
                    if !title.trim().is_empty() {
                        actions.update_title(idx, title.clone());
                    }
                    ui_state.editing_title = None;
                }
            } else {
                ui.label(
                    egui::RichText::new(&note.title)
                        .size(20.0)
                        .strong()
                );
                if ui.add(egui::Label::new("📝").sense(egui::Sense::click()))
                    .on_hover_text("Редактировать заголовок")
                    .clicked() 
                {
                    ui_state.editing_title = Some(idx);
                }
            }
        });
        
        ui.add_space(6.0);
        
        // Содержимое заметки
        if ui_state.editing_content == Some(idx) {
            // Режим редактирования
            UiComponents::multiline_text_edit(
                ui, 
                &mut ui_state.new_note_content, 
                "Текст заметки..."
            );
        } else {
            // Режим просмотра
            let mut readonly_content = note.content.clone();
            ui.add_enabled_ui(false, |ui| {
                UiComponents::multiline_text_edit(ui, &mut readonly_content, "");
            });
        }
        
        ui.add_space(8.0);
        
        // Кнопки действий
        ui.horizontal(|ui| {
            if ui_state.editing_content == Some(idx) {
                // Режим редактирования
                if ui.add_sized([80.0, 32.0], egui::Button::new("💾 Сохранить")).clicked() {
                    actions.save_note();
                }
                ui.add_space(8.0);
                if ui.add_sized([80.0, 32.0], egui::Button::new("❌ Отмена")).clicked() {
                    ui_state.stop_editing();
                }
            } else {
                // Режим просмотра
                if ui.add_sized([90.0, 32.0], egui::Button::new("📋 Копировать")).clicked() {
                    actions.copy_to_clipboard();
                }
                ui.add_space(8.0);
                
                if ui.add_sized([110.0, 32.0], egui::Button::new("📄 В заметки")).clicked() {
                    actions.copy_to_persistent();
                }
                ui.add_space(8.0);
                
                if ui.add_sized([110.0, 32.0], egui::Button::new("📝 Редактировать")).clicked() {
                    ui_state.editing_content = Some(idx);
                }
                ui.add_space(8.0);
                
                if ui.add_sized([80.0, 32.0], egui::Button::new("🗑 Удалить")).clicked() {
                    actions.delete_note();
                }
            }
        });
    }
    
    /// Упрощенная форма создания заметки
    fn show_note_creation_form_simple(
        app_state: &AppState,
        ui_state: &mut UiState,
        ui: &mut egui::Ui,
        actions: &mut CentralPanelActions,
    ) {
        ui.vertical(|ui| {
            // Поле заголовка
            UiComponents::text_field_frame().show(ui, |ui| {
                UiComponents::single_line_text_edit(
                    ui,
                    &mut ui_state.new_note_title,
                    "Заголовок заметки",
                    ui.available_width(),
                    28.0
                );
            });
            
            ui.add_space(10.0);
            
            // Поле содержимого
            UiComponents::multiline_text_edit(
                ui,
                &mut ui_state.new_note_content,
                "Текст заметки..."
            );
            
            ui.add_space(10.0);
            
            // Выбор группы для новой заметки
            ui.horizontal(|ui| {
                ui.label("Группа:");
                UiComponents::group_selector(
                    ui,
                    ui_state.new_note_group_id,
                    &app_state.groups,
                    "new_note_group_select",
                    |group_id| ui_state.new_note_group_id = group_id,
                    20
                );
            });
            
            ui.add_space(16.0);
            
            // Кнопка создания
            if ui.add_sized([120.0, 36.0], egui::Button::new("Создать")).clicked() {
                actions.create_note();
            }
        });
    }
    
    /// Упрощенное постоянное текстовое поле
    fn show_persistent_text_field_simple(
        app_state: &mut AppState,
        ui: &mut egui::Ui,
        actions: &mut CentralPanelActions,
    ) {
        ui.separator();
        ui.add_space(8.0);
        
        ui.label(
            egui::RichText::new("Черновик")
                .size(16.0)
                .strong()
        );
        ui.add_space(6.0);
        
        let response = UiComponents::multiline_text_edit(
            ui,
            &mut app_state.persistent_text,
            "Здесь вы можете писать заметки, которые сохраняются при переключении между заметками..."
        );
        
        // Автосохранение при изменении текста
        if response.changed() {
            actions.persistent_text_changed();
        }
    }
} 