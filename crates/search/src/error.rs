//! 搜索错误类型。

use thiserror::Error;

pub type SearchResult<T> = std::result::Result<T, SearchError>;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("tantivy error: {0}")]
    Tantivy(String),

    #[error("index io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("index not open")]
    NotOpen,

    #[error("invalid query: {0}")]
    InvalidQuery(String),
}
