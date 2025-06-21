use fast_bind::state::AppState;

mod common;
use common::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note_success() {
        // Тест успешного создания заметки
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();

        create_test_note(&mut app_state, &note_title, &note_content);
        
        // Проверяем, что заметка была создана
        assert_eq!(app_state.notes.len(), 1);
    }

    #[test]
    fn test_create_note_empty_title() {
        // Тест создания заметки с пустым заголовком
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_content = "This is a test note".to_string();
        
        let result = app_state.create_note("".to_string(), note_content, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_note_creation_before_deletion() {
        // Тест что заметка была создана перед удалением
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);
        assert_eq!(app_state.notes.len(), 1);
    }

    #[test]  
    fn test_delete_note_returns_ok() {
        // Тест что удаление заметки возвращает Ok
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let res_delete = app_state.delete_note(0);
        assert!(res_delete.is_ok());
    }

    #[test]
    fn test_delete_note_removes_from_collection() {
        // Тест что заметка действительно удалена из коллекции
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let _ = app_state.delete_note(0);
        
        assert_eq!(app_state.notes.len(), 0);
    }

    #[test]
    fn test_note_creation_before_update() {
        // Тест что заметка была создана перед обновлением
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);
        assert_eq!(app_state.notes.len(), 1);
    }

    #[test]
    fn test_update_note_returns_ok() {
        // Тест что обновление заметки возвращает Ok
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let res_update = app_state.update_note(0, Some(note_title), Some("test".to_string()));
        assert!(res_update.is_ok());
    }

    #[test]
    fn test_update_note_changes_content() {
        // Тест что содержимое заметки действительно изменилось
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let _ = app_state.update_note(0, Some(note_title), Some("test".to_string()));

        assert_eq!(app_state.notes[0].content, "test");
    }

    #[test]
    fn test_toggle_pin_to_true() {
        // Тест закрепления заметки
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let _ = app_state.toggle_pin(0);
        assert_eq!(app_state.notes[0].pinned, true);
    }

    #[test]
    fn test_toggle_pin_to_false() {
        // Тест открепления заметки
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, &note_title, &note_content);

        let _ = app_state.toggle_pin(0);
        let _ = app_state.toggle_pin(0);
        assert_eq!(app_state.notes[0].pinned, false);
    }

    #[test]
    fn test_sort_notes_preparation() {
        // Тест создания заметок для сортировки
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, "Test Note", &note_content);
        create_test_note(&mut app_state, "Abcd", &note_content);

        assert_eq!(app_state.notes.len(), 2);
    }

    #[test]
    fn test_sort_notes_alphabetical_order() {
        // Тест сортировки заметок по алфавиту
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_content = "This is a test note".to_string();
        
        create_test_note(&mut app_state, "Test Note", &note_content);
        create_test_note(&mut app_state, "Abcd", &note_content);

        AppState::sort_notes(&mut app_state.notes);

        assert_eq!(app_state.notes[0].title, "Abcd");
    }

    #[test]
    fn test_get_note_content() {
        let (mut app_state, _temp_dir) = create_test_app_state();
        let note_title = "Test Note".to_string();
        let note_content = "This is a test note".to_string();

        create_test_note(&mut app_state, &note_title, &note_content);

        let content = app_state.get_note_content(0);

        assert_eq!(content, Some(note_content));
    }

    #[test]
    fn test_get_note_content_invalid_index() {
        let (app_state, _temp_dir) = create_test_app_state();
        
        // Пробуем получить содержимое несуществующей заметки
        let content = app_state.get_note_content(999);
        
        assert_eq!(content, None);
    }

    #[test]
    fn test_append_note_to_persistent() {
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        // Устанавливаем начальный текст
        app_state.persistent_text = "Existing text. ".to_string();
        
        let note_content = "This is a test note".to_string();
        create_test_note(&mut app_state, "Test Note", &note_content);

        let _ = app_state.append_note_to_persistent(0);

        assert_eq!(app_state.persistent_text, "Existing text. This is a test note");
    }

    #[test]
    fn test_append_note_to_persistent_invalid_index_returns_ok() {
        // Тест что добавление несуществующей заметки не падает
        let (mut app_state, _temp_dir) = create_test_app_state();
        let _initial_text = app_state.persistent_text.clone();
        
        let result = app_state.append_note_to_persistent(999);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_append_note_to_persistent_invalid_index_text_unchanged() {
        // Тест что текст не изменился при добавлении несуществующей заметки
        let (mut app_state, _temp_dir) = create_test_app_state();
        let initial_text = app_state.persistent_text.clone();
        
        let _ = app_state.append_note_to_persistent(999);
        
        assert_eq!(app_state.persistent_text, initial_text);
    }

    #[test]
    fn test_save_persistent_text_returns_ok() {
        // Тест что сохранение постоянного текста возвращает Ok
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        app_state.persistent_text = "Test persistent content".to_string();
        
        let result = app_state.save_persistent_text();
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_persistent_text_creates_file() {
        // Тест что файл действительно создается
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        app_state.persistent_text = "Test persistent content".to_string();
        
        let _ = app_state.save_persistent_text();
        
        let persistent_file = app_state.notes_manager.get_base_dir().join("persistent_text.txt");
        assert!(persistent_file.exists());
    }

    #[test]
    fn test_save_persistent_text_correct_content() {
        // Тест что файл содержит правильные данные
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        app_state.persistent_text = "Test persistent content".to_string();
        
        let _ = app_state.save_persistent_text();
        
        let persistent_file = app_state.notes_manager.get_base_dir().join("persistent_text.txt");
        let file_content = std::fs::read_to_string(persistent_file).unwrap();
        assert_eq!(file_content, "Test persistent content");
    }

    #[test]
    fn test_create_group_success() {
        // Тест успешного создания группы
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_name = "Test Group".to_string();

        create_test_group(&mut app_state, &group_name);
        assert_eq!(app_state.groups.len(), 1);
    }

    #[test]
    fn test_create_group_invalid_name() {
        // Тест создания группы с некорректным именем
        let (mut app_state, _temp_dir) = create_test_app_state();

        let result = app_state.create_group("".to_string(), None, vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_group() {
        // Тест удаления группы
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_name = "Test Group".to_string();

        let uuid = create_test_group(&mut app_state, &group_name);

        let _ =app_state.delete_group(uuid);

        assert_eq!(app_state.groups.len(), 0);
    }

    #[test]
    fn test_update_group() {
        // Тест обновления группы
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_name = "Test Group".to_string();

        let uuid = create_test_group(&mut app_state, &group_name);

        let _ =app_state.update_group(uuid, "Test Group 2".to_string(), None);

        assert_eq!(app_state.groups[0].name, "Test Group 2");
    }

    #[test]
    fn test_toggle_group_collapsed_to_true() {
        // Тест сворачивания группы
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_name = "Test Group".to_string();

        let uuid = create_test_group(&mut app_state, &group_name);

        let _ = app_state.toggle_group_collapsed(uuid);

        assert_eq!(app_state.groups[0].collapsed, true);
    }

    #[test]
    fn test_toggle_group_collapsed_to_false() {
        // Тест разворачивания группы
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_name = "Test Group".to_string();

        let uuid = create_test_group(&mut app_state, &group_name);

        let _ = app_state.toggle_group_collapsed(uuid);
        let _ = app_state.toggle_group_collapsed(uuid);

        assert_eq!(app_state.groups[0].collapsed, false);
    }

    #[test]
    fn test_root_group_properties() {
        // Тест свойств корневой группы в иерархии
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (root_group_id, _, _) = create_test_group_hierarchy(&mut app_state);
        
        let root_group = app_state.groups.iter().find(|g| g.id == root_group_id).unwrap();
        assert_eq!(root_group.level, 0);
    }

    #[test]
    fn test_root_group_has_no_parent() {
        // Тест что корневая группа не имеет родителя
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (root_group_id, _, _) = create_test_group_hierarchy(&mut app_state);
        
        let root_group = app_state.groups.iter().find(|g| g.id == root_group_id).unwrap();
        assert_eq!(root_group.parent_id, None);
    }

    #[test]
    fn test_child_group_properties() {
        // Тест свойств дочерней группы в иерархии
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_root_group_id, child_group_id, _) = create_test_group_hierarchy(&mut app_state);
        
        let child_group = app_state.groups.iter().find(|g| g.id == child_group_id).unwrap();
        assert_eq!(child_group.level, 1);
    }

    #[test]
    fn test_child_group_has_correct_parent() {
        // Тест что дочерняя группа имеет правильного родителя
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (root_group_id, child_group_id, _) = create_test_group_hierarchy(&mut app_state);
        
        let child_group = app_state.groups.iter().find(|g| g.id == child_group_id).unwrap();
        assert_eq!(child_group.parent_id, Some(root_group_id));
    }

    #[test]
    fn test_grandchild_group_properties() {
        // Тест свойств внучатой группы в иерархии
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_, _, grandchild_group_id) = create_test_group_hierarchy(&mut app_state);
        
        let grandchild_group = app_state.groups.iter().find(|g| g.id == grandchild_group_id).unwrap();
        assert_eq!(grandchild_group.level, 2);
    }

    #[test]
    fn test_grandchild_group_has_correct_parent() {
        // Тест что внучатая группа имеет правильного родителя
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_, child_group_id, grandchild_group_id) = create_test_group_hierarchy(&mut app_state);
        
        let grandchild_group = app_state.groups.iter().find(|g| g.id == grandchild_group_id).unwrap();
        assert_eq!(grandchild_group.parent_id, Some(child_group_id));
    }

    #[test]
    fn test_group_deletion_updates_children_parent() {
        // Тест что удаление промежуточной группы обновляет родителя дочерних групп
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (root_group_id, child_group_id, grandchild_group_id) = create_test_group_hierarchy(&mut app_state);
        
        let _ = app_state.delete_group(child_group_id);
        
        let updated_grandchild = app_state.groups.iter().find(|g| g.id == grandchild_group_id).unwrap();
        assert_eq!(updated_grandchild.parent_id, Some(root_group_id));
    }

    #[test]
    fn test_group_deletion_updates_children_level() {
        // Тест что удаление промежуточной группы обновляет уровень дочерних групп
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_root_group_id, child_group_id, grandchild_group_id) = create_test_group_hierarchy(&mut app_state);
        
        let _ = app_state.delete_group(child_group_id);
        
        let updated_grandchild = app_state.groups.iter().find(|g| g.id == grandchild_group_id).unwrap();
        assert_eq!(updated_grandchild.level, 1);
    }

    #[test]
    fn test_group_deletion_removes_group_from_collection() {
        // Тест что удаленная группа исчезает из коллекции
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_, child_group_id, _) = create_test_group_hierarchy(&mut app_state);
        
        let _ = app_state.delete_group(child_group_id);
        
        assert!(!app_state.groups.iter().any(|g| g.id == child_group_id));
    }

    #[test]
    fn test_group_deletion_correct_remaining_count() {
        // Тест что после удаления промежуточной группы остается правильное количество групп
        let (mut app_state, _temp_dir) = create_test_app_state();
        
        let (_, child_group_id, _) = create_test_group_hierarchy(&mut app_state);
        
        let _ = app_state.delete_group(child_group_id);
        
        assert_eq!(app_state.groups.len(), 2);
    }

    #[test]
    fn test_parent_group_notes_count() {
        // Тест количества заметок в родительской группе
        let (mut app_state, _temp_dir) = create_test_app_state();

        let parent_group_id = create_test_group(&mut app_state, "Parent Group");
        let child_group_id = create_test_group_full(&mut app_state, "Child Group", Some(parent_group_id), vec![]);
        
        create_test_note_with_group(&mut app_state, "Parent Note", "Content", Some(parent_group_id));
        create_test_note_with_group(&mut app_state, "Child Note", "Content", Some(child_group_id));
        create_test_note(&mut app_state, "Ungrouped Note", "Content");

        let parent_notes: Vec<_> = app_state.notes.iter()
            .filter(|n| n.group_id == Some(parent_group_id))
            .collect();
            
        assert_eq!(parent_notes.len(), 1);
    }

    #[test]
    fn test_child_group_notes_count() {
        // Тест количества заметок в дочерней группе
        let (mut app_state, _temp_dir) = create_test_app_state();

        let parent_group_id = create_test_group(&mut app_state, "Parent Group");
        let child_group_id = create_test_group_full(&mut app_state, "Child Group", Some(parent_group_id), vec![]);
        
        create_test_note_with_group(&mut app_state, "Parent Note", "Content", Some(parent_group_id));
        create_test_note_with_group(&mut app_state, "Child Note", "Content", Some(child_group_id));
        create_test_note(&mut app_state, "Ungrouped Note", "Content");

        let child_notes: Vec<_> = app_state.notes.iter()
            .filter(|n| n.group_id == Some(child_group_id))
            .collect();
            
        assert_eq!(child_notes.len(), 1);
    }

    #[test]
    fn test_ungrouped_notes_count() {
        // Тест количества заметок без группы
        let (mut app_state, _temp_dir) = create_test_app_state();

        let parent_group_id = create_test_group(&mut app_state, "Parent Group");
        let child_group_id = create_test_group_full(&mut app_state, "Child Group", Some(parent_group_id), vec![]);
        
        create_test_note_with_group(&mut app_state, "Parent Note", "Content", Some(parent_group_id));
        create_test_note_with_group(&mut app_state, "Child Note", "Content", Some(child_group_id));
        create_test_note(&mut app_state, "Ungrouped Note", "Content");

        let ungrouped_notes: Vec<_> = app_state.notes.iter()
            .filter(|n| n.group_id.is_none())
            .collect();
            
        assert_eq!(ungrouped_notes.len(), 1);
    }

    #[test]
    fn test_child_group_deletion_moves_notes_to_parent() {
        // Тест что удаление дочерней группы перемещает заметки к родителю
        let (mut app_state, _temp_dir) = create_test_app_state();

        let parent_group_id = create_test_group(&mut app_state, "Parent Group");
        let child_group_id = create_test_group_full(&mut app_state, "Child Group", Some(parent_group_id), vec![]);
        
        create_test_note_with_group(&mut app_state, "Parent Note", "Content", Some(parent_group_id));
        create_test_note_with_group(&mut app_state, "Child Note", "Content", Some(child_group_id));
        create_test_note(&mut app_state, "Ungrouped Note", "Content");

        let _ = app_state.delete_group(child_group_id);

        let parent_notes_after: Vec<_> = app_state.notes.iter()
            .filter(|n| n.group_id == Some(parent_group_id))
            .collect();
        assert_eq!(parent_notes_after.len(), 2);
    }

    #[test]
    fn test_group_initial_parent() {
        // Тест начального родителя дочерней группы
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let _group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(child.parent_id, Some(group1_id));
    }

    #[test]
    fn test_group_initial_level() {
        // Тест начального уровня дочерней группы
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let _group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(child.level, 1);
    }

    #[test]
    fn test_group_parent_change() {
        // Тест изменения родителя группы
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let _ = app_state.update_group(child_id, "Child".to_string(), Some(group2_id));

        let updated_child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(updated_child.parent_id, Some(group2_id));
    }

    #[test]
    fn test_group_parent_change_maintains_level() {
        // Тест что изменение родителя сохраняет уровень группы
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let _ = app_state.update_group(child_id, "Child".to_string(), Some(group2_id));

        let updated_child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(updated_child.level, 1);
    }

    #[test]
    fn test_group_becomes_root_parent() {
        // Тест что группа становится корневой (без родителя)
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let _group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let _ = app_state.update_group(child_id, "Child".to_string(), None);

        let root_child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(root_child.parent_id, None);
    }

    #[test]
    fn test_group_becomes_root_level() {
        // Тест что группа становится корневой (уровень 0)
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group1_id = create_test_group(&mut app_state, "Group 1");
        let _group2_id = create_test_group(&mut app_state, "Group 2");
        
        let child_id = create_test_group_full(&mut app_state, "Child", Some(group1_id), vec![]);

        let _ = app_state.update_group(child_id, "Child".to_string(), None);

        let root_child = app_state.groups.iter().find(|g| g.id == child_id).unwrap();
        assert_eq!(root_child.level, 0);
    }

    #[test]
    fn test_circular_dependency_prevention_root_to_grandchild() {
        // Тест предотвращения циклической зависимости: корневая группа не может стать дочерней внучатой
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group_a_id = create_test_group(&mut app_state, "Group A");
        let group_b_id = create_test_group_full(&mut app_state, "Group B", Some(group_a_id), vec![]);
        let group_c_id = create_test_group_full(&mut app_state, "Group C", Some(group_b_id), vec![]);

        let result = app_state.update_group(group_a_id, "Group A".to_string(), Some(group_c_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_dependency_prevention_root_parent_unchanged() {
        // Тест что родитель корневой группы остается неизменным при попытке создания цикла
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group_a_id = create_test_group(&mut app_state, "Group A");
        let group_b_id = create_test_group_full(&mut app_state, "Group B", Some(group_a_id), vec![]);
        let group_c_id = create_test_group_full(&mut app_state, "Group C", Some(group_b_id), vec![]);

        let _ = app_state.update_group(group_a_id, "Group A".to_string(), Some(group_c_id));

        let group_a = app_state.groups.iter().find(|g| g.id == group_a_id).unwrap();
        assert_eq!(group_a.parent_id, None);
    }

    #[test]
    fn test_circular_dependency_prevention_child_to_grandchild() {
        // Тест предотвращения циклической зависимости: дочерняя группа не может стать дочерней внучатой
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group_a_id = create_test_group(&mut app_state, "Group A");
        let group_b_id = create_test_group_full(&mut app_state, "Group B", Some(group_a_id), vec![]);
        let group_c_id = create_test_group_full(&mut app_state, "Group C", Some(group_b_id), vec![]);

        let result = app_state.update_group(group_b_id, "Group B".to_string(), Some(group_c_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_dependency_prevention_child_parent_unchanged() {
        // Тест что родитель дочерней группы остается неизменным при попытке создания цикла
        let (mut app_state, _temp_dir) = create_test_app_state();

        let group_a_id = create_test_group(&mut app_state, "Group A");
        let group_b_id = create_test_group_full(&mut app_state, "Group B", Some(group_a_id), vec![]);
        let group_c_id = create_test_group_full(&mut app_state, "Group C", Some(group_b_id), vec![]);

        let _ = app_state.update_group(group_b_id, "Group B".to_string(), Some(group_c_id));

        let group_b = app_state.groups.iter().find(|g| g.id == group_b_id).unwrap();
        assert_eq!(group_b.parent_id, Some(group_a_id));
    }
}
