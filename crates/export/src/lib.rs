//! 会话导出，对应 plan §6.6「导出与备份」。
//!
//! 能力：
//! - 导出单条会话为 Markdown（人读）。
//! - 导出为 JSON（机器读，可重新导入）。
//! - 导出前敏感信息扫描 + 脱敏（plan §14.6：密钥/Token/邮箱）。
//! - 可选择是否包含命令/Diff/Artifact。

pub mod markdown;
pub mod redact;
pub mod serialize;

pub use markdown::to_markdown;
pub use redact::{redact, redact_with, CustomRule, RedactionRule, RedactionStats};
pub use serialize::{to_json, ExportData, ExportOptions};
