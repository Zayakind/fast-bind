use uuid::Uuid;
use crate::notes::{Note, NotesManager, NoteGroup, NoteMetadata};
use crate::ui::theme::ThemeMode;
use crate::validation::ValidationRules;
use crate::performance::{LazyNoteLoader, LoaderStats};
use crate::{log_error, log_info, log_success};

/// Режим загрузки заметок
#[derive(Debug, Clone, PartialEq)]
pub enum LoadMode {
    /// Автоматический выбор режима в зависимости от количества заметок
    Auto,
    /// Загружать все заметки сразу (для совместимости)
    Eager,
    /// Ленивая загрузка (оптимизированная для больших коллекций)
    Lazy,
}

/// Состояние приложения, отделенное от UI
pub struct AppState {
    pub notes: Vec<Note>,
    pub groups: Vec<NoteGroup>,
    pub notes_manager: NotesManager,
    pub persistent_text: String,
    /// Система ленивой загрузки заметок
    pub lazy_loader: Option<LazyNoteLoader>,
    /// Режим загрузки заметок
    pub load_mode: LoadMode,
    /// Пороговое количество заметок для переключения на ленивую загрузку
    pub lazy_threshold: usize,
}

impl AppState {
    pub fn new(notes_manager: NotesManager) -> Self {
        Self::with_load_mode(notes_manager, LoadMode::Auto)
    }

    /// Создает новое состояние с выбранным режимом загрузки
    pub fn with_load_mode(notes_manager: NotesManager, load_mode: LoadMode) -> Self {
        // Определяем режим загрузки
        let lazy_threshold = 100; // Переходим на ленивую загрузку при >100 заметках
        let notes_count = notes_manager.get_notes_count().unwrap_or(0);
        
        let actual_load_mode = match load_mode {
            LoadMode::Auto => {
                if notes_count > lazy_threshold {
                    LoadMode::Lazy
                } else {
                    LoadMode::Eager
                }
            }
            mode => mode,
        };

        let mut state = Self {
            notes: Vec::new(),
            groups: notes_manager.load_groups().unwrap_or_default(),
            notes_manager,
            persistent_text: String::new(),
            lazy_loader: None,
            load_mode: actual_load_mode,
            lazy_threshold,
        };

        // Инициализируем загрузку данных
        state.initialize_data_loading();
        
        state
    }

    /// Инициализирует загрузку данных в зависимости от режима
    fn initialize_data_loading(&mut self) {
        match self.load_mode {
            LoadMode::Auto | LoadMode::Eager => {
                // Обычная загрузка всех заметок
                self.notes = Self::load_and_sort_notes(&self.notes_manager);
                log_info!("init", "notes", "eager", &format!("Загружено {} заметок в режиме Eager", self.notes.len()));
            }
            LoadMode::Lazy => {
                // Инициализируем ленивую загрузку
                let mut lazy_loader = LazyNoteLoader::new(20, 0); // 20 заметок на страницу
                if let Err(e) = lazy_loader.initialize_with_manager(&self.notes_manager) {
                    log_error!("init", "lazy_loader", &e);
                    // Fallback на обычную загрузку
                    self.load_mode = LoadMode::Eager;
                    self.notes = Self::load_and_sort_notes(&self.notes_manager);
                } else {
                    let total_notes = lazy_loader.total_count();
                    self.lazy_loader = Some(lazy_loader);
                    // Загружаем только первую страницу
                    self.load_initial_page();
                    log_info!("init", "notes", "lazy", &format!("Инициализирована ленивая загрузка для {} заметок", total_notes));
                }
            }
        }

        // Загружаем постоянный текст
        self.persistent_text = Self::load_persistent_text(&self.notes_manager).unwrap_or_default();
    }

