//! Conversation Hub 标准化流水线，对应 plan §11（数据采集与同步）与 §12.3（原始/标准化）。
//!
//! 职责：
//! - 计算内容 Hash（plan §11.3 幂等策略）。
//! - 评估导入完整度（plan §17.3：完整/部分/有限）。
//! - 把 Adapter 解析出的 `RawConversation` 规整为可入库的领域对象。

pub mod completeness;
pub mod hash;
pub mod pipeline;

pub use completeness::{completeness_score, Completeness};
pub use hash::{content_hash_for_conversation, content_hash_for_message};
pub use pipeline::{
    normalize, NormalizationError, NormalizationResult, RawConversation, RawEvent, RawMessage,
};
