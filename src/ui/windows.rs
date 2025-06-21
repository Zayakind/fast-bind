use eframe::egui;
use uuid::Uuid;
use crate::notes::Note;
use crate::state::{AppState, UiState, LoadMode};
use crate::ui::{UiComponents, ThemeMode, SettingsActions};

/// Управление всеми окнами приложения
pub struct WindowManager;

impl WindowManager {
    
    /// Отображает окно настроек приложения
    pub fn show_settings_window(
        app_state: &AppState,
        ui_state: &mut UiState,
        ctx: &egui::Context,
    ) -> SettingsActions {
        let mut actions = SettingsActions::new();
        
        if ui_state.show_settings {
            egui::Window::new("Настройки")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Секция темы
                        ui.heading("🎨 Тема приложения");
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            if ui.radio_value(&mut ui_state.theme_mode, ThemeMode::Auto, "Автоматически").clicked() {
                                actions.change_theme(ui_state.theme_mode);
                            }
                            ui.label("(следовать системной теме)");
                        });
                        
                        if ui.radio_value(&mut ui_state.theme_mode, ThemeMode::Light, "Светлая тема").clicked() {
                            actions.change_theme(ui_state.theme_mode);
                        }
                        if ui.radio_value(&mut ui_state.theme_mode, ThemeMode::Dark, "Тёмная тема").clicked() {
                            actions.change_theme(ui_state.theme_mode);
                        }
                        
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(15.0);
                        
                        // Секция производительности
                        ui.heading("⚡ Производительность");
                        ui.add_space(10.0);
                        
                        ui.label("Режим загрузки заметок:");
                        ui.add_space(5.0);
                        
                        let old_mode = ui_state.preferred_load_mode.clone();
                        ui.horizontal(|ui| {
                            if ui.radio_value(&mut ui_state.preferred_load_mode, LoadMode::Auto, "🤖 Автоматически").clicked() {
                                actions.change_load_mode(ui_state.preferred_load_mode.clone());
                            }
                            ui.label("(оптимальный выбор)");
                        });
                        if ui.radio_value(&mut ui_state.preferred_load_mode, LoadMode::Eager, "📋 Загружать всё").clicked() {
                            actions.change_load_mode(ui_state.preferred_load_mode.clone());
                        }
                        if ui.radio_value(&mut ui_state.preferred_load_mode, LoadMode::Lazy, "⚡ Ленивая загрузка").clicked() {
                            actions.change_load_mode(ui_state.preferred_load_mode.clone());
                        }
                        
                        ui.add_space(10.0);
                        
                        let old_stats = ui_state.show_performance_stats;
                        if ui.checkbox(&mut ui_state.show_performance_stats, "📊 Показывать статистику производительности").clicked() {
                            if old_stats != ui_state.show_performance_stats {
                                actions.toggle_performance_stats(ui_state.show_performance_stats);
                            }
                        }
                        
