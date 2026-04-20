use thiserror::Error;

/// 嵌入库错误类型
#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::content_manager::errors::StorageError),

    #[error("Operation error: {0}")]
    Operation(#[from] segment::common::operation_error::OperationError),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EmbeddedError>;
