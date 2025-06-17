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
}

 