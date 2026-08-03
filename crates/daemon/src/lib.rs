//! Conversation Hub 本地常驻服务，对应 plan §8.2「本地组件架构」与 §16「接口与协议设计」。
//!
//! ## 角色
//!
//! Daemon 是主数据 + 索引的唯一持有者和写者（plan §9.4 单点写）。
//! UI / CLI / 其它客户端通过 JSON-RPC 2.0（newline-delimited）over stdio 与它通信。
//!
//! ## 协议方法（plan §16.1 子集）
//!
//! - `system.getInfo` —— 返回版本/状态
//! - `workspace.list` —— 列出所有 workspace
//! - `conversation.list` —— 列出会话（可按 workspace 过滤）
//! - `conversation.get` —— 获取单条会话详情
//! - `message.list` —— 列出会话的消息
//! - `search.query` —— 全文搜索（默认走 FTS5，可切 Tantivy）
//! - `provider.sync` —— 导入一个文件（解析 + 入库 + 索引）
//!
//! 复用现有 crate 的能力，不在 daemon 内重新实现业务逻辑。

pub mod protocol;
pub mod server;
pub mod state;

pub use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use server::serve_stdio;
pub use state::{DaemonState, DaemonStateConfig};
