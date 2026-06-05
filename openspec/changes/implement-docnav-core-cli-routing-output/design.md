**一句话核心：核心 CLI 负责把用户意图解析为显式 invoke 请求，并把 adapter 协议结果映射回阅读输出。**

## Context

`docnav` 是所有接入方式共享的核心契约。它不解析格式内容，但负责项目根、path、配置、adapter 选择、默认参数、invoke 进程、协议校验、输出层和错误映射。

## Goals / Non-Goals

**Goals:**

- 实现 `docnav outline/read/find/info` 文档操作。
- 实现 `init`、`doctor`、`version` 和 `config get|set|unset|list` 的核心 CLI 基础。
- 实现显式 format、扩展名、全量 probe 的分阶段 adapter 选择。
- 实现 text、readable-json 和 protocol-json 输出。
- 保证 `outline -> ref -> read` 端到端链路可运行。

**Non-Goals:**

- 不实现 Markdown parser。
- 不实现正式 adapter install/update/remove/list 的完整算法。
- 不实现 MCP transport。

## Decisions

1. CLI 在启动 invoke 前完成全部默认参数解析。
   - page 省略时显式传入 `page: 1`。
   - limit_chars 从显式参数、项目配置、用户配置和内置默认值解析为有限正整数。

2. path 规范化只产生项目相对路径。
   - adapter 子进程 cwd 设置为项目根。
   - `document.path` 使用 `/`，并必须位于项目根内。

3. adapter 选择严格按三阶段执行。
   - 显式 format/content type 只产生候选，不可跳过 probe/校验。
   - 扩展名候选失败后进入全量 probe。
   - 等价多成功返回 `FORMAT_AMBIGUOUS`，全失败返回 `FORMAT_UNKNOWN`。

4. 输出映射分层处理。
   - `protocol-json` 输出完整原始协议 envelope。
   - text/readable-json 输出阅读层结果，不包含 envelope。
   - read 的 readable 输出保留 `content_type`。

5. 错误映射保持 code 稳定、展示可配置。
   - 稳定错误 code 不受文本模板配置影响。
   - CLI 退出码按主规范映射。

## Risks / Trade-offs

- [adapter 选择流程较复杂] → 用阶段性测试覆盖显式格式、content type、扩展名、全量 probe 和歧义。
- [阅读输出与 protocol 输出漂移] → 对同一 fixture 同时验证 protocol-json 和 readable-json 业务语义一致。
- [配置影响稳定字段] → 配置只允许影响文本模板、guidance、usage 和错误建议。

## Migration Plan

1. 在协议/SDK和 Markdown adapter 完成后实现核心 CLI。
2. 先支持手工注册或测试 registry，使路由链路可测。
3. adapter 管理 change 完成后切换到正式安装记录读取。

## Open Questions

- 正式 adapter registry 的存储细节由 adapter 管理 change 完成；本 change 只需要定义可被替换的读取接口。