    /// Загружает первую страницу заметок при ленивой загрузке
    fn load_initial_page(&mut self) {
        if let Some(ref mut lazy_loader) = self.lazy_loader {
            match lazy_loader.load_next_page(&self.notes_manager) {
                Ok(initial_notes) => {
                    self.notes = initial_notes;
                    Self::sort_notes(&mut self.notes);
                    log_info!("load", "page", "0", &format!("Загружена первая страница: {} заметок", self.notes.len()));
                }
                Err(e) => {
                    log_error!("load", "page", "0", &e);
                }
            }
        }
    }

    /// Загружает больше заметок при необходимости (для ленивой загрузки)
    pub fn load_more_if_needed(&mut self, visible_range: (usize, usize)) -> Result<bool, Box<dyn std::error::Error>> {
        if self.load_mode != LoadMode::Lazy {
            return Ok(false);
        }

        if let Some(ref mut lazy_loader) = self.lazy_loader {
            if lazy_loader.should_load_more(visible_range) {
                match lazy_loader.load_next_page(&self.notes_manager) {
                    Ok(new_notes) => {
                        if !new_notes.is_empty() {
                            self.notes.extend(new_notes);
                            Self::sort_notes(&mut self.notes);
                            log_info!("load", "page", &lazy_loader.loaded_pages().to_string(), 
                                     &format!("Дозагружено {} заметок", lazy_loader.loaded_pages() * 20));
                            return Ok(true);
                        }
                    }
                    Err(e) => {
                        log_error!("load", "page", &lazy_loader.loaded_pages().to_string(), &e);
                        return Err(Box::new(e));
                    }
                }
            }
        }

        Ok(false)
    }

    /// Получает статистику загрузки
    pub fn get_loading_stats(&self) -> Option<LoaderStats> {
        self.lazy_loader.as_ref().map(|loader| loader.get_stats())
    }

    /// Переключает режим загрузки
    pub fn switch_load_mode(&mut self, new_mode: LoadMode) -> Result<(), Box<dyn std::error::Error>> {
        if self.load_mode == new_mode {
            return Ok(());
        }

        let old_mode = self.load_mode.clone();
        
        log_info!("switch", "load_mode", &format!("{:?}", new_mode), &format!("Переключение с {:?} на {:?}", old_mode, new_mode));

        self.load_mode = new_mode;

        // Переинициализируем загрузку
        self.initialize_data_loading();

        Ok(())
    }

    /// Получает метаданные заметки без полной загрузки (для ленивой загрузки)
    pub fn get_note_metadata(&self, index: usize) -> Option<&NoteMetadata> {
        if let Some(ref lazy_loader) = self.lazy_loader {
            lazy_loader.get_note_metadata(index)
        } else {
            None
        }
    }

    /// Проверяет, загружена ли заметка в кэш
    pub fn is_note_cached(&self, index: usize) -> bool {
        if let Some(ref lazy_loader) = self.lazy_loader {
            lazy_loader.is_note_cached(index)
        } else {
            index < self.notes.len()
        }
    }

    /// Получает общее количество заметок (включая незагруженные)
    pub fn total_notes_count(&self) -> usize {
        if let Some(ref lazy_loader) = self.lazy_loader {
            lazy_loader.total_count()
        } else {
            self.notes.len()
        }
    }

    /// Загружает и сортирует заметки
    fn load_and_sort_notes(notes_manager: &NotesManager) -> Vec<Note> {
        let mut notes = notes_manager.get_all_notes().unwrap_or_default();
        Self::sort_notes(&mut notes);
        notes
    }
    
