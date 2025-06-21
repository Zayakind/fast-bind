#![allow(dead_code)]

use fast_bind::state::AppState;
use fast_bind::notes::NotesManager;
use uuid::Uuid;
use tempfile::TempDir;

/// Создает тестовое состояние приложения с временной директорией
pub fn create_test_app_state() -> (AppState, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let notes_manager = NotesManager::new(temp_dir.path().to_path_buf());
    let app_state = AppState::new(notes_manager);
    (app_state, temp_dir)
}

/// Создает тестовую заметку без группы
pub fn create_test_note(app_state: &mut AppState, title: &str, content: &str) {
    let result = app_state.create_note(title.to_string(), content.to_string(), None);
    assert!(result.is_ok(), "Failed to create test note: {:?}", result.err());
}

/// Создает тестовую заметку с указанной группой
pub fn create_test_note_with_group(app_state: &mut AppState, title: &str, content: &str, group_id: Option<Uuid>) {
    let result = app_state.create_note(title.to_string(), content.to_string(), group_id);
    assert!(result.is_ok(), "Failed to create test note with group: {:?}", result.err());
}

/// Создает простую тестовую группу без родителя и заметок
pub fn create_test_group(app_state: &mut AppState, name: &str) -> Uuid {
    let result = app_state.create_group(name.to_string(), None, vec![]);
    assert!(result.is_ok(), "Failed to create test group: {:?}", result.err());
    result.unwrap()
}

/// Создает тестовую группу с полными параметрами (родитель, заметки)
pub fn create_test_group_full(app_state: &mut AppState, name: &str, parent_id: Option<Uuid>, selected_notes: Vec<Uuid>) -> Uuid {
    let result = app_state.create_group(name.to_string(), parent_id, selected_notes);
    assert!(result.is_ok(), "Failed to create test group with full params: {:?}", result.err());
    result.unwrap()
}

/// Создает несколько тестовых заметок для массовых операций
pub fn create_multiple_test_notes(app_state: &mut AppState, count: usize) -> Vec<String> {
    let mut titles = Vec::new();
    for i in 0..count {
        let title = format!("Test Note {}", i + 1);
        let content = format!("Content for test note {}", i + 1);
        create_test_note(app_state, &title, &content);
        titles.push(title);
    }
    titles
}

/// Создает иерархию групп для тестирования
pub fn create_test_group_hierarchy(app_state: &mut AppState) -> (Uuid, Uuid, Uuid) {
    let root_id = create_test_group(app_state, "Root Group");
    let child_id = create_test_group_full(app_state, "Child Group", Some(root_id), vec![]);
    let grandchild_id = create_test_group_full(app_state, "Grandchild Group", Some(child_id), vec![]);
    
    (root_id, child_id, grandchild_id)
}

/// Вспомогательная функция для проверки количества заметок
pub fn assert_notes_count(app_state: &AppState, expected_count: usize) {
    assert_eq!(app_state.notes.len(), expected_count, "Expected {} notes, but found {}", expected_count, app_state.notes.len());
}

/// Вспомогательная функция для проверки количества групп  
pub fn assert_groups_count(app_state: &AppState, expected_count: usize) {
    assert_eq!(app_state.groups.len(), expected_count, "Expected {} groups, but found {}", expected_count, app_state.groups.len());
} 