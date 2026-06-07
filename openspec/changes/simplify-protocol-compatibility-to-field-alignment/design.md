## Context

Docnav 当前同时拥有稳定协议层和协议版本区间管理。稳定协议层仍然必要，因为 `docnav`、adapter、MCP bridge 和测试脚本通过进程边界通信；但 `protocol.min/max`、闭区间版本协商和 `PROTOCOL_INCOMPATIBLE` 在 v0 阶段让 adapter 选择和错误映射变得过重。

本 change 采用当前契约硬校验：`docnav` 只接受当前 schema；adapter 的 manifest、probe 和 invoke 响应必须通过当前 schema、必需字段、字段类型、operation/result shape 和语义校验，任一环节不一致时当前阶段失败。

manifest 也需要收敛职责。它只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities，不声明协议范围，也不提供默认参数。`recommended_parameters` 是过期字段，不能再作为 `docnav` 默认值或格式 options 的来源。invoke 请求仍保留可选 `arguments.options` 作为 adapter 拥有的 opaque 参数通道；`docnav` 只处理 core 通用参数，不解析或合成格式 options。

## Goals / Non-Goals

**Goals:**

- 保留原始协议层和 `protocol-json` envelope。
- 保留 `protocol_version: "0.1"`、`manifest_version`、`probe_version` 作为固定 schema 识别字段。
- 移除 `protocol.min/max` 版本区间协商和最高兼容版本选择。
- 将 adapter 可用性统一为当前 schema 和语义硬校验。
- 将 manifest 职责收敛为 adapter 能力声明。
- 将 manifest `recommended_parameters` 视为过期字段并移除；格式专属默认值留在 adapter 直接 CLI 和 adapter 配置域内。
- 移除 `PROTOCOL_INCOMPATIBLE` 在路由、安装、更新和 SDK 兼容判断中的使用。

**Non-Goals:**

- 不移除 `protocol_version`、`manifest_version` 或 `probe_version` 字段。
- 不移除 invoke 请求中的可选 `arguments.options` 字段。
- 不改变 `request_id`、operation、result、page、ref、content_type 或 stable error 的基本 envelope 结构。
- 不改变格式 adapter 的解析、ref 或分页策略。
- 不引入多版本 adapter registry。
- 不让 `docnav` 从 manifest、配置或隐式默认值解析或生成格式专属 options。

## Decisions

1. 保留协议层，简化兼容判断。
   - `protocol_version` 固定为当前 schema 识别字段。
   - adapter 选择不协商版本；只校验当前 schema、字段 shape 和语义。
   - `docnav` 只接受当前契约。manifest/probe/invoke 输出不符合当前 schema 或语义校验时，当前阶段失败并记录可定位原因。

2. manifest 只声明 adapter 能力身份。
   - manifest 契约不包含协议范围字段 `protocol.min/max`。
   - manifest 契约不包含 `recommended_parameters`。
   - manifest 只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities。
   - `manifest_version` 继续表达 manifest schema 版本。

3. core 参数和格式参数分属不同配置域。
   - `docnav` 只处理 path、ref、query、page、limit_chars、output 和 adapter 等 core 通用参数。
   - adapter 直接 CLI 和 adapter 配置域负责格式专属默认值，例如 Markdown 的 `max_heading_level`。
   - invoke 请求中的 page、limit_chars、ref 和 query 来自调用方显式参数或 core 默认解析结果。
   - invoke 请求中的 `arguments.options` 仍是 adapter-owned opaque 对象；adapter 直接 CLI 可以由 adapter 自有 flag 生成它，core `docnav` 不从 manifest、配置或 `recommended_parameters` 合成它。

4. `PROTOCOL_INCOMPATIBLE` 退出稳定错误集合。
   - 版本区间无交集不再是产品语义。
   - 调用方请求字段错误映射为 `INVALID_REQUEST`；adapter manifest/probe/invoke 输出错误按阶段映射为 `ADAPTER_UNAVAILABLE`、`ADAPTER_INVOKE_FAILED` 或候选失败证据。
   - error rules、schema、examples 和 Rust enum 同步删除该错误码。

5. SDK 去掉版本协商 API。
   - 移除 `ProtocolRange`、`select_highest_compatible` 等用于闭区间协商的公共 API。
   - 保留当前 schema 校验函数和 request context 提取能力。
   - invoke 请求的 `protocol_version` 字段仍按当前 schema 校验。
   - 请求无法解析或无法提取版本字段时，SDK failure envelope 使用当前 `PROTOCOL_VERSION` 常量填充 `protocol_version`。

6. Markdown adapter 只更新 manifest shape。
   - Markdown adapter 不再在 manifest 中声明 `protocol.min/max` 或 `recommended_parameters`。
   - Markdown parser、probe、outline/read/find/info、pagination 和 ref 不变化。

## Risks / Trade-offs

- [未来 breaking change 缺少平滑共存机制] → v0 明确不做兼容或迁移；未来出现真实多版本需求时作为新问题处理。
- [删除错误码影响示例和生成物] → 同步更新 error-rules、schema、examples、Rust enum 和验证脚本，并运行生成/验证命令。
- [硬校验错误信息可能不如版本错误直观] → 在候选失败证据或稳定错误 details 中记录 stage、code、reason 和 schema path，便于定位缺字段、类型错误或 shape mismatch。
- [移除过期 `recommended_parameters` 后默认值来源减少] → 通过 adapter 自身配置域和直接 CLI 默认值覆盖格式专属参数；core CLI 只解析 core 通用参数，且不合成 `arguments.options`。
- [现有实现依赖 ProtocolRange 类型] → 通过任务显式覆盖 SDK、Markdown adapter 和 tests 的编译修复。