    /// Сортирует заметки (DRY - убираем дублирование)
    pub fn sort_notes(notes: &mut Vec<Note>) {
        notes.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.created_at.cmp(&a.created_at)));
    }
    
    /// Создает новую заметку с валидацией
    pub fn create_note(&mut self, title: String, content: String, group_id: Option<Uuid>) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Валидация входных данных
        let validation = ValidationRules::validate_note_creation(&title, &content);
        if !validation.is_valid {
            let error_msg = validation.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
                         log_error!("create", "note", &std::io::Error::new(std::io::ErrorKind::InvalidInput, error_msg.clone()));
            return Err(format!("Ошибки валидации: {}", error_msg).into());
        }
        
        let note_id = Uuid::new_v4();
        let note = Note {
            id: note_id,
            title: title.trim().to_string(),
            content,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            pinned: false,
            group_id,
        };
        
        log_info!("create", "note", &note_id.to_string(), &format!("Создание заметки '{}'", note.title));
        
        match self.notes_manager.save_note(&note) {
            Ok(_) => {
                self.notes.push(note);
                Self::sort_notes(&mut self.notes);
                
                // Обновляем ленивую загрузку если нужно
                if let Some(ref mut lazy_loader) = self.lazy_loader {
                    if let Err(e) = lazy_loader.initialize_with_manager(&self.notes_manager) {
                        log_error!("update", "lazy_loader", &e);
                    }
                }
                
                log_success!("create", "note", &note_id.to_string());
                Ok(note_id)
            }
            Err(e) => {
                log_error!("create", "note", &note_id.to_string(), &e);
                Err(Box::new(e))
            }
        }
    }
    
    /// Удаляет заметку по индексу
    pub fn delete_note(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.notes.len() {
            return Ok(());
        }
        
        let note_id = self.notes[index].id;
        self.notes_manager.delete_note(note_id)?;
        self.notes.remove(index);
        Self::sort_notes(&mut self.notes);
        Ok(())
    }
    
    /// Обновляет заметку
    pub fn update_note(&mut self, index: usize, title: Option<String>, content: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.notes.len() {
            return Ok(());
        }
        
        let note = &mut self.notes[index];
        
        if let Some(new_title) = title {
            if !new_title.trim().is_empty() {
                note.title = new_title.trim().to_string();
            }
        }
        
        if let Some(new_content) = content {
            note.content = new_content;
        }
        
        note.updated_at = chrono::Utc::now();
        self.notes_manager.save_note(note)?;
        Self::sort_notes(&mut self.notes);
        Ok(())
    }
    
    /// Переключает закрепление заметки
    pub fn toggle_pin(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index >= self.notes.len() {
            return Ok(());
        }
        
        self.notes[index].pinned = !self.notes[index].pinned;
        self.notes_manager.save_note(&self.notes[index])?;
        Self::sort_notes(&mut self.notes);
        Ok(())
    }
    

    
    /// Получает содержимое заметки для копирования
    pub fn get_note_content(&self, index: usize) -> Option<String> {
        self.notes.get(index).map(|note| note.content.clone())
    }
    
    /// Добавляет содержимое заметки к постоянному тексту
    pub fn append_note_to_persistent(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(note) = self.notes.get(index) {
            self.persistent_text.push_str(&note.content);
            self.save_persistent_text()?;
        }
        Ok(())
    }
    
    /// Сохраняет постоянный текст
    pub fn save_persistent_text(&self) -> Result<(), Box<dyn std::error::Error>> {
        let persistent_file = self.notes_manager.get_base_dir().join("persistent_text.txt");
        std::fs::write(persistent_file, &self.persistent_text)?;
        Ok(())
    }
    
    /// Загружает постоянный текст
    fn load_persistent_text(notes_manager: &NotesManager) -> Result<String, Box<dyn std::error::Error>> {
        let persistent_file = notes_manager.get_base_dir().join("persistent_text.txt");
        
        if persistent_file.exists() {
            Ok(std::fs::read_to_string(persistent_file)?)
        } else {
            Ok(String::new())
        }
    }
    
    /// Создает группу с валидацией
    pub fn create_group(&mut self, name: String, parent_id: Option<Uuid>, selected_notes: Vec<Uuid>) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Валидация входных данных
        let validation = ValidationRules::validate_group_creation(&name, parent_id, &self.groups);
        if !validation.is_valid {
            let error_msg = validation.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
                         log_error!("create", "group", &std::io::Error::new(std::io::ErrorKind::InvalidInput, error_msg.clone()));
            return Err(format!("Ошибки валидации: {}", error_msg).into());
        }
        
        let (level, parent_id) = if let Some(parent_id) = parent_id {
            let parent_level = self.groups.iter()
                .find(|g| g.id == parent_id)
                .map(|g| g.level)
                .unwrap_or(0);
            (parent_level + 1, Some(parent_id))
        } else {
            (0, None)
        };
        
        let group_id = Uuid::new_v4();
        let group = NoteGroup {
            id: group_id,
            name: name.trim().to_string(),
            collapsed: false,
            parent_id,
            level,
        };
        
        log_info!("create", "group", &group_id.to_string(), &format!("Создание группы '{}'", group.name));
        
        self.groups.push(group);
        
        // Добавляем выбранные заметки в группу
        let mut updated_notes = 0;
        for note in &mut self.notes {
            if selected_notes.contains(&note.id) {
                note.group_id = Some(group_id);
                if let Err(e) = self.notes_manager.save_note(note) {
                    log_error!("update", "note", &note.id.to_string(), &e);
                } else {
                    updated_notes += 1;
                }
            }
        }
        
        match self.notes_manager.save_groups(&self.groups) {
            Ok(_) => {
                log_success!("create", "group", &group_id.to_string());
                if updated_notes > 0 {
                    log_info!("update", "group", &group_id.to_string(), &format!("Добавлено {} заметок в группу", updated_notes));
                }
                Ok(group_id)
            }
            Err(e) => {
                log_error!("create", "group", &group_id.to_string(), &e);
                Err(Box::new(e))
            }
        }
    }
    
    /// Переключает состояние сворачивания группы
    pub fn toggle_group_collapsed(&mut self, group_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.collapsed = !group.collapsed;
            self.notes_manager.save_groups(&self.groups)?;
        }
        Ok(())
    }
    
    /// Удаляет группу и перемещает её содержимое в родительскую группу
    pub fn delete_group(&mut self, group_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
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
            self.notes_manager.save_groups(&self.groups)?;
        }
        Ok(())
    }
    
    /// Обновляет группу с валидацией
    pub fn update_group(&mut self, group_id: Uuid, name: String, parent_id: Option<Uuid>) -> Result<(), Box<dyn std::error::Error>> {
        // Валидация изменения родительской группы
        let parent_validation = ValidationRules::validate_group_parent_change(group_id, parent_id, &self.groups);
        if !parent_validation.is_valid {
            let error_msg = parent_validation.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
                         log_error!("update", "group", &group_id.to_string(), &std::io::Error::new(std::io::ErrorKind::InvalidInput, error_msg.clone()));
            return Err(format!("Ошибки валидации родителя: {}", error_msg).into());
        }
        
        // Валидация названия группы
        let name_validation = ValidationRules::validate_group_creation(&name, parent_id, &self.groups);
        if !name_validation.is_valid {
            let error_msg = name_validation.errors.iter()
                .filter(|e| matches!(e, crate::validation::ValidationError::EmptyGroupName | crate::validation::ValidationError::GroupNameTooLong(_)))
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
                         if !error_msg.is_empty() {
                 log_error!("update", "group", &group_id.to_string(), &std::io::Error::new(std::io::ErrorKind::InvalidInput, error_msg.clone()));
                 return Err(format!("Ошибки валидации названия: {}", error_msg).into());
             }
        }
        
        // Находим новый уровень родителя
        let new_level = if let Some(parent_id) = parent_id {
            self.groups.iter()
                .find(|g| g.id == parent_id)
                .map(|g| g.level + 1)
                .unwrap_or(0)
        } else {
            0
        };
        
        log_info!("update", "group", &group_id.to_string(), &format!("Обновление группы: '{}'", name));
        
        // Обновляем группу
        let (old_name, old_parent_id) = {
            if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
                let old_name = group.name.clone();
                let old_parent_id = group.parent_id;
                
                group.name = name.trim().to_string();
                group.parent_id = parent_id;
                
                // Если изменился родитель, обновляем уровень
                if old_parent_id != parent_id {
                    group.level = new_level;
                }
                
                (old_name, old_parent_id)
            } else {
                let error = std::io::Error::new(std::io::ErrorKind::NotFound, "Группа не найдена");
                log_error!("update", "group", &group_id.to_string(), &error);
                return Err(error.into());
            }
        };
        
        // Обновляем подгруппы после освобождения заимствования
        if old_parent_id != parent_id {
            self.update_subgroup_levels(group_id, new_level);
            log_info!("update", "group", &group_id.to_string(), "Обновлена иерархия подгрупп");
        }
        
        if old_name != name.trim() {
            log_info!("update", "group", &group_id.to_string(), &format!("Название изменено с '{}' на '{}'", old_name, name.trim()));
        }
        
        if false {
            let error = std::io::Error::new(std::io::ErrorKind::NotFound, "Группа не найдена");
            log_error!("update", "group", &group_id.to_string(), &error);
            return Err(error.into());
        }
        
        match self.notes_manager.save_groups(&self.groups) {
            Ok(_) => {
                log_success!("update", "group", &group_id.to_string());
                Ok(())
            }
            Err(e) => {
                log_error!("update", "group", &group_id.to_string(), &e);
                Err(Box::new(e))
            }
        }
    }
    
    /// Обновляет уровни всех подгрупп итеративно
    fn update_subgroup_levels(&mut self, parent_id: Uuid, parent_level: u32) {
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
}

