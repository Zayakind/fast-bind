/// Структуры для действий, возвращаемых из панелей
use uuid::Uuid;
use crate::state::LoadMode;
use crate::ui::ThemeMode;

/// Действия для окна настроек
#[derive(Debug, Default)]
pub struct SettingsActions {
    pub theme_changed: Option<ThemeMode>,
    pub load_mode_changed: Option<LoadMode>,
    pub show_performance_stats_changed: Option<bool>,
    pub close_settings: bool,
}

/// Действия для боковой панели
#[derive(Debug, Default)]
pub struct SidePanelActions {
    pub selected_note: Option<usize>,
    pub new_note_clicked: bool,
    pub create_group_clicked: bool,
    pub show_settings_clicked: bool,
    pub show_group_editor_clicked: bool,
    pub toggled_group: Option<Uuid>,
    pub load_more_requested: Option<(usize, usize)>, // (visible_start, visible_end)
}

/// Действия для центральной панели
#[derive(Debug, Default)]
pub struct CentralPanelActions {
    pub save_note_clicked: bool,
    pub delete_note_clicked: bool,
    pub copy_to_clipboard_clicked: bool,
    pub copy_to_persistent_clicked: bool,
    pub toggle_pin: Option<usize>,
    pub update_title: Option<(usize, String)>,
    pub create_note_clicked: bool,
    pub persistent_text_changed: bool,
}

impl SidePanelActions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn select_note(&mut self, idx: usize) {
        self.selected_note = Some(idx);
    }
    
    pub fn new_note(&mut self) {
        self.new_note_clicked = true;
    }
    
    pub fn create_group(&mut self) {
        self.create_group_clicked = true;
    }
    
    pub fn show_settings(&mut self) {
        self.show_settings_clicked = true;
    }
    
    pub fn show_group_editor(&mut self) {
        self.show_group_editor_clicked = true;
    }
    
    pub fn toggle_group(&mut self, group_id: Uuid) {
        self.toggled_group = Some(group_id);
    }
    
    pub fn request_load_more(&mut self, visible_start: usize, visible_end: usize) {
        self.load_more_requested = Some((visible_start, visible_end));
    }
}

impl CentralPanelActions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn save_note(&mut self) {
        self.save_note_clicked = true;
    }
    
    pub fn delete_note(&mut self) {
        self.delete_note_clicked = true;
    }
    
    pub fn copy_to_clipboard(&mut self) {
        self.copy_to_clipboard_clicked = true;
    }
    
    pub fn copy_to_persistent(&mut self) {
        self.copy_to_persistent_clicked = true;
    }
    
    pub fn toggle_pin(&mut self, idx: usize) {
        self.toggle_pin = Some(idx);
    }
    
    pub fn update_title(&mut self, idx: usize, title: String) {
        self.update_title = Some((idx, title));
    }
    
    pub fn create_note(&mut self) {
        self.create_note_clicked = true;
    }
    
    pub fn persistent_text_changed(&mut self) {
        self.persistent_text_changed = true;
    }
}

impl SettingsActions {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn change_theme(&mut self, theme: ThemeMode) {
        self.theme_changed = Some(theme);
    }
    
    pub fn change_load_mode(&mut self, mode: LoadMode) {
        self.load_mode_changed = Some(mode);
    }
    
    pub fn toggle_performance_stats(&mut self, show: bool) {
        self.show_performance_stats_changed = Some(show);
    }
    
    pub fn close(&mut self) {
        self.close_settings = true;
    }
} 