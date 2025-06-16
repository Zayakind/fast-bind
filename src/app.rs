use eframe::egui;
use crate::notes::{Note, NotesManager, NoteGroup};
use std::path::PathBuf;
use uuid::Uuid;
use clipboard::{ClipboardContext, ClipboardProvider};

pub struct App {
    notes: Vec<Note>,
    groups: Vec<NoteGroup>,
    selected_note: Option<usize>,
    new_note_title: String,
    new_note_content: String,
    notes_manager: NotesManager,
    right_panel_width: f32, // ширина правой панели
    editing_title: Option<usize>, // индекс редактируемого заголовка
    new_group_name: String, // для создания группы
    show_group_creation: bool,
    group_creation_selected_notes: Vec<Uuid>, // id выбранных заметок
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Создаем директорию для заметок в домашней директории пользователя
        let notes_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".notes")
            .join("notes");
        
        let notes_manager = NotesManager::new(notes_dir);
        
        // Загружаем существующие заметки
        let notes = notes_manager.get_all_notes().unwrap_or_default();
        let mut notes = notes;
        notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
        
        let groups = notes_manager.load_groups().unwrap_or_default();
        
        let default_window_width = 1024.0;
        let left_panel_width = (default_window_width * 0.20) as f32;
        Self {
            notes,
            groups,
            selected_note: None,
            new_note_title: String::new(),
            new_note_content: String::new(),
            notes_manager,
            right_panel_width: left_panel_width,
            editing_title: None,
            new_group_name: String::new(),
            show_group_creation: false,
            group_creation_selected_notes: Vec::new(),
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
                group_id: None,
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
                    group_id: old_note.group_id,
                };
                
                if let Ok(()) = self.notes_manager.save_note(&updated_note) {
                    self.notes[idx] = updated_note;
                    self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                }
            }
        }
    }
    
    fn set_note_group(&mut self, note_idx: usize, group_id: Option<Uuid>) {
        if note_idx < self.notes.len() {
            self.notes[note_idx].group_id = group_id;
            let _ = self.notes_manager.save_note(&self.notes[note_idx]);
        }
    }
    
    fn toggle_group_collapsed(&mut self, group_id: Uuid) {
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.collapsed = !group.collapsed;
            let _ = self.notes_manager.save_groups(&self.groups);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("notes_panel")
            .frame(egui::Frame {
                stroke: egui::Stroke::new(0.0, egui::Color32::TRANSPARENT),
                ..egui::Frame::side_top_panel(&egui::Style::default())
            })
            .resizable(false)
            .min_width(self.right_panel_width)
            .max_width(self.right_panel_width)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        ui.add_space(10.0);
                        ui.heading(format!("Заметки ({})", self.notes.len()));
                        ui.add_space(10.0);
                        if ui.add_sized([self.right_panel_width - 12.0, 32.0], egui::Button::new("+ Новая заметка")).clicked() {
                            self.selected_note = None;
                            self.new_note_title.clear();
                            self.new_note_content.clear();
                        }
                        ui.add_space(6.0);
                        if ui.add_sized([self.right_panel_width - 12.0, 32.0], egui::Button::new("Создать группу")).clicked() {
                            self.show_group_creation = true;
                            self.new_group_name.clear();
                            self.group_creation_selected_notes.clear();
                        }
                        ui.add_space(10.0);

                        // --- Группы ---
                        let mut toggled_group: Option<Uuid> = None;
                        for group in &mut self.groups {
                            let group_id = group.id;
                            let group_name = &group.name;
                            let collapsed = group.collapsed;
                            let note_indices: Vec<usize> = self.notes.iter().enumerate()
                                .filter(|(_, n)| n.group_id == Some(group_id))
                                .map(|(i, _)| i)
                                .collect();
                            let header = egui::CollapsingHeader::new(group_name)
                                .id_salt(group_id)
                                .default_open(!collapsed)
                                .show(ui, |ui| {
                                    for idx in note_indices {
                                        let note = &self.notes[idx];
                                        let is_selected = self.selected_note == Some(idx);
                                        let mut title = String::new();
                                        if note.pinned { title.push_str("📌 "); }
                                        title.push_str(&note.title);
                                        if ui.add_sized([self.right_panel_width - 20.0, 28.0], egui::SelectableLabel::new(is_selected, title)).clicked() {
                                            self.selected_note = Some(idx);
                                            self.new_note_title = note.title.clone();
                                            self.new_note_content = note.content.clone();
                                        }
                                        ui.add_space(4.0);
                                    }
                                });
                            if header.header_response.clicked() {
                                toggled_group = Some(group_id);
                            }
                        }
                        if let Some(group_id) = toggled_group {
                            self.toggle_group_collapsed(group_id);
                        }

                        // --- Заметки без группы ---
                        let no_group_notes: Vec<_> = self.notes.iter().enumerate().filter(|(_, n)| n.group_id.is_none()).collect();
                        if !no_group_notes.is_empty() {
                            egui::CollapsingHeader::new("Без группы")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for (idx, note) in no_group_notes {
                                        let is_selected = self.selected_note == Some(idx);
                                        let mut title = String::new();
                                        if note.pinned { title.push_str("📌 "); }
                                        title.push_str(&note.title);
                                        if ui.add_sized([self.right_panel_width - 20.0, 28.0], egui::SelectableLabel::new(is_selected, title)).clicked() {
                                            self.selected_note = Some(idx);
                                            self.new_note_title = note.title.clone();
                                            self.new_note_content = note.content.clone();
                                        }
                                        ui.add_space(4.0);
                                    }
                                });
                        }
                    });
            });

        egui::CentralPanel::default()
        .show(ctx, |ui| {
            let splitter_width = 6.0;
            let available_width = ui.available_width();
            let left_width = self.right_panel_width;
            let right_width = available_width - left_width - splitter_width - 48.0; // 48px под колонку кнопок

            ui.horizontal(|ui| {
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
                            egui::Frame::new()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .corner_radius(6)
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    let mut content = self.new_note_content.clone();
                                    let text_edit = egui::TextEdit::multiline(&mut content)
                                        .hint_text("Текст заметки...")
                                        .desired_rows(8)
                                        .frame(false);
                                    if ui.add_sized([ui.available_width(), 120.0], text_edit).changed() {
                                        self.new_note_content = content;
                                        self.save_note_changes();
                                    }
                                });

                            // ComboBox для выбора группы
                            ui.horizontal(|ui| {
                                ui.label("Группа:");
                                let mut new_group_id: Option<Option<Uuid>> = None;
                                egui::ComboBox::from_id_salt("group_select")
                                    .selected_text(self.groups.iter().find(|g| Some(g.id) == self.notes[idx].group_id).map(|g| g.name.as_str()).unwrap_or("Без группы"))
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(self.notes[idx].group_id.is_none(), "Без группы").clicked() {
                                            new_group_id = Some(None);
                                        }
                                        for group in &self.groups {
                                            if ui.selectable_label(self.notes[idx].group_id == Some(group.id), &group.name).clicked() {
                                                new_group_id = Some(Some(group.id));
                                            }
                                        }
                                    });
                                if let Some(gid) = new_group_id {
                                    self.set_note_group(idx, gid);
                                }
                            });
                        }
                    } else {
                        // Все элементы формы создания заметки располагаем вертикально, фон убран, только лёгкая рамка
                        ui.vertical(|ui| {
                            egui::Frame::new()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .corner_radius(6)
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    ui.add_sized([
                                        ui.available_width(), 28.0],
                                        egui::TextEdit::singleline(&mut self.new_note_title)
                                            .hint_text("Заголовок заметки")
                                            .frame(false)
                                    );
                                });
                            ui.add_space(10.0);
                            egui::Frame::new()
                                .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                                .corner_radius(6)
                                .inner_margin(egui::Margin::same(6))
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

        if self.show_group_creation {
            egui::Window::new("Создать группу")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Имя группы:");
                    ui.text_edit_singleline(&mut self.new_group_name);
                    ui.separator();
                    ui.label("Добавить заметки в группу:");
                    for note in &self.notes {
                        let mut checked = self.group_creation_selected_notes.contains(&note.id);
                        if ui.checkbox(&mut checked, &note.title).changed() {
                            if checked {
                                self.group_creation_selected_notes.push(note.id);
                            } else {
                                self.group_creation_selected_notes.retain(|id| id != &note.id);
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("Создать").clicked() && !self.new_group_name.trim().is_empty() {
                        let group = NoteGroup {
                            id: Uuid::new_v4(),
                            name: self.new_group_name.trim().to_string(),
                            collapsed: false,
                        };
                        let group_id = group.id;
                        self.groups.push(group);
                        for note in &mut self.notes {
                            if self.group_creation_selected_notes.contains(&note.id) {
                                note.group_id = Some(group_id);
                                let _ = self.notes_manager.save_note(note);
                            }
                        }
                        let _ = self.notes_manager.save_groups(&self.groups);
                        self.show_group_creation = false;
                    }
                    if ui.button("Отмена").clicked() {
                        self.show_group_creation = false;
                    }
                });
        }
    }
} 