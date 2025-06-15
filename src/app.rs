use eframe::egui;
use crate::notes::{Note, NotesManager};
use crate::error::AppError;
use std::path::PathBuf;
use uuid::Uuid;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::fs;
use serde_json;

pub struct App {
    notes: Vec<Note>,
    selected_note: Option<usize>,
    new_note_title: String,
    new_note_content: String,
    notes_manager: NotesManager,
    right_panel_width: f32, // ширина правой панели
    editing_title: Option<usize>, // индекс редактируемого заголовка
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Создаем директорию для заметок в домашней директории пользователя
        let notes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".mimir")
            .join("notes");
        
        let notes_manager = NotesManager::new(notes_dir);
        
        // Загружаем существующие заметки
        let notes = notes_manager.get_all_notes().unwrap_or_default();
        let mut notes = notes;
        notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
        
        let default_window_width = 1024.0;
        let left_panel_width = (default_window_width * 0.20) as f32;
        Self {
            notes,
            selected_note: None,
            new_note_title: String::new(),
            new_note_content: String::new(),
            notes_manager,
            right_panel_width: left_panel_width,
            editing_title: None,
        }
    }
    
    fn create_new_note(&mut self) {
        if !self.new_note_title.is_empty() {
            let note = Note {
                id: Uuid::new_v4(),
                title: self.new_note_title.clone(),
                content: self.new_note_content.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                pinned: false,
            };
            
            if let Ok(()) = self.notes_manager.save_note(&note) {
                self.notes.push(note);
                self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                self.new_note_title.clear();
                self.new_note_content.clear();
            }
        }
    }
    
    fn delete_selected_note(&mut self) {
        if let Some(idx) = self.selected_note {
            if idx < self.notes.len() {
                let note_id = self.notes[idx].id;
                if let Ok(()) = self.notes_manager.delete_note(note_id) {
                    self.notes.remove(idx);
                    self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                    self.selected_note = None;
                }
            }
        }
    }
    
    fn copy_note_to_clipboard(&self) {
        if let Some(idx) = self.selected_note {
            if let Some(note) = self.notes.get(idx) {
                if let Ok(mut ctx) = ClipboardContext::new() {
                    let _ = ctx.set_contents(note.content.clone());
                }
            }
        }
    }
    
    fn save_note_changes(&mut self) {
        if let Some(idx) = self.selected_note {
            if idx < self.notes.len() {
                let old_note = &self.notes[idx];
                let updated_note = Note {
                    id: old_note.id,
                    title: old_note.title.clone(),
                    content: self.new_note_content.clone(),
                    created_at: old_note.created_at,
                    updated_at: chrono::Utc::now(),
                    pinned: old_note.pinned,
                };
                
                if let Ok(()) = self.notes_manager.save_note(&updated_note) {
                    self.notes[idx] = updated_note;
                    self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let splitter_width = 6.0;
            let min_left_width = 160.0;
            let max_left_width = 400.0;
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let left_width = self.right_panel_width;
            let right_width = available_width - left_width - splitter_width - 48.0; // 48px под колонку кнопок

            ui.horizontal(|ui| {
                // Левая панель (заметки)
                ui.vertical(|ui| {
                    ui.set_width(left_width);
                    ui.add_space(10.0);
                    ui.heading("Заметки");
                    ui.add_space(10.0);
                    if ui.add_sized([left_width - 20.0, 32.0], egui::Button::new("+ Новая заметка")).clicked() {
                        self.selected_note = None;
                        self.new_note_title.clear();
                        self.new_note_content.clear();
                    }
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        for (idx, note) in self.notes.iter().enumerate() {
                            let is_selected = self.selected_note == Some(idx);
                            let mut title = String::new();
                            if note.pinned {
                                title.push_str("📌");
                            }
                            title.push_str(&note.title);
                            if ui.add_sized([
                                left_width - 20.0, 28.0],
                                egui::SelectableLabel::new(is_selected, title)
                            ).clicked() {
                                self.selected_note = Some(idx);
                                self.new_note_title = note.title.clone();
                                self.new_note_content = note.content.clone();
                            }
                            ui.add_space(4.0);
                        }
                    });
                });

                // Разделитель
                let splitter_rect = egui::Rect::from_min_size(
                    egui::pos2(ui.min_rect().left() + left_width, ui.min_rect().top()),
                    egui::vec2(splitter_width, available_height),
                );
                let splitter_response = ui.interact(splitter_rect, ui.id().with("splitter"), egui::Sense::click_and_drag());
                let splitter_color = if splitter_response.is_pointer_button_down_on() {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                };
                ui.painter().rect_filled(splitter_rect, 0.0, splitter_color);
                if splitter_response.dragged() {
                    let delta = splitter_response.drag_delta().x;
                    self.right_panel_width = (left_width + delta)
                        .clamp(min_left_width, max_left_width);
                }

                // Колонка с кнопками между разделителем и текстом заметки
                if let Some(idx) = self.selected_note {
                    if idx < self.notes.len() {
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            if ui.add_sized([32.0, 32.0], egui::Button::new("📋")).on_hover_text("Копировать").clicked() {
                                self.copy_note_to_clipboard();
                            }
                            ui.add_space(8.0);
                            if ui.add_sized([32.0, 32.0], egui::Button::new("🗑")).on_hover_text("Удалить").clicked() {
                                self.delete_selected_note();
                            }
                        });
                    } else {
                        ui.add_space(48.0); // чтобы не ломать layout, если заметка не выбрана
                    }
                } else {
                    ui.add_space(48.0);
                }

                // Правая панель (заметка или создание)
                ui.vertical(|ui| {
                    ui.set_width(right_width.max(220.0));
                    if let Some(idx) = self.selected_note {
                        if idx < self.notes.len() {
                            ui.add_space(6.0);
                            // Заголовок заметки с иконкой ✏️ для редактирования
                            ui.horizontal(|ui| {
                                // Кнопка закрепить/открепить
                                let pin_icon = if self.notes[idx].pinned { "📌" } else { "📍" };
                                if ui.add(egui::Label::new(pin_icon).sense(egui::Sense::click())).on_hover_text(if self.notes[idx].pinned { "Открепить" } else { "Закрепить" }).clicked() {
                                    self.notes[idx].pinned = !self.notes[idx].pinned;
                                    let note = &self.notes[idx];
                                    let _ = self.notes_manager.save_note(note);
                                    // После изменения pinned пересортировать список
                                    self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                                }
                                if self.editing_title == Some(idx) {
                                    let mut title = self.notes[idx].title.clone();
                                    let response = ui.add_sized([
                                        ui.available_width() - 36.0, 32.0],
                                        egui::TextEdit::singleline(&mut title)
                                            .hint_text("Заголовок заметки")
                                            .frame(false)
                                    );
                                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || response.lost_focus() && !response.has_focus() {
                                        if !title.trim().is_empty() {
                                            self.notes[idx].title = title.clone();
                                            // Сохраняем заметку с новым заголовком
                                            let note = &self.notes[idx];
                                            let _ = self.notes_manager.save_note(note);
                                        }
                                        self.editing_title = None;
                                    }
                                    if ui.add(egui::Button::new("✔️")).on_hover_text("Сохранить").clicked() {
                                        if !title.trim().is_empty() {
                                            self.notes[idx].title = title.clone();
                                            let note = &self.notes[idx];
                                            let _ = self.notes_manager.save_note(note);
                                        }
                                        self.editing_title = None;
                                    }
                                } else {
                                    ui.label(
                                        egui::RichText::new(&self.notes[idx].title)
                                            .size(20.0)
                                            .strong()
                                    );
                                    if ui.add(egui::Label::new("📝").sense(egui::Sense::click())).on_hover_text("Редактировать заголовок").clicked() {
                                        self.editing_title = Some(idx);
                                    }
                                }
                            });
                            ui.add_space(6.0);
                            // Поле для редактирования содержимого с лёгкой рамкой
                            egui::Frame::none()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::same(6.0))
                                .show(ui, |ui| {
                                    let mut content = self.new_note_content.clone();
                                    let text_edit = egui::TextEdit::multiline(&mut content)
                                        .hint_text("Текст заметки...")
                                        .desired_rows(8)
                                        .frame(false)
                                        .code_editor();
                                    if ui.add_sized([ui.available_width(), 120.0], text_edit).changed() {
                                        self.new_note_content = content;
                                        self.save_note_changes();
                                    }
                                });
                        }
                    } else {
                        // Все элементы формы создания заметки располагаем вертикально, фон убран, только лёгкая рамка
                        ui.vertical(|ui| {
                            egui::Frame::none()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.add_sized([
                                        ui.available_width(), 28.0],
                                        egui::TextEdit::singleline(&mut self.new_note_title)
                                            .hint_text("Заголовок заметки")
                                            .frame(false)
                                    );
                                });
                            ui.add_space(10.0);
                            egui::Frame::none()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .rounding(egui::Rounding::same(6.0))
                                .inner_margin(egui::Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.add_sized([
                                        ui.available_width(), 80.0],
                                        egui::TextEdit::multiline(&mut self.new_note_content)
                                            .hint_text("Текст заметки...")
                                            .desired_rows(4)
                                            .frame(false)
                                            .code_editor()
                                    );
                                });
                            ui.add_space(16.0);
                            if ui.add_sized([120.0, 36.0], egui::Button::new("Создать")).clicked() {
                                self.create_new_note();
                            }
                        });
                    }
                });
            });
        });
    }
} 