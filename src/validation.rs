/// Модуль для валидации данных приложения
use uuid::Uuid;
use crate::notes::NoteGroup;

/// Результат валидации
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }
    

    
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }
}

/// Типы ошибок валидации
#[derive(Debug, Clone)]
pub enum ValidationError {
    EmptyTitle,
    TitleTooLong(usize),
    ContentTooLong(usize),
    EmptyGroupName,
    GroupNameTooLong(usize),
    CircularGroupDependency(Uuid),
    GroupNotFound(Uuid),
    InvalidGroupHierarchy,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyTitle => write!(f, "Заголовок заметки не может быть пустым"),
            ValidationError::TitleTooLong(len) => write!(f, "Заголовок слишком длинный: {} символов (максимум 255)", len),
            ValidationError::ContentTooLong(len) => write!(f, "Содержимое заметки слишком длинное: {} символов (максимум 1MB)", len),
            ValidationError::EmptyGroupName => write!(f, "Название группы не может быть пустым"),
            ValidationError::GroupNameTooLong(len) => write!(f, "Название группы слишком длинное: {} символов (максимум 100)", len),
            ValidationError::CircularGroupDependency(id) => write!(f, "Обнаружена циклическая зависимость в группе {}", id),
            ValidationError::GroupNotFound(id) => write!(f, "Группа с ID {} не найдена", id),
            ValidationError::InvalidGroupHierarchy => write!(f, "Неверная иерархия групп"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Правила валидации для приложения
pub struct ValidationRules;

impl ValidationRules {
    // Константы для ограничений
    pub const MAX_TITLE_LENGTH: usize = 255;
    pub const MAX_CONTENT_LENGTH: usize = 1_048_576; // 1MB
    pub const MAX_GROUP_NAME_LENGTH: usize = 100;
    pub const MAX_GROUP_DEPTH: u32 = 10;
    

    
    /// Валидация данных для создания заметки
    pub fn validate_note_creation(title: &str, content: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        // Проверка заголовка
        if title.trim().is_empty() {
            result.add_error(ValidationError::EmptyTitle);
        } else if title.len() > Self::MAX_TITLE_LENGTH {
            result.add_error(ValidationError::TitleTooLong(title.len()));
        }
        
        // Проверка содержимого
        if content.len() > Self::MAX_CONTENT_LENGTH {
            result.add_error(ValidationError::ContentTooLong(content.len()));
        }
        
        result
    }
    

    
    /// Валидация данных для создания группы
    pub fn validate_group_creation(name: &str, parent_id: Option<Uuid>, groups: &[NoteGroup]) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        // Проверка названия
        if name.trim().is_empty() {
            result.add_error(ValidationError::EmptyGroupName);
        } else if name.len() > Self::MAX_GROUP_NAME_LENGTH {
            result.add_error(ValidationError::GroupNameTooLong(name.len()));
        }
        
        // Проверка существования родительской группы
        if let Some(parent_id) = parent_id {
            if !groups.iter().any(|g| g.id == parent_id) {
                result.add_error(ValidationError::GroupNotFound(parent_id));
            } else {
                // Проверка глубины вложенности
                let parent_level = groups.iter()
                    .find(|g| g.id == parent_id)
                    .map(|g| g.level)
                    .unwrap_or(0);
                
                if parent_level >= Self::MAX_GROUP_DEPTH {
                    result.add_error(ValidationError::InvalidGroupHierarchy);
                }
            }
        }
        
        result
    }
    
    /// Валидация изменения родительской группы (проверка циклов)
    pub fn validate_group_parent_change(
        group_id: Uuid,
        new_parent_id: Option<Uuid>,
        groups: &[NoteGroup]
    ) -> ValidationResult {
        let mut result = ValidationResult::valid();
        
        if let Some(new_parent_id) = new_parent_id {
            // Проверка, что новая родительская группа существует
            if !groups.iter().any(|g| g.id == new_parent_id) {
                result.add_error(ValidationError::GroupNotFound(new_parent_id));
                return result;
            }
            
            // Проверка на циклические зависимости
            if Self::would_create_cycle(group_id, new_parent_id, groups) {
                result.add_error(ValidationError::CircularGroupDependency(group_id));
            }
        }
        
        result
    }
    
    /// Проверка, создаст ли изменение родителя циклическую зависимость
    fn would_create_cycle(group_id: Uuid, new_parent_id: Uuid, groups: &[NoteGroup]) -> bool {
        let mut current_id = new_parent_id;
        let mut visited = std::collections::HashSet::new();
        
        loop {
            if current_id == group_id {
                return true; // Найден цикл
            }
            
            if visited.contains(&current_id) {
                return false; // Цикл не через исходную группу
            }
            
            visited.insert(current_id);
            
            // Найти родителя текущей группы
            if let Some(group) = groups.iter().find(|g| g.id == current_id) {
                if let Some(parent_id) = group.parent_id {
                    current_id = parent_id;
                } else {
                    break; // Достигли корня
                }
            } else {
                break; // Группа не найдена
            }
        }
        
        false
    }
    

} 