use std::collections::{VecDeque, HashMap};
use uuid::Uuid;
use crate::notes::{Note, NotesManager, NoteMetadata};
use crate::error::AppError;

/// Структура для ленивой загрузки заметок с оптимизацией производительности
pub struct LazyNoteLoader {
    /// Размер страницы (количество заметок на одну загрузку)
    notes_per_page: usize,
    /// Текущая активная страница
    current_page: usize,
    /// Общее количество заметок в системе
    total_notes: usize,
    /// Кэш загруженных заметок (индекс -> заметка)
    loaded_notes: HashMap<usize, Note>,
    /// Очередь загруженных индексов для управления памятью
    loaded_indices: VecDeque<usize>,
    /// Максимальное количество заметок в кэше
    max_cache_size: usize,
    /// Список ID всех заметок (для быстрого доступа)
    note_ids: Vec<Uuid>,
    /// Буфер предзагрузки (количество страниц для предзагрузки)
    prefetch_pages: usize,
    /// Кэш метаданных для быстрого отображения
    metadata_cache: HashMap<Uuid, NoteMetadata>,
}

impl LazyNoteLoader {
    /// Создает новый загрузчик с настройками по умолчанию
    pub fn new(notes_per_page: usize, total_notes: usize) -> Self {
        Self {
            notes_per_page,
            current_page: 0,
            total_notes,
            loaded_notes: HashMap::new(),
            loaded_indices: VecDeque::new(),
            max_cache_size: notes_per_page * 5, // Кэшируем 5 страниц
            note_ids: Vec::new(),
            prefetch_pages: 2, // Предзагружаем 2 страницы вперед
            metadata_cache: HashMap::new(),
        }
    }

    /// Инициализирует загрузчик с менеджером заметок
    pub fn initialize_with_manager(&mut self, notes_manager: &NotesManager) -> Result<(), AppError> {
        // Загружаем метаданные заметок для быстрого отображения
        let metadata = notes_manager.get_notes_metadata()?;
        self.note_ids = metadata.iter().map(|m| m.id).collect();
        self.total_notes = self.note_ids.len();
        
        // Кэшируем метаданные
        self.metadata_cache.clear();
        for meta in metadata {
            self.metadata_cache.insert(meta.id, meta);
        }
        
        self.loaded_notes.clear();
        self.loaded_indices.clear();
        self.current_page = 0;
        
        Ok(())
    }

    /// Инициализирует загрузчик со списком ID заметок
    pub fn initialize(&mut self, note_ids: Vec<Uuid>) {
        self.note_ids = note_ids;
        self.total_notes = self.note_ids.len();
        self.loaded_notes.clear();
        self.loaded_indices.clear();
        self.current_page = 0;
    }

    /// Загружает следующую страницу заметок
    pub fn load_next_page(&mut self, notes_manager: &NotesManager) -> Result<Vec<Note>, AppError> {
        if self.current_page * self.notes_per_page >= self.total_notes {
            return Ok(Vec::new()); // Больше нет данных
        }

        let page_notes = self.load_page(self.current_page, notes_manager)?;
        self.current_page += 1;
        
        // Предзагружаем следующие страницы в фоне
        self.prefetch_next_pages(notes_manager)?;
        
        Ok(page_notes)
    }

    /// Загружает конкретную страницу заметок
    pub fn load_page(&mut self, page: usize, notes_manager: &NotesManager) -> Result<Vec<Note>, AppError> {
        let start_idx = page * self.notes_per_page;
        let end_idx = (start_idx + self.notes_per_page).min(self.total_notes);
        
        let mut page_notes = Vec::new();
        
        for i in start_idx..end_idx {
            if let Some(note) = self.get_note_by_index(i, notes_manager)? {
                page_notes.push(note.clone());
            }
        }
        
        Ok(page_notes)
    }

    /// Получает заметку по индексу с кэшированием
    pub fn get_note_by_index(&mut self, index: usize, notes_manager: &NotesManager) -> Result<Option<Note>, AppError> {
        if index >= self.total_notes {
            return Ok(None);
        }

        // Проверяем кэш
        if let Some(note) = self.loaded_notes.get(&index) {
            return Ok(Some(note.clone()));
        }

        // Загружаем заметку из файла
        let note_id = self.note_ids[index];
        if let Some(note) = notes_manager.load_note_by_id(note_id)? {
            self.cache_note(index, note.clone());
            Ok(Some(note))
        } else {
            Ok(None)
        }
    }

    /// Получает метаданные заметки по индексу (быстрая операция)
    pub fn get_note_metadata(&self, index: usize) -> Option<&NoteMetadata> {
        if index >= self.total_notes {
            return None;
        }
        
        let note_id = self.note_ids[index];
        self.metadata_cache.get(&note_id)
    }