                        // Отображаем текущую статистику
                        if ui_state.show_performance_stats {
                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.label("📈 Текущая статистика:");
                                ui.add_space(5.0);
                                
                                ui.horizontal(|ui| {
                                    ui.label("Режим:");
                                    ui.colored_label(
                                        egui::Color32::from_rgb(0, 150, 0),
                                        format!("{:?}", app_state.load_mode)
                                    );
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Загружено заметок:");
                                    ui.colored_label(
                                        egui::Color32::from_rgb(0, 100, 200),
                                        format!("{}", app_state.notes.len())
                                    );
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Общее количество:");
                                    ui.colored_label(
                                        egui::Color32::from_rgb(100, 100, 100),
                                        format!("{}", app_state.total_notes_count())
                                    );
                                });
                                
                                // Статистика ленивой загрузки
                                if let Some(stats) = app_state.get_loading_stats() {
                                    ui.horizontal(|ui| {
                                        ui.label("В кэше:");
                                        ui.colored_label(
                                            egui::Color32::from_rgb(150, 0, 150),
                                            format!("{}", stats.loaded_notes)
                                        );
                                    });
                                    
                                    ui.horizontal(|ui| {
                                        ui.label("Эффективность кэша:");
                                        ui.colored_label(
                                            egui::Color32::from_rgb(200, 100, 0),
                                            format!("{:.1}%", stats.cache_hit_ratio * 100.0)
                                        );
                                    });
                                }
                            });
                        }
                        
                        // Описания режимов
                        ui.add_space(10.0);
                        ui.collapsing("ℹ️ Описание режимов", |ui| {
                            ui.label("🤖 Автоматически: Выбирает оптимальный режим в зависимости от количества заметок");
                            ui.label("📋 Загружать всё: Загружает все заметки при запуске (подходит для <100 заметок)");
                            ui.label("⚡ Ленивая загрузка: Загружает заметки по мере необходимости (для больших коллекций)");
                        });
                        
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        if ui.button("Закрыть").clicked() {
                            actions.close();
                        }
                    });
                });
        }
        
        actions
    }
    
    /// Отображает окно создания группы
    pub fn show_group_creation_window(
        app_state: &AppState,
        ui_state: &mut UiState,
        ctx: &egui::Context,
        on_create_group: impl FnOnce(String, Option<Uuid>, Vec<Uuid>),
    ) -> bool {
        let mut should_create = false;
        
        if ui_state.show_group_creation {
            let window_title = if ui_state.creating_subgroup_for.is_some() {
                "Создать подгруппу"
            } else {
                "Создать группу"
            };

            egui::Window::new(window_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // Информация о родительской группе
                    if let Some(parent_id) = ui_state.creating_subgroup_for {
                        if let Some(parent_group) = app_state.groups.iter().find(|g| g.id == parent_id) {
                            ui.label(format!("Родительская группа: {}", parent_group.name));
                            ui.separator();
                        }
                    }

                    ui.label("Имя группы:");
                    UiComponents::text_field_frame().show(ui, |ui| {
                        UiComponents::single_line_text_edit(
                            ui,
                            &mut ui_state.new_group_name,
                            "Название группы",
                            ui.available_width(),
                            24.0
                        );
                    });
                    ui.add_space(10.0);
                    
                    // Выбор родительской группы (только если не создаём подгруппу)
                    if ui_state.creating_subgroup_for.is_none() {
                        ui.label("Родительская группа:");
                        UiComponents::group_selector(
                            ui,
                            ui_state.new_note_parent_group_id,
                            &app_state.groups,
                            "parent_group_select",
                            |group_id| ui_state.new_note_parent_group_id = group_id,
                            20
                        );
                    }
                    
                    ui.separator();
                    
                    // Выбор заметок для добавления в группу
                    ui.label("Добавить заметки в группу:");
                    let ungrouped_notes: Vec<&Note> = app_state.notes.iter()
                        .filter(|note| note.group_id.is_none())
                        .collect();
                    
                    if ungrouped_notes.is_empty() {
                        ui.label("Нет заметок без группы");
                    } else {
                        for note in ungrouped_notes {
                            let mut checked = ui_state.group_creation_selected_notes.contains(&note.id);
                            if ui.checkbox(&mut checked, &note.title).changed() {
                                if checked {
                                    ui_state.group_creation_selected_notes.push(note.id);
                                } else {
                                    ui_state.group_creation_selected_notes.retain(|id| id != &note.id);
                                }
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Создать").clicked() && !ui_state.new_group_name.trim().is_empty() {
                            should_create = true;
                        }
                        
                        if ui.button("Отмена").clicked() {
                            ui_state.clear_group_form();
                        }
                    });
                });
        }
        
        if should_create {
            let parent_id = if ui_state.creating_subgroup_for.is_some() {
                ui_state.creating_subgroup_for
            } else {
                ui_state.new_note_parent_group_id
            };
            
            on_create_group(
                ui_state.new_group_name.trim().to_string(),
                parent_id,
                ui_state.group_creation_selected_notes.clone()
            );
            ui_state.clear_group_form();
        }
        
        should_create
    }
    
    /// Отображает окно редактора групп
    pub fn show_group_editor_window(
        app_state: &AppState,
        ui_state: &mut UiState,
        ctx: &egui::Context,
    ) -> (Vec<(Uuid, String, Option<Uuid>)>, Vec<Uuid>) {
        let mut groups_to_save = Vec::new();
        let mut groups_to_delete = Vec::new();
        if ui_state.show_group_editor {
            egui::Window::new("Редактор групп")
                .collapsible(false)
                .resizable(true)
                .min_width(400.0)
                .min_height(300.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Управление группами");
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                                            .show(ui, |ui| {
                        let mut groups_data: Vec<(Uuid, String, Option<Uuid>, u32)> = app_state.groups.iter()
                            .map(|g| (g.id, g.name.clone(), g.parent_id, g.level))
                            .collect();
                        
                        groups_data.sort_by(|a, b| a.3.cmp(&b.3).then(a.1.cmp(&b.1)));
                        
                        for (group_id, group_name, parent_id, level) in groups_data {
                                ui.horizontal(|ui| {
                                    // Отступ для уровня вложенности
                                    ui.add_space(level as f32 * 20.0);
                                    
                                    if ui_state.editing_group_id == Some(group_id) {
                                        // Режим редактирования
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Имя:");
                                                UiComponents::text_field_frame().show(ui, |ui| {
                                                    UiComponents::single_line_text_edit(
                                                        ui,
                                                        &mut ui_state.editing_group_name,
                                                        "Имя группы",
                                                        ui.available_width(),
                                                        20.0
                                                    );
                                                });
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Родитель:");
                                                UiComponents::group_selector(
                                                    ui,
                                                    ui_state.editing_group_parent_id,
                                                    &app_state.groups,
                                                    &format!("edit_parent_{}", group_id),
                                                    |parent_id| ui_state.editing_group_parent_id = parent_id,
                                                    20
                                                );
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                if ui.small_button("✔️").on_hover_text("Сохранить").clicked() {
                                                    groups_to_save.push((group_id, ui_state.editing_group_name.clone(), ui_state.editing_group_parent_id));
                                                }
                                                if ui.small_button("❌").on_hover_text("Отмена").clicked() {
                                                    ui_state.editing_group_id = None;
                                                    ui_state.editing_group_name.clear();
                                                    ui_state.editing_group_parent_id = None;
                                                }
                                            });
                                        });
                                    } else {
                                        // Режим просмотра
                                        ui.label(&group_name);
                                        
                                        // Показываем родительскую группу
                                        if let Some(parent_id) = parent_id {
                                            if let Some(parent) = app_state.groups.iter().find(|g| g.id == parent_id) {
                                                ui.label(format!("(в: {})", parent.name));
                                            }
                                        } else {
                                            ui.label("(корневая)");
                                        }
                                        
                                        // Количество заметок в группе
                                        let notes_count = app_state.notes.iter().filter(|n| n.group_id == Some(group_id)).count();
                                        ui.label(format!("({} заметок)", notes_count));
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Кнопка удаления
                                            if ui.small_button("🗑").on_hover_text("Удалить группу").clicked() {
                                                groups_to_delete.push(group_id);
                                            }
                                            
                                            // Кнопка редактирования
                                            if ui.small_button("✏️").on_hover_text("Редактировать группу").clicked() {
                                                ui_state.editing_group_id = Some(group_id);
                                                ui_state.editing_group_name = group_name.clone();
                                                ui_state.editing_group_parent_id = parent_id;
                                            }
                                        });
                                    }
                                });
                                ui.separator();
                            }
                                                });
                
                ui.separator();
                
                if ui.button("Закрыть").clicked() {
                    ui_state.show_group_editor = false;
                }
            });
        }
        
        // Очищаем состояние редактирования при сохранении
        if !groups_to_save.is_empty() {
            ui_state.editing_group_id = None;
            ui_state.editing_group_name.clear();
            ui_state.editing_group_parent_id = None;
        }
        
        (groups_to_save, groups_to_delete)
    }
} 