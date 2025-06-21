use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::error::AppError;

// Структура, представляющая группу заметок
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteGroup {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub collapsed: bool,
    #[serde(default)]
    pub parent_id: Option<Uuid>, // ID родительской группы для вложенности
    #[serde(default)]
    pub level: u32, // Уровень вложенности (0 = корневая группа)
}

// Структура, представляющая заметку
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,                    // Уникальный идентификатор заметки
    pub title: String,               // Заголовок заметки
    pub content: String,             // Содержимое заметки
    pub created_at: DateTime<Utc>,   // Время создания
    pub updated_at: DateTime<Utc>,   // Время последнего обновления
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub group_id: Option<Uuid>,
}

// Структура для управления заметками
pub struct NotesManager {
    notes_dir: PathBuf,  // Директория, где хранятся заметки
}

impl NotesManager {
    // Создаем новый менеджер заметок
    pub fn new(notes_dir: PathBuf) -> Self {
        // Создаем директорию, если она не существует
        if !notes_dir.exists() {
            fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");
        }
        
        Self { notes_dir }
    }

    // Получаем путь к файлу заметки по её ID
    fn get_note_path(&self, id: Uuid) -> PathBuf {
        self.notes_dir.join(format!("{}.json", id))
    }

    // Сохраняем заметку в файл
    pub fn save_note(&self, note: &Note) -> Result<(), AppError> {
        let file_path = self.get_note_path(note.id);
        let content = serde_json::to_string_pretty(note)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    // Получаем список всех заметок
    pub fn get_all_notes(&self) -> Result<Vec<Note>, AppError> {
        let mut notes = Vec::new();
        
        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Note>(&content) {
                        Ok(note) => notes.push(note),
                        Err(e) => eprintln!("Ошибка десериализации заметки {:?}: {}", path, e),
                    },
                    Err(e) => eprintln!("Ошибка чтения файла заметки {:?}: {}", path, e),
                }
            }
        }
        
        // Сортируем заметки по времени создания (новые сверху)
        notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(notes)
    }

    // Удаляем заметку
    pub fn delete_note(&self, id: Uuid) -> Result<(), AppError> {
        let file_path = self.notes_dir.join(format!("{}.json", id));
        if file_path.exists() {
            fs::remove_file(file_path)?;
            Ok(())
        } else {
            Err(AppError::NoteNotFound)
        }
    }

    pub fn save_groups(&self, groups: &Vec<NoteGroup>) -> Result<(), AppError> {
        let file_path = self.notes_dir.join("groups.json");
        let content = serde_json::to_string_pretty(groups)?;
        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn load_groups(&self) -> Result<Vec<NoteGroup>, AppError> {
        let file_path = self.notes_dir.join("groups.json");
        if !file_path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(file_path)?;
        let groups: Vec<NoteGroup> = serde_json::from_str(&content)?;
        Ok(groups)
    }

    /// Получает путь к директории для дополнительных файлов (родительская директория notes)
    pub fn get_base_dir(&self) -> &std::path::Path {
        self.notes_dir.parent().unwrap_or(&self.notes_dir)
    }

    /// Получает список ID всех заметок (быстрая операция для ленивой загрузки)
    pub fn get_note_ids(&self) -> Result<Vec<Uuid>, AppError> {
        let mut note_ids = Vec::new();
        
        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(filename) = path.file_stem() {
                    if let Some(filename_str) = filename.to_str() {
                        if let Ok(uuid) = Uuid::parse_str(filename_str) {
                            note_ids.push(uuid);
                        }
                    }
                }
            }
        }
        
        Ok(note_ids)
    }

    /// Загружает заметку по ID (оптимизировано для ленивой загрузки)
    pub fn load_note_by_id(&self, id: Uuid) -> Result<Option<Note>, AppError> {
        let file_path = self.get_note_path(id);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&file_path)?;
        let note: Note = serde_json::from_str(&content)?;
        Ok(Some(note))
    }

    /// Получает метаданные заметок без загрузки содержимого (для быстрого отображения списков)
    pub fn get_notes_metadata(&self) -> Result<Vec<NoteMetadata>, AppError> {
        let mut metadata = Vec::new();
        
        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Note>(&content) {
                        Ok(note) => {
                            metadata.push(NoteMetadata {
                                id: note.id,
                                title: note.title,
                                created_at: note.created_at,
                                updated_at: note.updated_at,
                                pinned: note.pinned,
                                group_id: note.group_id,
                                content_length: note.content.len(),
                            });
                        },
                        Err(e) => eprintln!("Ошибка десериализации заметки {:?}: {}", path, e),
                    },
                    Err(e) => eprintln!("Ошибка чтения файла заметки {:?}: {}", path, e),
                }
            }
        }
        
        // Сортируем по времени создания (новые сверху)
        metadata.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(metadata)
    }

    /// Пагинированная загрузка заметок (для ленивой загрузки)
    pub fn get_notes_page(&self, page: usize, page_size: usize) -> Result<Vec<Note>, AppError> {
        let all_notes = self.get_all_notes()?;
        let start_idx = page * page_size;
        let end_idx = (start_idx + page_size).min(all_notes.len());
        
        if start_idx >= all_notes.len() {
            return Ok(Vec::new());
        }
        
        Ok(all_notes[start_idx..end_idx].to_vec())
    }

    /// Получает количество заметок без их загрузки
    pub fn get_notes_count(&self) -> Result<usize, AppError> {
        let mut count = 0;
        
        for entry in fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

/// Метаданные заметки для быстрого отображения в списках
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pinned: bool,
    pub group_id: Option<Uuid>,
    pub content_length: usize, // Длина содержимого для оценки размера
}

 