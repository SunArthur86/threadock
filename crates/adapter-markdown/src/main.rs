//! Markdown Adapter 独立进程入口。
//!
//! 作为 stdio JSON-RPC server 运行（plan §10.4 隔离）。
//! adapter-host spawn 本二进制后，通过 stdin/stdout 调用。

use ch_adapter_sdk::protocol::{AdapterMetadata, HealthResponse, PROTOCOL_VERSION};
use ch_adapter_sdk::runtime::{serve_stdio, AdapterError, ConversationAdapter};
use ch_domain::Provider;

struct MarkdownAdapter;

impl ConversationAdapter for MarkdownAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: ch_adapter_markdown::ADAPTER_ID.into(),
            name: "Markdown".into(),
            version: ch_adapter_markdown::ADAPTER_VERSION.into(),
            protocol_version: PROTOCOL_VERSION,
            provider: Provider::Generic,
        }
    }

    fn parse(
        &self,
        source_id: &str,
        content: &[u8],
    ) -> Result<ch_normalization::RawConversation, AdapterError> {
        let text =
            std::str::from_utf8(content).map_err(|e| AdapterError::Parse(format!("utf8: {e}")))?;
        ch_adapter_markdown::parse_str(text, source_id)
            .map_err(|e| AdapterError::Parse(e.to_string()))
    }

    fn health(&self) -> HealthResponse {
        HealthResponse {
            healthy: true,
            detail: None,
        }
    }
}

fn main() {
    let adapter = MarkdownAdapter;
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve_stdio(
        &adapter,
        stdin.lock(),
        &mut std::io::BufWriter::new(stdout.lock()),
    );
}
