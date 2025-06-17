use eframe::egui;
use crate::notes::{Note, NotesManager, NoteGroup};
use std::path::PathBuf;
use uuid::Uuid;
use clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Auto,    // Следовать системной теме
    Light,   // Светлая тема
    Dark,    // Тёмная тема
}

pub struct App {
    notes: Vec<Note>,
    groups: Vec<NoteGroup>,
    selected_note: Option<usize>,
    new_note_title: String,
    new_note_content: String,
    notes_manager: NotesManager,
    editing_title: Option<usize>, // индекс редактируемого заголовка
    editing_content: Option<usize>, // индекс редактируемого содержимого
    new_group_name: String, // для создания группы
    show_group_creation: bool,
    group_creation_selected_notes: Vec<Uuid>, // id выбранных заметок
    new_note_group_id: Option<Uuid>, // группа для новой заметки
    creating_subgroup_for: Option<Uuid>, // ID группы, для которой создаём подгруппу
    new_note_parent_group_id: Option<Uuid>, // родительская группа для новой группы
    show_group_editor: bool, // показывать окно редактора групп
    editing_group_id: Option<Uuid>, // ID редактируемой группы
    editing_group_name: String, // временное имя редактируемой группы
    editing_group_parent_id: Option<Uuid>, // временный родитель редактируемой группы
    theme_mode: ThemeMode, // выбранная тема
    show_settings: bool, // показывать окно настроек
    persistent_text: String, // постоянное текстовое поле
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
        
        // Загружаем постоянный текст
        let persistent_text = Self::load_persistent_text(&notes_manager).unwrap_or_default();
        
        Self {
            notes,
            groups,
            selected_note: None,
            new_note_title: String::new(),
            new_note_content: String::new(),
            notes_manager,
            editing_title: None,
            editing_content: None,
            new_group_name: String::new(),
            show_group_creation: false,
            group_creation_selected_notes: Vec::new(),
            new_note_group_id: None,
            creating_subgroup_for: None,
            new_note_parent_group_id: None,
            show_group_editor: false,
            editing_group_id: None,
            editing_group_name: String::new(),
            editing_group_parent_id: None,
            theme_mode: ThemeMode::Auto,
            show_settings: false,
            persistent_text,
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
                group_id: self.new_note_group_id,
            };
            
