//! 领域层错误类型。

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),

    #[error("invalid id: {0}")]
    InvalidId(String),

    #[error("validation failed: {0}")]
    Validation(String),
}