/// UI состояние, отделенное от бизнес-логики
pub struct UiState {
    pub selected_note: Option<usize>,
    pub editing_title: Option<usize>,
    pub editing_content: Option<usize>,
    pub theme_mode: ThemeMode,
    
    // Настройки производительности
    pub preferred_load_mode: LoadMode,
    pub show_performance_stats: bool,
    
    // Формы
    pub new_note_title: String,
    pub new_note_content: String,
    pub new_note_group_id: Option<Uuid>,
    
    // Окна
    pub show_settings: bool,
    pub show_group_creation: bool,
    pub show_group_editor: bool,
    
    // Создание групп
    pub new_group_name: String,
    pub group_creation_selected_notes: Vec<Uuid>,
    pub creating_subgroup_for: Option<Uuid>,
    pub new_note_parent_group_id: Option<Uuid>,
    
    // Редактирование групп
    pub editing_group_id: Option<Uuid>,
    pub editing_group_name: String,
    pub editing_group_parent_id: Option<Uuid>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            selected_note: None,
            editing_title: None,
            editing_content: None,
            theme_mode: ThemeMode::Auto,
            
            // Настройки производительности
            preferred_load_mode: LoadMode::Auto,
            show_performance_stats: false,
            
            new_note_title: String::new(),
            new_note_content: String::new(),
            new_note_group_id: None,
            
            show_settings: false,
            show_group_creation: false,
            show_group_editor: false,
            
            new_group_name: String::new(),
            group_creation_selected_notes: Vec::new(),
            creating_subgroup_for: None,
            new_note_parent_group_id: None,
            
            editing_group_id: None,
            editing_group_name: String::new(),
            editing_group_parent_id: None,
        }
    }
    
    /// Очищает форму создания заметки
    pub fn clear_note_form(&mut self) {
        self.new_note_title.clear();
        self.new_note_content.clear();
        self.new_note_group_id = None;
        self.selected_note = None;
    }
    
    /// Очищает форму создания группы
    pub fn clear_group_form(&mut self) {
        self.show_group_creation = false;
        self.creating_subgroup_for = None;
        self.new_note_parent_group_id = None;
        self.group_creation_selected_notes.clear();
        self.new_group_name.clear();
    }
    

    
    /// Завершает редактирование
    pub fn stop_editing(&mut self) {
        self.editing_content = None;
        self.editing_title = None;
    }
} 