            if let Ok(()) = self.notes_manager.save_note(&note) {
                self.notes.push(note);
                self.notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
                self.new_note_title.clear();
                self.new_note_content.clear();
                self.new_note_group_id = None; // Сбрасываем выбор группы
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
    
    fn copy_note_to_persistent_text(&mut self) {
        if let Some(idx) = self.selected_note {
            if let Some(note) = self.notes.get(idx) {
                // Добавляем только содержимое заметки в конец существующего текста
                self.persistent_text.push_str(&note.content);
                
                // Сохраняем изменения
                let _ = self.save_persistent_text();
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



    /// Создаёт подгруппу для указанной родительской группы
    fn create_subgroup(&mut self, parent_id: Uuid, name: String) {
        let parent_level = self.groups.iter()
            .find(|g| g.id == parent_id)
            .map(|g| g.level)
            .unwrap_or(0);

        let subgroup = NoteGroup {
            id: Uuid::new_v4(),
            name,
            collapsed: false,
            parent_id: Some(parent_id),
            level: parent_level + 1,
        };

        self.groups.push(subgroup);
        let _ = self.notes_manager.save_groups(&self.groups);
    }

    /// Применяет выбранную тему
    fn apply_theme(&self, ctx: &egui::Context) {
        match self.theme_mode {
            ThemeMode::Auto => {
                // Используем системную тему по умолчанию
                ctx.set_visuals(egui::Visuals::default());
            }
            ThemeMode::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            ThemeMode::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    /// Получает цвет для заголовков в зависимости от текущей темы
    fn get_header_color(&self, ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - используем светлый цвет
            egui::Color32::WHITE
        } else {
            // Светлая тема - используем тёмный цвет для контраста
            egui::Color32::BLACK
        }
    }

    /// Получает цвет для текста заметок в зависимости от текущей темы
    fn get_note_text_color(&self, ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - используем светло-серый
            egui::Color32::from_gray(220)
        } else {
            // Светлая тема - используем тёмно-серый для лучшей читаемости
            egui::Color32::from_gray(60)
        }
    }

    /// Получает цвет фона для боковой панели в зависимости от темы
    fn get_side_panel_bg_color(&self, ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - тёмно-серый фон
            egui::Color32::from_gray(30)
        } else {
            // Светлая тема - светло-серый фон для контраста с белым
            egui::Color32::from_gray(240)
        }
    }

    /// Получает настройки тени для боковой панели в зависимости от темы
    fn get_panel_shadow(&self, ctx: &egui::Context) -> egui::Shadow {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - лёгкая тень
            egui::Shadow {
                offset: [2, 0],
                blur: 4,
                spread: 0,
                color: egui::Color32::from_black_alpha(80),
            }
        } else {
            // Светлая тема - мягкая тень
            egui::Shadow {
                offset: [2, 0],
                blur: 6,
                spread: 0,
                color: egui::Color32::from_black_alpha(30),
            }
        }
    }

    /// Получает цвет фона для центральной панели в зависимости от темы
    fn get_central_panel_bg_color(&self, ctx: &egui::Context) -> egui::Color32 {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - тёмно-серый фон
            egui::Color32::from_gray(35)
        } else {
            // Светлая тема - белый фон
            egui::Color32::WHITE
        }
    }

    /// Получает цвета для кнопки настроек в зависимости от темы
    fn get_settings_button_colors(&self, ctx: &egui::Context) -> (egui::Color32, egui::Color32) {
        if ctx.style().visuals.dark_mode {
            // Тёмная тема - тёмный фон, светлая граница
            (egui::Color32::from_gray(50), egui::Color32::from_gray(80))
        } else {
            // Светлая тема - светлый фон, тёмная граница
            (egui::Color32::from_gray(220), egui::Color32::from_gray(160))
        }
    }

    /// Отображает боковую панель внутри основной панели
    fn show_side_panel_inside(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let panel_bg_color = self.get_side_panel_bg_color(ctx);
        let panel_shadow = self.get_panel_shadow(ctx);
        
        // Вычисляем динамическую ширину панели - 30% от доступной ширины
        let dynamic_panel_width = ui.available_width() * 0.30;
        
        egui::SidePanel::left("notes_panel")
            .frame(egui::Frame {
                fill: panel_bg_color,
                stroke: egui::Stroke::NONE,
                inner_margin: egui::Margin::same(8),
                shadow: panel_shadow,
                ..Default::default()
            })
            .resizable(false)
            .min_width(dynamic_panel_width)
            .max_width(dynamic_panel_width)
            .show_inside(ui, |ui| {
                self.show_notes_list(ui, ctx, dynamic_panel_width);
            });
    }



    /// Отображает список заметок и групп внутри боковой панели
    fn show_notes_list(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, panel_width: f32) {
        let panel_bg_color = self.get_side_panel_bg_color(ctx);
        
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Устанавливаем стили для ScrollArea чтобы убрать чёрную полосу
                ui.style_mut().visuals.extreme_bg_color = panel_bg_color;
                ui.style_mut().visuals.widgets.inactive.bg_fill = panel_bg_color;
                ui.style_mut().visuals.widgets.active.bg_fill = panel_bg_color;
                ui.style_mut().visuals.widgets.hovered.bg_fill = panel_bg_color;
                ui.add_space(10.0);
                
                // Компактная строка с заголовком и системными кнопками
                ui.horizontal(|ui| {
                    // Заголовок с адаптивным цветом
                    let header_color = self.get_header_color(ctx);
                    ui.add(egui::Label::new(
                        egui::RichText::new(format!("Заметки ({})", self.notes.len()))
                            .size(18.0)
                            .strong()
                            .color(header_color)
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Компактная кнопка настроек с адаптивным стилем
                        let (button_bg, button_border) = self.get_settings_button_colors(ctx);
                        let settings_button = egui::Button::new("⚙")
                            .min_size(egui::Vec2::new(24.0, 24.0))
                            .fill(button_bg)
                            .stroke(egui::Stroke::new(1.0, button_border));
                        
                        if ui.add(settings_button)
                            .on_hover_text("Настройки приложения")
                            .clicked() 
                        {
                            self.show_settings = true;
                        }
                        
                        // Кнопка редактора групп
                        let group_editor_button = egui::Button::new("📁")
                            .min_size(egui::Vec2::new(24.0, 24.0))
                            .fill(button_bg)
                            .stroke(egui::Stroke::new(1.0, button_border));
                        
                        if ui.add(group_editor_button)
                            .on_hover_text("Редактор групп")
                            .clicked() 
                        {
                            self.show_group_editor = true;
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Кнопки управления
                if ui.add_sized([panel_width - 12.0, 32.0], egui::Button::new("+ Новая заметка")).clicked() {
                    self.selected_note = None;
                    self.new_note_title.clear();
                    self.new_note_content.clear();
                    self.new_note_group_id = None;
                }
                ui.add_space(6.0);
                if ui.add_sized([panel_width - 12.0, 32.0], egui::Button::new("Создать группу")).clicked() {
                    self.show_group_creation = true;
                    self.new_group_name.clear();
                    self.group_creation_selected_notes.clear();
                }
                ui.add_space(10.0);

                // Отображение групп и заметок
                self.show_groups_and_notes(ui, ctx, panel_width);
            });
    }

    /// Отображает группы с заметками и заметки без группы
    fn show_groups_and_notes(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, panel_width: f32) {
        // --- Показываем корневые группы ---
        let mut toggled_group: Option<Uuid> = None;
        let root_groups: Vec<(Uuid, String, bool)> = self.groups.iter()
            .filter(|g| g.parent_id.is_none())
            .map(|g| (g.id, g.name.clone(), g.collapsed))
            .collect();
        
        for (group_id, group_name, collapsed) in root_groups {
            if let Some(toggle_id) = self.show_group_with_hierarchy(ui, ctx, group_id, &group_name, collapsed, 0, panel_width) {
                toggled_group = Some(toggle_id);
            }
        }
        
        if let Some(group_id) = toggled_group {
            self.toggle_group_collapsed(group_id);
        }

        // --- Заметки без группы ---
        self.show_ungrouped_notes(ui, ctx, panel_width);
    }

    /// Отображает группу и её подгруппы
    fn show_group_with_hierarchy(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, group_id: Uuid, group_name: &str, collapsed: bool, depth: usize, panel_width: f32) -> Option<Uuid> {
        let mut toggled_group: Option<Uuid> = None;
        
        // Отступ для вложенности
        if depth > 0 {
            ui.add_space(depth as f32 * 15.0);
        }
        
        let note_indices: Vec<usize> = self.notes.iter().enumerate()
            .filter(|(_, n)| n.group_id == Some(group_id))
            .map(|(i, _)| i)
            .collect();
        
        let header = egui::CollapsingHeader::new(
            egui::RichText::new(group_name)
                .color(self.get_header_color(ctx))
                .strong()
        )
            .id_salt(group_id)
            .default_open(!collapsed)
            .show(ui, |ui| {
                // Показываем заметки в этой группе
                self.show_notes_in_group(ui, ctx, note_indices, panel_width);
                
                // Показываем подгруппы
                let subgroups: Vec<(Uuid, String, bool)> = self.groups.iter()
                    .filter(|g| g.parent_id == Some(group_id))
                    .map(|g| (g.id, g.name.clone(), g.collapsed))
                    .collect();
                
                for (sub_id, sub_name, sub_collapsed) in subgroups {
                    if let Some(sub_toggle) = self.show_group_with_hierarchy(ui, ctx, sub_id, &sub_name, sub_collapsed, depth + 1, panel_width) {
                        toggled_group = Some(sub_toggle);
                    }
                }
            });
        
        if header.header_response.clicked() {
            return Some(group_id);
        }
        
        toggled_group
    }

    /// Отображает заметки внутри группы
    fn show_notes_in_group(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, note_indices: Vec<usize>, panel_width: f32) {
        let note_color = self.get_note_text_color(ctx);
        
        for idx in note_indices {
            let note = &self.notes[idx];
            let is_selected = self.selected_note == Some(idx);
            let mut title = String::new();
            if note.pinned { 
                title.push_str("📌 "); 
            }
            title.push_str(&note.title);
            
            let label = egui::SelectableLabel::new(is_selected, 
                egui::RichText::new(title).color(note_color)
            );
            
            if ui.add_sized([panel_width - 20.0, 28.0], label).clicked() {
                self.selected_note = Some(idx);
                self.new_note_title = note.title.clone();
                self.new_note_content = note.content.clone();
            }
            ui.add_space(4.0);
        }
    }

    /// Отображает заметки без группы
    fn show_ungrouped_notes(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, panel_width: f32) {
        let no_group_notes: Vec<_> = self.notes.iter().enumerate()
            .filter(|(_, n)| n.group_id.is_none())
            .collect();
        
        if !no_group_notes.is_empty() {
            let header_color = self.get_header_color(ctx);
            let note_color = self.get_note_text_color(ctx);
            
            egui::CollapsingHeader::new(
                egui::RichText::new("Без группы")
                    .color(header_color)
                    .strong()
            )
                .default_open(true)
                .show(ui, |ui| {
                    for (idx, note) in no_group_notes {
                        let is_selected = self.selected_note == Some(idx);
                        let mut title = String::new();
                        if note.pinned { 
                            title.push_str("📌 "); 
                        }
                        title.push_str(&note.title);
                        
                        let label = egui::SelectableLabel::new(is_selected, 
                            egui::RichText::new(title).color(note_color)
                        );
                        
                        if ui.add_sized([panel_width - 20.0, 28.0], label).clicked() {
                            self.selected_note = Some(idx);
                            self.new_note_title = note.title.clone();
                            self.new_note_content = note.content.clone();
                        }
                        ui.add_space(4.0);
                    }
                });
        }
    }

    /// Отображает центральную панель внутри основной панели
    fn show_central_panel_inside(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let central_bg_color = self.get_central_panel_bg_color(ctx);
        
        egui::CentralPanel::default()
        .frame(egui::Frame {
            fill: central_bg_color,
            stroke: egui::Stroke::NONE,
            inner_margin: egui::Margin::same(8),
            ..Default::default()
        })
        .show_inside(ui, |ui| {
            // Контент заметки растягивается автоматически до правого края
            self.show_note_content_or_creation(ui);
        });
    }





    /// Отображает содержимое заметки или форму создания новой заметки
    fn show_note_content_or_creation(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Убираем принудительное ограничение ширины - позволяем растягиваться до края
            
            if let Some(idx) = self.selected_note {
                if idx < self.notes.len() {
                    self.show_note_editor(ui, idx);
                }
            } else {
                self.show_note_creation_form(ui);
            }
            
            // Добавляем постоянное текстовое поле
            ui.add_space(12.0);
            self.show_persistent_text_field(ui);
        });
    }

    /// Отображает постоянное текстовое поле
    fn show_persistent_text_field(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.add_space(8.0);
        
        ui.label(
            egui::RichText::new("Заметки и черновики")
                .size(16.0)
                .strong()
        );
        ui.add_space(6.0);
        
        // Постоянное текстовое поле с поддержкой всех хоткеев
        egui::Frame::new()
            .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
            .corner_radius(6)
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                let response = ui.add_sized(
                    [ui.available_width(), 120.0], // Фиксированная высота
                    egui::TextEdit::multiline(&mut self.persistent_text)
                        .hint_text("Здесь вы можете писать заметки, которые сохраняются при переключении между заметками...")
                        .desired_rows(6)
                        .frame(false)
                        .code_editor() // Поддержка хоткеев
                );
                
                // Автосохранение при изменении текста
                if response.changed() {
                    let _ = self.save_persistent_text();
                }
            });
    }

