//! Daemon 状态：持有主数据、搜索索引、Raw Store（plan §9.4 单点写）。

use ch_raw_store::RawStore;
use ch_search::SearchIndex;
use ch_storage::Repository;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct DaemonStateConfig {
    pub data_dir: PathBuf,
}

/// Daemon 全局状态。所有字段 Mutex 保护，支持并发 JSON-RPC 调用。
pub struct DaemonState {
    pub repo: Mutex<Repository>,
    pub search_index: Mutex<SearchIndex>,
    pub raw_store: Mutex<RawStore>,
    pub data_dir: PathBuf,
}

impl DaemonState {
    /// 在 data_dir 下打开/创建 Repository + SearchIndex + RawStore。
    pub fn open(config: DaemonStateConfig) -> Result<Self, DaemonStateError> {
        std::fs::create_dir_all(&config.data_dir)?;
        let db_path = config.data_dir.join("conversation-hub.db");
        let repo = Repository::open(&db_path)?;
        let search_index = SearchIndex::open(config.data_dir.join("index"))?;
        let raw_store = RawStore::new(&config.data_dir)?;
        Ok(Self {
            repo: Mutex::new(repo),
            search_index: Mutex::new(search_index),
            raw_store: Mutex::new(raw_store),
            data_dir: config.data_dir,
        })
    }

    /// 内存模式（测试用）。SQLite 用内存库，Tantivy 用 RAM 索引，Raw 用临时目录。
    pub fn open_in_memory() -> Result<Self, DaemonStateError> {
        let dir = tempfile::TempDir::new().map_err(DaemonStateError::Io)?;
        let db_path = dir.path().join("conversation-hub.db");
        let repo = Repository::open(&db_path)?;
        // Tantivy 用 RAM 索引（避免持久化锁问题）
        let search_index = SearchIndex::open_in_memory()?;
        let raw_store = RawStore::new(dir.path())?;
        Ok(Self {
            repo: Mutex::new(repo),
            search_index: Mutex::new(search_index),
            raw_store: Mutex::new(raw_store),
            data_dir: dir.path().to_path_buf(),
        })
    }

    /// 清空所有数据（conversations/workspaces/providers + 搜索索引 + raw blobs）。
    /// 保留 schema 和用户自定义脱敏规则。用于「重置数据」。
    pub fn wipe_all(&self) -> Result<(), DaemonStateError> {
        self.repo.lock().unwrap().clear_all()?;
        self.search_index.lock().unwrap().clear_all()?;
        self.raw_store.lock().unwrap().clear()?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonStateError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage: {0}")]
    Storage(#[from] ch_storage::StorageError),
    #[error("search: {0}")]
    Search(#[from] ch_search::SearchError),
    #[error("raw store: {0}")]
    Raw(#[from] ch_raw_store::RawStoreError),
}
