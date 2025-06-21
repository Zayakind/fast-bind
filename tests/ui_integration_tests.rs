use fast_bind::ui::{SidePanelActions, CentralPanelActions};
use uuid::Uuid;

mod common;
use common::*;

#[cfg(test)]
mod tests {
    use super::*;

    // === SidePanelActions Начальное состояние ===
    
    #[test]
    fn side_panel_actions_selected_note_is_none_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.selected_note, None);
    }

    #[test]
    fn side_panel_actions_new_note_clicked_is_false_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.new_note_clicked, false);
    }

    #[test]
    fn side_panel_actions_create_group_clicked_is_false_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.create_group_clicked, false);
    }

    #[test]
    fn side_panel_actions_show_settings_clicked_is_false_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.show_settings_clicked, false);
    }

    #[test]
    fn side_panel_actions_show_group_editor_clicked_is_false_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.show_group_editor_clicked, false);
    }

    #[test]
    fn side_panel_actions_toggled_group_is_none_initially() {
        let actions = SidePanelActions::new();
        assert_eq!(actions.toggled_group, None);
    }

    // === SidePanelActions Методы ===

    #[test]
    fn side_panel_actions_select_note_sets_selected_note() {
        let mut actions = SidePanelActions::new();
        actions.select_note(5);
        assert_eq!(actions.selected_note, Some(5));
    }

    #[test]
    fn side_panel_actions_new_note_sets_flag_to_true() {
        let mut actions = SidePanelActions::new();
        actions.new_note();
        assert_eq!(actions.new_note_clicked, true);
    }

    #[test]
    fn side_panel_actions_create_group_sets_flag_to_true() {
        let mut actions = SidePanelActions::new();
        actions.create_group();
        assert_eq!(actions.create_group_clicked, true);
    }

    #[test]
    fn side_panel_actions_show_settings_sets_flag_to_true() {
        let mut actions = SidePanelActions::new();
        actions.show_settings();
        assert_eq!(actions.show_settings_clicked, true);
    }

    #[test]
    fn side_panel_actions_show_group_editor_sets_flag_to_true() {
        let mut actions = SidePanelActions::new();
        actions.show_group_editor();
        assert_eq!(actions.show_group_editor_clicked, true);
    }

    #[test]
    fn side_panel_actions_toggle_group_sets_group_id() {
        let mut actions = SidePanelActions::new();
        let test_group_id = Uuid::new_v4();
        actions.toggle_group(test_group_id);
        assert_eq!(actions.toggled_group, Some(test_group_id));
    }

    // === CentralPanelActions Начальное состояние ===

    #[test]
    fn central_panel_actions_save_note_clicked_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.save_note_clicked, false);
    }

    #[test]
    fn central_panel_actions_delete_note_clicked_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.delete_note_clicked, false);
    }

    #[test]
    fn central_panel_actions_copy_to_clipboard_clicked_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.copy_to_clipboard_clicked, false);
    }

    #[test]
    fn central_panel_actions_copy_to_persistent_clicked_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.copy_to_persistent_clicked, false);
    }

    #[test]
    fn central_panel_actions_toggle_pin_is_none_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.toggle_pin, None);
    }

    #[test]
    fn central_panel_actions_update_title_is_none_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.update_title, None);
    }

    #[test]
    fn central_panel_actions_create_note_clicked_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.create_note_clicked, false);
    }

    #[test]
    fn central_panel_actions_persistent_text_changed_is_false_initially() {
        let actions = CentralPanelActions::new();
        assert_eq!(actions.persistent_text_changed, false);
    }

    // === CentralPanelActions Методы ===

    #[test]
    fn central_panel_actions_save_note_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.save_note();
        assert_eq!(actions.save_note_clicked, true);
    }

    #[test]
    fn central_panel_actions_delete_note_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.delete_note();
        assert_eq!(actions.delete_note_clicked, true);
    }

    #[test]
    fn central_panel_actions_copy_to_clipboard_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.copy_to_clipboard();
        assert_eq!(actions.copy_to_clipboard_clicked, true);
    }

    #[test]
    fn central_panel_actions_copy_to_persistent_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.copy_to_persistent();
        assert_eq!(actions.copy_to_persistent_clicked, true);
    }

    #[test]
    fn central_panel_actions_toggle_pin_sets_index() {
        let mut actions = CentralPanelActions::new();
        actions.toggle_pin(3);
        assert_eq!(actions.toggle_pin, Some(3));
    }

    #[test]
    fn central_panel_actions_update_title_sets_index_and_title() {
        let mut actions = CentralPanelActions::new();
        actions.update_title(2, "New Title".to_string());
        assert_eq!(actions.update_title, Some((2, "New Title".to_string())));
    }

    #[test]
    fn central_panel_actions_create_note_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.create_note();
        assert_eq!(actions.create_note_clicked, true);
    }

    #[test]
    fn central_panel_actions_persistent_text_changed_sets_flag_to_true() {
        let mut actions = CentralPanelActions::new();
        actions.persistent_text_changed();
        assert_eq!(actions.persistent_text_changed, true);
    }

    // === Тесты независимости действий ===

    #[test]
    fn side_actions_do_not_affect_central_actions() {
        let mut side_actions = SidePanelActions::new();
        let central_actions = CentralPanelActions::new();
        
        side_actions.select_note(1);
        side_actions.new_note();
        
        assert_eq!(central_actions.save_note_clicked, false);
    }

    #[test]
    fn central_actions_do_not_affect_side_actions() {
        let side_actions = SidePanelActions::new();
        let mut central_actions = CentralPanelActions::new();
        
        central_actions.save_note();
        central_actions.delete_note();
        
        assert_eq!(side_actions.selected_note, None);
    }

    #[test]
    fn side_actions_preserve_other_fields_when_modified() {
        let mut side_actions = SidePanelActions::new();
        
        side_actions.select_note(1);
        
        assert_eq!(side_actions.create_group_clicked, false);
    }

    #[test]
    fn central_actions_preserve_other_fields_when_modified() {
        let mut central_actions = CentralPanelActions::new();
        
        central_actions.save_note();
        
        assert_eq!(central_actions.copy_to_clipboard_clicked, false);
    }

    // === Сценарии интеграции с AppState ===

    #[test] 
    fn side_actions_select_note_with_valid_index() {
        let (mut app_state, _temp_dir) = create_test_app_state();
        create_test_note(&mut app_state, "Test Note", "Content");
        
        let mut side_actions = SidePanelActions::new();
        side_actions.select_note(0);
        
        assert!(side_actions.selected_note.unwrap() < app_state.notes.len());
    }

    #[test]
    fn side_actions_toggle_group_stores_valid_group_id() {
        let (mut app_state, _temp_dir) = create_test_app_state();
        let group_id = create_test_group(&mut app_state, "Test Group");
        
        let mut side_actions = SidePanelActions::new();
        side_actions.toggle_group(group_id);
        
        assert_eq!(side_actions.toggled_group, Some(group_id));
    }

    #[test]
    fn central_actions_toggle_pin_with_valid_index() {
        let (mut app_state, _temp_dir) = create_test_app_state();
        create_test_note(&mut app_state, "Test Note 1", "Content 1");
        create_test_note(&mut app_state, "Test Note 2", "Content 2");
        
        let mut central_actions = CentralPanelActions::new();
        central_actions.toggle_pin(1);
        
        assert!(central_actions.toggle_pin.unwrap() < app_state.notes.len());
    }
}