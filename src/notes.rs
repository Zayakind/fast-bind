use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use clipboard::{ClipboardContext, ClipboardProvider};
use crate::error::AppError;

// Структура, представляющая группу заметок
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteGroup {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub collapsed: bool,
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

    // Загружаем заметку из файла
    pub fn load_note(&self, id: Uuid) -> Result<Note, AppError> {
        let file_path = self.get_note_path(id);
        let content = fs::read_to_string(file_path)?;
        let note: Note = serde_json::from_str(&content)?;
        Ok(note)
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
            Err(AppError::Note("Заметка не найдена".into()))
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
}

// Функции для работы с заметками (публичный API)

// Создание новой заметки
pub fn create_note(title: String, content: String) -> Result<Note, AppError> {
    let note = Note {
        id: Uuid::new_v4(),
        title,
        content,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        pinned: false,
        group_id: None,
    };
    Ok(note)
}

// Обновление существующей заметки
pub fn update_note(id: Uuid, title: String, content: String) -> Result<Note, AppError> {
    // TODO: Реализовать обновление заметки
    unimplemented!()
}

// Удаление заметки
pub fn delete_note(id: Uuid) -> Result<(), AppError> {
    // TODO: Реализовать удаление заметки
    unimplemented!()
}

// Получение списка всех заметок
pub fn get_notes() -> Result<Vec<Note>, AppError> {
    // TODO: Реализовать получение списка заметок
    unimplemented!()
}

// Копирование содержимого заметки в буфер обмена
pub fn copy_to_clipboard(content: String) -> Result<(), AppError> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| AppError::Clipboard(e.to_string()))?;
    
    ctx.set_contents(content)
        .map_err(|e| AppError::Clipboard(e.to_string()))?;
    
    Ok(())
} 