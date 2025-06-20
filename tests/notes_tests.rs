use fast_bind::notes::{Note, NotesManager};
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

// Вспомогательная функция для создания тестовой заметки
fn create_test_note() -> Note {
    Note {
        id: Uuid::new_v4(),
        title: "Test Note".to_string(),
        content: "Test Content".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

// Тест создания и сохранения заметки
#[test]
fn test_create_and_save_note() {
    // Создаем временную директорию для тестов
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    
    // Создаем менеджер заметок
    let manager = NotesManager::new(notes_dir.clone());
    
    // Создаем тестовую заметку
    let note = create_test_note();
    
    // Сохраняем заметку
    assert!(manager.save_note(&note).is_ok());
    
    // Проверяем, что файл создан
    let note_path = notes_dir.join(format!("{}.json", note.id));
    assert!(note_path.exists());
    
    // Проверяем содержимое файла
    let content = fs::read_to_string(note_path).unwrap();
    assert!(content.contains(&note.title));
    assert!(content.contains(&note.content));
}

// Тест загрузки заметки
#[test]
fn test_load_note() {
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    let manager = NotesManager::new(notes_dir);
    
    // Создаем и сохраняем заметку
    let original_note = create_test_note();
    manager.save_note(&original_note).unwrap();
    
    // Загружаем заметку
    let loaded_note = manager.load_note(original_note.id).unwrap();
    
    // Проверяем, что данные совпадают
    assert_eq!(loaded_note.id, original_note.id);
    assert_eq!(loaded_note.title, original_note.title);
    assert_eq!(loaded_note.content, original_note.content);
}

// Тест получения всех заметок
#[test]
fn test_get_all_notes() {
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    let manager = NotesManager::new(notes_dir);
    
    // Создаем несколько заметок
    let note1 = create_test_note();
    let note2 = create_test_note();
    let note3 = create_test_note();
    
    // Сохраняем заметки
    manager.save_note(&note1).unwrap();
    manager.save_note(&note2).unwrap();
    manager.save_note(&note3).unwrap();
    
    // Получаем все заметки
    let notes = manager.get_all_notes().unwrap();
    
    // Проверяем количество заметок
    assert_eq!(notes.len(), 3);
    
    // Проверяем, что все заметки присутствуют
    let note_ids: Vec<Uuid> = notes.iter().map(|n| n.id).collect();
    assert!(note_ids.contains(&note1.id));
    assert!(note_ids.contains(&note2.id));
    assert!(note_ids.contains(&note3.id));
}

// Тест удаления заметки
#[test]
fn test_delete_note() {
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    let manager = NotesManager::new(notes_dir.clone());
    
    // Создаем и сохраняем заметку
    let note = create_test_note();
    manager.save_note(&note).unwrap();
    
    // Проверяем, что файл создан
    let note_path = notes_dir.join(format!("{}.json", note.id));
    assert!(note_path.exists());
    
    // Удаляем заметку
    assert!(manager.delete_note(note.id).is_ok());
    
    // Проверяем, что файл удален
    assert!(!note_path.exists());
    
    // Проверяем, что заметка больше не загружается
    assert!(manager.load_note(note.id).is_err());
}

// Тест обработки несуществующей заметки
#[test]
fn test_nonexistent_note() {
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    let manager = NotesManager::new(notes_dir);
    
    // Пытаемся загрузить несуществующую заметку
    let nonexistent_id = Uuid::new_v4();
    assert!(manager.load_note(nonexistent_id).is_err());
    
    // Пытаемся удалить несуществующую заметку
    assert!(manager.delete_note(nonexistent_id).is_err());
}

// Тест сортировки заметок
#[test]
fn test_notes_sorting() {
    let temp_dir = TempDir::new().unwrap();
    let notes_dir = temp_dir.path().to_path_buf();
    let manager = NotesManager::new(notes_dir);
    
    // Создаем заметки с разным временем создания
    let mut note1 = create_test_note();
    note1.created_at = chrono::Utc::now() - chrono::Duration::hours(2);
    
    let mut note2 = create_test_note();
    note2.created_at = chrono::Utc::now() - chrono::Duration::hours(1);
    
    let mut note3 = create_test_note();
    note3.created_at = chrono::Utc::now();
    
    // Сохраняем заметки
    manager.save_note(&note1).unwrap();
    manager.save_note(&note2).unwrap();
    manager.save_note(&note3).unwrap();
    
    // Получаем все заметки
    let notes = manager.get_all_notes().unwrap();
    
    // Проверяем порядок (новые сверху)
    assert_eq!(notes[0].id, note3.id);
    assert_eq!(notes[1].id, note2.id);
    assert_eq!(notes[2].id, note1.id);
} 