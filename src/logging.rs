/// Модуль для улучшенной обработки ошибок и логирования
use std::fmt;

/// Уровни логирования для приложения
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error,
    Info,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Info => write!(f, "INFO"),
        }
    }
}

/// Контекст операции для более информативного логирования
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub operation: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
}

impl OperationContext {
    pub fn new(operation: &str, entity_type: &str) -> Self {
        Self {
            operation: operation.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: None,
        }
    }
    
    pub fn with_id(mut self, id: &str) -> Self {
        self.entity_id = Some(id.to_string());
        self
    }
}

impl fmt::Display for OperationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.entity_id {
            write!(f, "{} {} ({})", self.operation, self.entity_type, id)
        } else {
            write!(f, "{} {}", self.operation, self.entity_type)
        }
    }
}

/// Структурированное логирование ошибок приложения
pub struct AppLogger;

impl AppLogger {
    /// Логирование ошибки с контекстом
    pub fn log_error(context: &OperationContext, error: &dyn std::error::Error) {
        log::error!("[{}] Ошибка: {}", context, error);
        
        // Логируем цепочку ошибок если есть
        let mut source = error.source();
        while let Some(err) = source {
            log::error!("  Вызвано: {}", err);
            source = err.source();
        }
    }
    
    /// Логирование информации об операции
    pub fn log_info(context: &OperationContext, message: &str) {
        log::info!("[{}] {}", context, message);
    }
    
    /// Логирование успешного завершения операции
    pub fn log_success(context: &OperationContext) {
        log::info!("[{}] Операция выполнена успешно", context);
    }
}

/// Макрос для упрощения создания контекста операции
#[macro_export]
macro_rules! operation_context {
    ($op:expr, $entity:expr) => {
        crate::logging::OperationContext::new($op, $entity)
    };
    ($op:expr, $entity:expr, $id:expr) => {
        crate::logging::OperationContext::new($op, $entity).with_id($id)
    };
}

/// Макрос для логирования ошибок с автоматическим созданием контекста
#[macro_export]
macro_rules! log_error {
    ($op:expr, $entity:expr, $error:expr) => {
        $crate::logging::AppLogger::log_error(
            &$crate::operation_context!($op, $entity),
            &$error
        )
    };
    ($op:expr, $entity:expr, $id:expr, $error:expr) => {
        $crate::logging::AppLogger::log_error(
            &$crate::operation_context!($op, $entity, $id),
            &$error
        )
    };
}

/// Макрос для логирования информации с автоматическим созданием контекста
#[macro_export]
macro_rules! log_info {
    ($op:expr, $entity:expr, $msg:expr) => {
        $crate::logging::AppLogger::log_info(
            &$crate::operation_context!($op, $entity),
            $msg
        )
    };
    ($op:expr, $entity:expr, $id:expr, $msg:expr) => {
        $crate::logging::AppLogger::log_info(
            &$crate::operation_context!($op, $entity, $id),
            $msg
        )
    };
}

/// Макрос для логирования успеха с автоматическим созданием контекста
#[macro_export]
macro_rules! log_success {
    ($op:expr, $entity:expr) => {
        $crate::logging::AppLogger::log_success(
            &$crate::operation_context!($op, $entity)
        )
    };
    ($op:expr, $entity:expr, $id:expr) => {
        $crate::logging::AppLogger::log_success(
            &$crate::operation_context!($op, $entity, $id)
        )
    };
} 