    /// Отображает редактор выбранной заметки
    fn show_note_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.add_space(6.0);
        
        // Заголовок заметки с возможностью редактирования
        self.show_note_title_editor(ui, idx);
        
        ui.add_space(6.0);
        
        // Поле для редактирования содержимого
        self.show_note_content_editor(ui);
        
        // ComboBox для выбора группы
        self.show_group_selector(ui, idx);
        
        ui.add_space(8.0);
        
        // Кнопки действий для заметки
        self.show_note_action_buttons(ui, idx);
    }

    /// Отображает редактор заголовка заметки
    fn show_note_title_editor(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.horizontal(|ui| {
            // Кнопка закрепить/открепить
            let pin_icon = if self.notes[idx].pinned { "📌" } else { "📍" };
            if ui.add(egui::Label::new(pin_icon).sense(egui::Sense::click()))
                .on_hover_text(if self.notes[idx].pinned { "Открепить" } else { "Закрепить" })
                .clicked() 
            {
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
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || 
                   response.lost_focus() && !response.has_focus() 
                {
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
                if ui.add(egui::Label::new("📝").sense(egui::Sense::click()))
                    .on_hover_text("Редактировать заголовок")
                    .clicked() 
                {
                    self.editing_title = Some(idx);
                }
            }
        });
    }

    /// Отображает редактор содержимого заметки
    fn show_note_content_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(idx) = self.selected_note {
            if idx < self.notes.len() {
                if self.editing_content == Some(idx) {
                    // Режим редактирования - показываем текстовое поле
                    let mut content = self.new_note_content.clone();
                    
                    // Подсчитываем количество строк + 2 дополнительные
                    let line_count = content.lines().count().max(1) + 2;
                    let text_height = (line_count as f32 * 18.0).max(60.0); // минимум 60px
                    let frame_height = text_height + 12.0; // +12px для отступов
                    
                    egui::Frame::new()
                        .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                        .corner_radius(6)
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.set_height(frame_height);
                            
                            let text_edit = egui::TextEdit::multiline(&mut content)
                                .hint_text("Текст заметки...")
                                .desired_rows(line_count)
                                .frame(false);
                            
                            if ui.add_sized([ui.available_width(), text_height], text_edit).changed() {
                                self.new_note_content = content;
                            }
                        });
                } else {
                    // Режим просмотра - показываем только текст
                    let content = &self.notes[idx].content;
                    
                    // Подсчитываем высоту для отображения + 1 дополнительная строка
                    let line_count = content.lines().count().max(1) + 1;
                    let frame_height = (line_count as f32 * 18.0 + 20.0).max(60.0); // +20px для padding
                    
                    egui::Frame::new()
                        .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
                        .corner_radius(6)
                        .inner_margin(egui::Margin::same(6))
                        .show(ui, |ui| {
                            ui.set_height(frame_height);
                            
                            // Создаём прямоугольную область как в режиме редактирования, но для Label
                            let response = ui.allocate_response(
                                egui::Vec2::new(ui.available_width(), frame_height - 12.0),
                                egui::Sense::click_and_drag()
                            );
                            
                            // Отрисовываем текст в левом верхнем углу этого прямоугольника (современный метод)
                            let text_rect = egui::Rect::from_min_size(
                                response.rect.min,
                                response.rect.size()
                            );
                            
                            ui.allocate_new_ui(
                                egui::UiBuilder::new().max_rect(text_rect),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(content)
                                            .wrap()
                                            .selectable(true) // Можно выделять текст
                                    );
                                }
                            );
                        });
                }
            }
        }
    }

    /// Отображает селектор группы для заметки
    fn show_group_selector(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.horizontal(|ui| {
            ui.label("Группа:");
            let mut new_group_id: Option<Option<Uuid>> = None;
            
            egui::ComboBox::from_id_salt("group_select")
                .selected_text(
                    self.groups.iter()
                        .find(|g| Some(g.id) == self.notes[idx].group_id)
                        .map(|g| g.name.as_str())
                        .unwrap_or("Без группы")
                )
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

    /// Отображает кнопки действий для заметки (встроенные в область заметки)
    fn show_note_action_buttons(&mut self, ui: &mut egui::Ui, idx: usize) {
        ui.horizontal(|ui| {
            if self.editing_content == Some(idx) {
                // В режиме редактирования показываем кнопки Save/Cancel
                if ui.add_sized([80.0, 32.0], egui::Button::new("💾 Сохранить")).clicked() {
                    self.save_note_changes();
                    self.editing_content = None;
                }
                ui.add_space(8.0);
                if ui.add_sized([80.0, 32.0], egui::Button::new("❌ Отмена")).clicked() {
                    // Восстанавливаем оригинальное содержимое
                    self.new_note_content = self.notes[idx].content.clone();
                    self.editing_content = None;
                }
            } else {
                // В режиме просмотра показываем кнопки в нужном порядке:
                // 1. Копирование (первая - самая частая операция)
                if ui.add_sized([90.0, 32.0], egui::Button::new("📋 Копировать")).clicked() {
                    self.copy_note_to_clipboard();
                }
                ui.add_space(8.0);
                
                // 2. Копирование в постоянное поле (вторая по важности)
                if ui.add_sized([110.0, 32.0], egui::Button::new("📄 В заметки")).clicked() {
                    self.copy_note_to_persistent_text();
                }
                ui.add_space(8.0);
                
                // 3. Редактирование (третья)
                if ui.add_sized([110.0, 32.0], egui::Button::new("📝 Редактировать")).clicked() {
                    self.editing_content = Some(idx);
                    self.new_note_content = self.notes[idx].content.clone();
                }
                ui.add_space(8.0);
                
                // 4. Удаление (последняя - самая опасная операция)
                if ui.add_sized([80.0, 32.0], egui::Button::new("🗑 Удалить")).clicked() {
                    self.delete_selected_note();
                }
            }
        });
    }

    /// Отображает форму создания новой заметки
    fn show_note_creation_form(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Поле заголовка
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
            
            // Поле содержимого
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
            
            ui.add_space(10.0);
            
            // Выбор группы для новой заметки
            self.show_new_note_group_selector(ui);
            
            ui.add_space(16.0);
            
            // Кнопка создания
            if ui.add_sized([120.0, 36.0], egui::Button::new("Создать")).clicked() {
                self.create_new_note();
            }
        });
    }

    /// Отображает селектор группы для новой заметки
    fn show_new_note_group_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Группа:");
            let mut new_group_id: Option<Option<Uuid>> = None;
            
            egui::ComboBox::from_id_salt("new_note_group_select")
                .selected_text(
                    self.groups.iter()
                        .find(|g| Some(g.id) == self.new_note_group_id)
                        .map(|g| g.name.as_str())
                        .unwrap_or("Без группы")
                )
                .show_ui(ui, |ui| {
                    if ui.selectable_label(self.new_note_group_id.is_none(), "Без группы").clicked() {
                        new_group_id = Some(None);
                    }
                    for group in &self.groups {
                        if ui.selectable_label(self.new_note_group_id == Some(group.id), &group.name).clicked() {
                            new_group_id = Some(Some(group.id));
                        }
                    }
                });
            
            if let Some(gid) = new_group_id {
                self.new_note_group_id = gid;
            }
        });
    }

    /// Отображает окно настроек
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if self.show_settings {
            egui::Window::new("Настройки")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Тема приложения");
                    ui.add_space(10.0);
                    
                    let mut theme_changed = false;
                    
                    ui.horizontal(|ui| {
                        if ui.radio_value(&mut self.theme_mode, ThemeMode::Auto, "Автоматически").clicked() {
                            theme_changed = true;
                        }
                        ui.label("(следовать системной теме)");
                    });
                    
                    if ui.radio_value(&mut self.theme_mode, ThemeMode::Light, "Светлая тема").clicked() {
                        theme_changed = true;
                    }
                    
                    if ui.radio_value(&mut self.theme_mode, ThemeMode::Dark, "Тёмная тема").clicked() {
                        theme_changed = true;
                    }
                    
                    if theme_changed {
                        self.apply_theme(ctx);
                    }
                    
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    if ui.button("Закрыть").clicked() {
                        self.show_settings = false;
                    }
                });
        }
    }

    /// Отображает окно создания группы
    fn show_group_creation_window(&mut self, ctx: &egui::Context) {
        if self.show_group_creation {
            let window_title = if self.creating_subgroup_for.is_some() {
                "Создать подгруппу"
            } else {
                "Создать группу"
            };

            egui::Window::new(window_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // Показываем информацию о родительской группе, если создаём подгруппу
                    if let Some(parent_id) = self.creating_subgroup_for {
                        if let Some(parent_group) = self.groups.iter().find(|g| g.id == parent_id) {
                            ui.label(format!("Родительская группа: {}", parent_group.name));
                            ui.separator();
                        }
                    }

                    ui.label("Имя группы:");
                    ui.text_edit_singleline(&mut self.new_group_name);
                    ui.add_space(10.0);
                    
                    // Выбор родительской группы (только если не создаём подгруппу)
                    if self.creating_subgroup_for.is_none() {
                        ui.label("Родительская группа:");
                        let mut new_parent_id: Option<Option<Uuid>> = None;
                        
                        egui::ComboBox::from_id_salt("parent_group_select")
                            .selected_text(
                                self.groups.iter()
                                    .find(|g| Some(g.id) == self.new_note_parent_group_id)
                                    .map(|g| g.name.as_str())
                                    .unwrap_or("Корневая группа")
                            )
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(self.new_note_parent_group_id.is_none(), "Корневая группа").clicked() {
                                    new_parent_id = Some(None);
                                }
                                for group in &self.groups {
                                    if ui.selectable_label(self.new_note_parent_group_id == Some(group.id), &group.name).clicked() {
                                        new_parent_id = Some(Some(group.id));
                                    }
                                }
                            });
                        
                        if let Some(pid) = new_parent_id {
                            self.new_note_parent_group_id = pid;
                        }
                    }
                    
                    ui.separator();
                    
                    ui.label("Добавить заметки в группу:");
                    // Отображаем только заметки без группы
                    let ungrouped_notes: Vec<&Note> = self.notes.iter()
                        .filter(|note| note.group_id.is_none())
                        .collect();
                    
                    if ungrouped_notes.is_empty() {
                        ui.label("Нет заметок без группы");
                    } else {
                        for note in ungrouped_notes {
                            let mut checked = self.group_creation_selected_notes.contains(&note.id);
                            if ui.checkbox(&mut checked, &note.title).changed() {
                                if checked {
                                    self.group_creation_selected_notes.push(note.id);
                                } else {
                                    self.group_creation_selected_notes.retain(|id| id != &note.id);
                                }
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    if ui.button("Создать").clicked() && !self.new_group_name.trim().is_empty() {
                        if let Some(parent_id) = self.creating_subgroup_for {
                            // Создаём подгруппу
                            self.create_subgroup(parent_id, self.new_group_name.trim().to_string());
                        } else {
                            // Создаём группу с выбранным родителем
                            let (parent_id, level) = if let Some(parent_id) = self.new_note_parent_group_id {
                                let parent_level = self.groups.iter()
                                    .find(|g| g.id == parent_id)
                                    .map(|g| g.level)
                                    .unwrap_or(0);
                                (Some(parent_id), parent_level + 1)
                            } else {
                                (None, 0)
                            };
                            
                            let group = NoteGroup {
                                id: Uuid::new_v4(),
                                name: self.new_group_name.trim().to_string(),
                                collapsed: false,
                                parent_id,
                                level,
                            };
                            let group_id = group.id;
                            self.groups.push(group);
                            
                            // Добавляем выбранные заметки в группу
                            for note in &mut self.notes {
                                if self.group_creation_selected_notes.contains(&note.id) {
                                    note.group_id = Some(group_id);
                                    let _ = self.notes_manager.save_note(note);
                                }
                            }
                            
                            let _ = self.notes_manager.save_groups(&self.groups);
                        }
                        
                        // Сбрасываем состояние
                        self.show_group_creation = false;
                        self.creating_subgroup_for = None;
                        self.new_note_parent_group_id = None;
                        self.group_creation_selected_notes.clear();
                        self.new_group_name.clear();
                    }
                    
                    if ui.button("Отмена").clicked() {
                        self.show_group_creation = false;
                        self.creating_subgroup_for = None;
                        self.new_note_parent_group_id = None;
                        self.group_creation_selected_notes.clear();
                        self.new_group_name.clear();
                    }
                });
        }
    }

    /// Отображает окно редактора групп
    fn show_group_editor_window(&mut self, ctx: &egui::Context) {
        if self.show_group_editor {
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
                            // Отображаем список всех групп для редактирования
                            let mut groups_to_delete = Vec::new();
                            let mut groups_data: Vec<(Uuid, String, Option<Uuid>, u32)> = self.groups.iter()
                                .map(|g| (g.id, g.name.clone(), g.parent_id, g.level))
                                .collect();
                            
                            groups_data.sort_by(|a, b| a.3.cmp(&b.3).then(a.1.cmp(&b.1))); // Сортируем по уровню, потом по имени
                            
                            for (group_id, group_name, parent_id, level) in groups_data {
                                ui.horizontal(|ui| {
                                    // Отступ для уровня вложенности
                                    ui.add_space(level as f32 * 20.0);
                                    
                                    if self.editing_group_id == Some(group_id) {
                                        // Режим редактирования
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Имя:");
                                                ui.text_edit_singleline(&mut self.editing_group_name);
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                ui.label("Родитель:");
                                                let mut new_parent_id: Option<Option<Uuid>> = None;
                                                
                                                egui::ComboBox::from_id_salt(format!("edit_parent_{}", group_id))
                                                    .selected_text(
                                                        if let Some(pid) = self.editing_group_parent_id {
                                                            self.groups.iter()
                                                                .find(|g| g.id == pid)
                                                                .map(|g| g.name.as_str())
                                                                .unwrap_or("Неизвестная группа")
                                                        } else {
                                                            "Корневая группа"
                                                        }
                                                    )
                                                    .show_ui(ui, |ui| {
                                                        if ui.selectable_label(self.editing_group_parent_id.is_none(), "Корневая группа").clicked() {
                                                            new_parent_id = Some(None);
                                                        }
                                                        for group in &self.groups {
                                                            if group.id != group_id { // Нельзя выбрать себя как родителя
                                                                if ui.selectable_label(self.editing_group_parent_id == Some(group.id), &group.name).clicked() {
                                                                    new_parent_id = Some(Some(group.id));
                                                                }
                                                            }
                                                        }
                                                    });
                                                
                                                if let Some(pid) = new_parent_id {
                                                    self.editing_group_parent_id = pid;
                                                }
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                if ui.small_button("✔️").on_hover_text("Сохранить").clicked() {
                                                    self.save_group_changes(group_id);
                                                }
                                                if ui.small_button("❌").on_hover_text("Отмена").clicked() {
                                                    self.cancel_group_editing();
                                                }
                                            });
                                        });
                                    } else {
                                        // Режим просмотра
                                        // Имя группы
                                        ui.label(&group_name);
                                        
                                        // Показываем родительскую группу
                                        if let Some(parent_id) = parent_id {
                                            if let Some(parent) = self.groups.iter().find(|g| g.id == parent_id) {
                                                ui.label(format!("(в: {})", parent.name));
                                            }
                                        } else {
                                            ui.label("(корневая)");
                                        }
                                        
                                        // Количество заметок в группе
                                        let notes_count = self.notes.iter().filter(|n| n.group_id == Some(group_id)).count();
                                        ui.label(format!("({} заметок)", notes_count));
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            // Кнопка удаления
                                            if ui.small_button("🗑").on_hover_text("Удалить группу").clicked() {
                                                groups_to_delete.push(group_id);
                                            }
                                            
                                            // Кнопка редактирования
                                            if ui.small_button("✏️").on_hover_text("Редактировать группу").clicked() {
                                                self.start_group_editing(group_id, &group_name, parent_id);
                                            }
                                        });
                                    }
                                });
                                ui.separator();
                            }
                            
                            // Удаляем отмеченные группы
                            for group_id in groups_to_delete {
                                self.delete_group(group_id);
                            }
                        });
                    
                    ui.separator();
                    
                    if ui.button("Закрыть").clicked() {
                        self.show_group_editor = false;
                    }
                });
        }
    }

    /// Удаляет группу и перемещает её содержимое в родительскую группу
    fn delete_group(&mut self, group_id: Uuid) {
        if let Some(group_index) = self.groups.iter().position(|g| g.id == group_id) {
            let group = self.groups[group_index].clone();
            
            // Перемещаем все заметки из удаляемой группы в родительскую
            for note in &mut self.notes {
                if note.group_id == Some(group_id) {
                    note.group_id = group.parent_id;
                    let _ = self.notes_manager.save_note(note);
                }
            }
            
            // Перемещаем все подгруппы в родительскую группу
            let parent_level = if let Some(parent_id) = group.parent_id {
                self.groups.iter()
                    .find(|g| g.id == parent_id)
                    .map(|g| g.level)
                    .unwrap_or(0)
            } else {
                0
            };
            
            for subgroup in &mut self.groups {
                if subgroup.parent_id == Some(group_id) {
                    subgroup.parent_id = group.parent_id;
                    subgroup.level = if group.parent_id.is_some() {
                        parent_level + 1
                    } else {
                        0
                    };
                }
            }
            
            // Удаляем группу
            self.groups.remove(group_index);
            let _ = self.notes_manager.save_groups(&self.groups);
        }
    }

    /// Начинает редактирование группы
    fn start_group_editing(&mut self, group_id: Uuid, group_name: &str, parent_id: Option<Uuid>) {
        self.editing_group_id = Some(group_id);
        self.editing_group_name = group_name.to_string();
        self.editing_group_parent_id = parent_id;
    }

    /// Отменяет редактирование группы
    fn cancel_group_editing(&mut self) {
        self.editing_group_id = None;
        self.editing_group_name.clear();
        self.editing_group_parent_id = None;
    }

    /// Сохраняет изменения группы
    fn save_group_changes(&mut self, group_id: Uuid) {
        // Проверяем, что имя не пустое
        if !self.editing_group_name.trim().is_empty() {
            // Сначала находим новый уровень родителя (без mutable borrow)
            let new_level = if let Some(parent_id) = self.editing_group_parent_id {
                self.groups.iter()
                    .find(|g| g.id == parent_id)
                    .map(|g| g.level + 1)
                    .unwrap_or(0)
            } else {
                0
            };
            
            // Теперь обновляем группу
            if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
                group.name = self.editing_group_name.trim().to_string();
                let old_parent_id = group.parent_id;
                group.parent_id = self.editing_group_parent_id;
                
                // Если изменился родитель, обновляем уровень
                if old_parent_id != self.editing_group_parent_id {
                    group.level = new_level;
                }
            }
            
            // Пересчитываем уровни подгрупп если изменился родитель
            let old_parent_id = self.groups.iter()
                .find(|g| g.id == group_id)
                .and_then(|g| if g.parent_id != self.editing_group_parent_id { 
                    Some(g.parent_id) 
                } else { 
                    None 
                });
                
            if old_parent_id.is_some() {
                self.update_subgroup_levels(group_id, new_level);
            }
            
            let _ = self.notes_manager.save_groups(&self.groups);
        }
        
        self.cancel_group_editing();
    }

    /// Обновляет уровни всех подгрупп рекурсивно
    fn update_subgroup_levels(&mut self, parent_id: Uuid, parent_level: u32) {
        // Используем итеративный подход вместо рекурсии для избежания borrow conflicts
        let mut to_update = vec![(parent_id, parent_level)];
        
        while let Some((current_parent_id, current_parent_level)) = to_update.pop() {
            let children: Vec<Uuid> = self.groups.iter()
                .filter(|g| g.parent_id == Some(current_parent_id))
                .map(|g| g.id)
                .collect();
            
            for child_id in children {
                if let Some(child) = self.groups.iter_mut().find(|g| g.id == child_id) {
                    child.level = current_parent_level + 1;
                    to_update.push((child_id, child.level));
                }
            }
        }
    }

    /// Загружает постоянный текст из файла
    fn load_persistent_text(notes_manager: &NotesManager) -> Result<String, crate::error::AppError> {
        let persistent_file = notes_manager.get_base_dir().join("persistent_text.txt");
        
        if persistent_file.exists() {
            std::fs::read_to_string(persistent_file)
                .map_err(|e| crate::error::AppError::Io(e))
        } else {
            Ok(String::new())
        }
    }

    /// Сохраняет постоянный текст в файл
    fn save_persistent_text(&self) -> Result<(), crate::error::AppError> {
        let persistent_file = self.notes_manager.get_base_dir().join("persistent_text.txt");
        
        std::fs::write(persistent_file, &self.persistent_text)
            .map_err(|e| crate::error::AppError::Io(e))
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Применяем выбранную тему
        self.apply_theme(ctx);
        
        // Устанавливаем фон основного окна чтобы убрать чёрную полосу
        let central_bg_color = self.get_central_panel_bg_color(ctx);
        
        // Устанавливаем фон контекста
        ctx.style_mut(|style| {
            style.visuals.panel_fill = central_bg_color;
            style.visuals.window_fill = central_bg_color;
            style.visuals.extreme_bg_color = central_bg_color;
        });
        
        // Создаём полноэкранную панель с правильным фоном
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: central_bg_color,
                stroke: egui::Stroke::NONE,
                inner_margin: egui::Margin::ZERO,
                ..Default::default()
            })
            .show(ctx, |ui| {
                // Внутри этой панели показываем только основные панели
                self.show_side_panel_inside(ui, ctx);
                self.show_central_panel_inside(ui, ctx);
            });
        
        // Показываем окна поверх всего - в основном контексте
        self.show_settings_window(ctx);
        self.show_group_creation_window(ctx);
        self.show_group_editor_window(ctx);
    }
} 