    /// Определяет, нужно ли загружать больше заметок на основе видимого диапазона
    pub fn should_load_more(&self, visible_range: (usize, usize)) -> bool {
        let (visible_start, visible_end) = visible_range;
        let loaded_end = self.current_page * self.notes_per_page;
        
        // Загружаем больше, если пользователь приближается к концу загруженных данных
        visible_end + self.notes_per_page >= loaded_end && loaded_end < self.total_notes
    }

    /// Получает заметки для отображения в заданном диапазоне
    pub fn get_visible_notes(&mut self, start: usize, count: usize, notes_manager: &NotesManager) -> Result<Vec<Option<Note>>, AppError> {
        let mut notes = Vec::with_capacity(count);
        
        for i in start..(start + count).min(self.total_notes) {
            notes.push(self.get_note_by_index(i, notes_manager)?);
        }
        
        // Дополняем None'ами до нужного размера
        while notes.len() < count && start + notes.len() < self.total_notes {
            notes.push(None);
        }
        
        Ok(notes)
    }

    /// Предзагружает следующие страницы в фоне
    fn prefetch_next_pages(&mut self, notes_manager: &NotesManager) -> Result<(), AppError> {
        for i in 1..=self.prefetch_pages {
            let page_to_prefetch = self.current_page + i;
            if page_to_prefetch * self.notes_per_page < self.total_notes {
                let _ = self.load_page(page_to_prefetch, notes_manager); // Игнорируем ошибки предзагрузки
            }
        }
        Ok(())
    }

    /// Добавляет заметку в кэш с управлением размером
    fn cache_note(&mut self, index: usize, note: Note) {
        // Если кэш полон, удаляем самую старую заметку
        if self.loaded_notes.len() >= self.max_cache_size {
            if let Some(old_index) = self.loaded_indices.pop_front() {
                self.loaded_notes.remove(&old_index);
            }
        }

        self.loaded_notes.insert(index, note);
        self.loaded_indices.push_back(index);
    }

    /// Очищает кэш загруженных заметок
    pub fn clear_cache(&mut self) {
        self.loaded_notes.clear();
        self.loaded_indices.clear();
    }

    /// Получает статистику загрузчика
    pub fn get_stats(&self) -> LoaderStats {
        LoaderStats {
            total_notes: self.total_notes,
            loaded_notes: self.loaded_notes.len(),
            current_page: self.current_page,
            cache_hit_ratio: if self.total_notes > 0 {
                self.loaded_notes.len() as f32 / self.total_notes as f32
            } else {
                0.0
            },
        }
    }

    /// Сбрасывает загрузчик в начальное состояние
    pub fn reset(&mut self) {
        self.current_page = 0;
        self.clear_cache();
    }

    /// Получает общее количество заметок
    pub fn total_count(&self) -> usize {
        self.total_notes
    }

    /// Получает количество загруженных страниц
    pub fn loaded_pages(&self) -> usize {
        self.current_page
    }

    /// Проверяет, загружена ли заметка в кэш
    pub fn is_note_cached(&self, index: usize) -> bool {
        self.loaded_notes.contains_key(&index)
    }

    /// Получает процент загруженных заметок
    pub fn get_loaded_percentage(&self) -> f32 {
        if self.total_notes == 0 {
            return 100.0;
        }
        (self.loaded_notes.len() as f32 / self.total_notes as f32) * 100.0
    }
}

/// Статистика производительности загрузчика
#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub total_notes: usize,
    pub loaded_notes: usize,
    pub current_page: usize,
    pub cache_hit_ratio: f32,
}

/// Виртуальный скроллер для эффективного отображения больших списков
pub struct VirtualScroller {
    /// Высота одного элемента в пикселях
    item_height: f32,
    /// Высота видимой области
    viewport_height: f32,
    /// Текущая позиция скролла
    scroll_offset: f32,
    /// Дополнительный буфер элементов для плавной прокрутки
    buffer_size: usize,
}

impl VirtualScroller {
    pub fn new(item_height: f32, viewport_height: f32) -> Self {
        Self {
            item_height,
            viewport_height,
            scroll_offset: 0.0,
            buffer_size: 5,
        }
    }

    /// Вычисляет диапазон видимых элементов
    pub fn get_visible_range(&self, total_items: usize) -> (usize, usize) {
        let first_visible = (self.scroll_offset / self.item_height) as usize;
        let visible_count = (self.viewport_height / self.item_height).ceil() as usize;
        
        let start = first_visible.saturating_sub(self.buffer_size);
        let end = (first_visible + visible_count + self.buffer_size).min(total_items);
        
        (start, end)
    }

    /// Обновляет позицию скролла
    pub fn update_scroll(&mut self, new_offset: f32) {
        self.scroll_offset = new_offset.max(0.0);
    }

    /// Получает общую высоту контента
    pub fn get_content_height(&self, total_items: usize) -> f32 {
        total_items as f32 * self.item_height
    }
}