# 实施计划：主子任务层级 + 数据重置 + 异步加载 + 定时同步 + 时间优化

## 概述
8 项改动，按依赖顺序分 6 个阶段。核心是给数据模型加 `source_parent_id` 字段打通主子任务链路。

---

## 阶段 1：数据模型 — `source_parent_id` 字段（基础设施，其他都依赖它）

### 1.1 `crates/domain/src/lib.rs`
- `Conversation` struct 末尾加 `pub source_parent_id: Option<String>,`
- `Conversation::new()` 初始化加 `source_parent_id: None,`

### 1.2 `crates/normalization/src/pipeline.rs`
- `RawConversation` struct 加 `pub source_parent_id: Option<String>,`
- `normalize()` 中加 `conversation.source_parent_id = raw.source_parent_id.clone();`
- 2 处测试字面量加 `source_parent_id: None,`

### 1.3 `crates/storage/src/schema.rs` + `migration.rs`
- 新增 `SCHEMA_V5`：`ALTER TABLE conversations ADD COLUMN source_parent_id TEXT;` + 索引 `idx_conversations_parent`
- `LATEST_VERSION` 改为 5，`migrations()` 加 V5 条目

### 1.4 `crates/storage/src/repository.rs`
- `upsert_conversation`：INSERT 列 + VALUES `?18` + ON CONFLICT SET + params 加 `source_parent_id`
- `row_to_conversation`：加 `c.source_parent_id = r.get(17)?;`
- 3 处 SELECT（`get_conversation`/`list_conversations`/`list_conversations_filtered`）列尾加 `, c.source_parent_id`
- 新增 `list_child_conversations(&self, parent_source_id, provider) -> Vec<Conversation>`：按 source_parent_id 查子任务
- 新增 `clear_all(&self)`：删除所有表数据（conversations 级联删 messages/events/tags/knowledge，再删 workspaces/providers/redaction_rules）

### 1.5 8 个 adapter 的 `RawConversation { ... }` 字面量
- 各加 `source_parent_id: None,`（cursor/zcode/claude-code/jsonl/markdown/sdk/host + minimax）

---

## 阶段 2：MiniMax adapter 支持主子任务

### 2.1 `crates/adapter-minimax/src/lib.rs`
- `parse_session()` 从 `record_json` 提取 `parentSessionId` → 填入 `RawConversation.source_parent_id`
- 这样导入主任务时 parent=null，导入子任务时 parent=主任务 source_id

### 2.2 Tauri 后端 `auto_sync` 中 MiniMax 块
- 改为：先导入所有主任务，再导入所有子任务（子任务带 source_parent_id）
- 幂等检查保持不变（按 source_conversation_id + provider）

---

## 阶段 3：Tauri 后端命令

### 3.1 `apps/desktop/src-tauri/src/lib.rs`
- **`ConversationDto`** 加 `source_parent_id: Option<String>` + `child_count: i64`（child_count 通过查询统计）
- **`WorkspaceDto`** 加 `created_at_ms` + `updated_at_ms`（domain 已有这俩字段，DTO 没暴露）
- **`list_conversations` 命令**：改为只返回根会话（`source_parent_id IS NULL`），新参数 `parentId: Option<String>` 用于查某主任务的子任务
- **新增 `list_child_conversations` 命令**：查指定父会话的子任务
- **新增 `reset_all_data` 命令**：调 `repo.clear_all()` + 清空 Tantivy 索引 + 清空 raw_store
- **`DaemonState`（`crates/daemon/src/state.rs`）** 加 `wipe_all()` 方法
- `import_raw_to_state`：在 `normalize(raw)` 前捕获 `source_parent_id`，赋值到 conv
- 注册新命令到 `invoke_handler`

### 3.2 `crates/search/src/index.rs`
- 新增 `clear_all()` 方法（删除所有 Tantivy 文档）

### 3.3 `crates/raw-store/src/lib.rs`
- 新增 `clear()` 方法（清空 blob 目录）

---

## 阶段 4：前端 — 异步加载 + Loading 图标

### 4.1 `apps/desktop/src/App.tsx`
- 新增 `loading` state（初始 true，loadWorkspaces 完成后 false）
- mount `useEffect` 改为：先 `setLoading(true)` → `loadWorkspaces()` → `setLoading(false)` → `autoSync()`（异步，不阻塞首屏）
- 新增 10 分钟定时 `useEffect`：`setInterval(() => autoSync(true), 600000)`，cleanup 清 interval
- `autoSync(silent?: boolean)` 加可选参数：silent=true 时不显示 loading（后台同步只更新 syncResult）
- 渲染：`loading` 时显示旋转 spinner 覆盖层，数据加载完后正常显示

### 4.2 `apps/desktop/src/styles.css`
- `.loading-overlay`：全屏半透明白底 + 居中 spinner 动画
- `@keyframes spin` 旋转动画

---

## 阶段 5：前端 — 主子任务折叠 + 时间格式 + 标签布局

### 5.1 `apps/desktop/src/App.tsx`
- **`Conversation` 接口**加 `source_parent_id: string | null` + `child_count: number`
- **`Workspace` 接口**加 `created_at_ms: number | null` + `updated_at_ms: number | null`
- **`formatTime`** 改为 `YYYY-MM-DD HH:MM:SS` 全格式（手动拼接，确保用 `-` 不用 `/`）
- **中栏会话列表**：
  - 只渲染根会话（`source_parent_id == null`）
  - 每个根会话有展开/折叠按钮（`child_count > 0` 时显示 `▶/▼`）
  - 展开时调 `list_child_conversations` 拉子任务，缩进显示
  - 新增 `expandedParents: Set<string>` state 管理展开
- **来源标签移到 meta 行**：`.title` 只放标题，`.meta` 行放 `[badge] [时间] [model]`，flex 布局
- **左栏 Workspaces**：显示 `created_at_ms`（创建）和 `updated_at_ms`（更新）两行时间
- **左栏 Workspaces 来源标签**：workspace 无 provider 字段，改为显示该 workspace 下会话最多的 provider（前端从 conversations 统计），放 meta 行

### 5.2 `apps/desktop/src/styles.css`
- `.list-item .meta` 改为 `display: flex; align-items: center; gap: 6px; flex-wrap: wrap;`
- `.child-item`：缩进样式（margin-left + 更小字体 + 灰色）
- `.expand-toggle`：展开按钮样式
- `.workspace-time`：workspace 时间行样式

---

## 阶段 6：验证

- `cargo build --workspace` + `cargo clippy --workspace --all-targets`（0 warnings）
- `cargo test --workspace`（308+ 测试全过，新增 clear_all / list_child_conversations 测试）
- `npx tsc --noEmit` + `npx vite build`（前端无错）
- 真实数据验证：启动后左栏只有主任务、中栏可展开子任务、时间显示完整

---

## 关键决策记录
- **子任务是独立 Conversation**，通过 `source_parent_id` 关联父会话的 `source_conversation_id`（不是 ID 外键，因为导入时父 ID 是动态生成的，但 source_id 是稳定的）
- **child_count 不入库**，DTO 转换时实时 COUNT 查询（避免数据不一致）
- **clear_all 用 DELETE 不用 DROP**（保留 schema，V5 迁移不丢）
- **auto_sync 幂等**已有（按 source_conversation_id + provider），子任务也走同样逻辑
- **10 分钟同步**复用 auto_sync 的幂等机制，天然增量
