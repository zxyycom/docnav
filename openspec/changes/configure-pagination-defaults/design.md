本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并让 core `docnav` 和 adapter SDK direct CLI 使用同一 pagination 配置/argv 行为；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 design，不影响现有其它文档或主规范。

## Context

分页位置 `page` 固定从 `1` 开始，invoke request 必须显式包含正整数 `limit_chars` 和 `page`，adapter 在生成 outline/read/find 结果时按字符预算分页并返回下一页 `page`。本 change 将分页是否启用和分页预算建模为同一个配置对象：`defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。

## Goals / Non-Goals

**Goals:**

- 为 core `docnav` 配置域和 adapter SDK direct CLI 配置域统一引入 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。
- 让配置路径和 CLI flag 投影到同一个 pagination 参数来源模型。
- 为 core `docnav` document commands 和 adapter SDK direct CLI document commands 统一引入 `--pagination enabled|disabled`。
- 在分页前初始化最终 `limit_chars`：`enabled: true` 时使用配置预算，`enabled: false` 时使用协议 `PositiveInteger` 可表示的最大值。
- 保持 invoke protocol 仍只接收显式正整数 `limit_chars` 和 `page`，不新增 `pagination` 字段。
- 保持 `page` 不可配置，省略时仍固定为 `1`。
- 让 SDK direct CLI 用户，例如 `docnav-markdown`，通过相同配置对象获得同样的“不启用分页”行为。

**Non-Goals:**

- 不改变 `--limit-chars` 的预算含义；它只提供 `defaults.pagination.limit_chars` 的显式来源，最终是否使用该预算由解析后的 `pagination.enabled` 决定。
- 不把 adapter native options 提升为 core pagination 配置；native option 仍由 adapter/SDK 的 options 注册与适用性处理拥有。
- 不改变 adapter `invoke` stdin JSON strict validation；invoke 不读取配置、不补默认值。
- 不改变 `ReadResult`、`OutlineResult`、`FindResult` 的 `page` 字段语义。
- 不把分页逻辑移动到 readable renderer 或 MCP bridge。

## Decisions

1. 使用 `defaults.pagination` 作为唯一分页默认值对象。

   `defaults.pagination` 是分页默认值的 owner，包含 `enabled` 和 `limit_chars`。配置、argv、context 输出和 tests 都围绕这个对象表达分页状态和预算，避免同一行为分散到多个默认值字段。

2. `enabled: false` 映射为协议正整数域的最大预算，而不是改变协议字段。

   Protocol request 当前要求 `limit_chars` 是正整数，adapter paging helper 也围绕正整数预算实现。禁用分页时在 core/SDK direct CLI 参数归一阶段把最终 `limit_chars` 设为 `PositiveInteger` 可表示的最大值，可以保持 invoke、adapter handler、paging helper、protocol schema 和 readable 输出不变。

   备选方案是在 protocol arguments 中新增 `pagination.enabled`。该方案会改变 machine-readable contract，并要求所有 adapter 和 schema 同步迁移，复杂度和风险明显更高。

3. Core 和 SDK 都使用同名 pagination canonical key。

   Core `docnav` 和 `docnav-adapter-sdk` direct CLI 都必须支持 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。两边使用相同 canonical key、config path、flag semantics、value validation、source priority 和 disabled finalization 语义；实现 owner 可以分开，但行为不能漂移。

4. 使用 `--pagination enabled|disabled` 作为显式 CLI 参数来源。

   Pagination enabled 是二值状态，显式 argv 必须能提供和配置文件同名同义的参数来源。`--pagination enabled|disabled` 映射到 `defaults.pagination.enabled` 的 explicit source；`--limit-chars <n>` 映射到 `defaults.pagination.limit_chars` 的 explicit source。两个 flag 都进入同一 pagination 参数来源模型。

5. `--pagination` 和 `--limit-chars` 都进入同一个 pagination 参数来源模型。

   两者和配置文件字段共享同一归一链路：先按字段分别解析来源优先级，再根据最终 `pagination.enabled` 初始化最终 `limit_chars`。

   因此 `--pagination disabled --limit-chars <n>` 是合法输入：最终 `pagination.enabled=false`，`pagination.limit_chars=<n>` 作为已解析预算保留，但最终发送给 adapter 的 `limit_chars` 仍归一为 `PositiveInteger` 最大值。若项目配置关闭分页，调用方需要使用 `--pagination enabled --limit-chars <n>` 才能让本次调用按 `<n>` 分页。

6. SDK direct CLI 配置读取层只投影 pagination 参数来源，最终类型校验留给 direct CLI 参数处理链路。

   配置读取层继续只负责 JSON 读取、字段投影、来源优先级和 warning。`pagination.limit_chars` 的正整数校验、`pagination.enabled` 的布尔/枚举校验和最终 `limit_chars` 显式化应在 direct CLI 参数处理链路完成，保持 config loading 和 operation semantics 分层。

7. `adapter invoke` 不读取新配置。

   invoke stdin JSON 是严格 protocol input；调用方必须在进入 invoke 前完成默认值解析，并显式写入 `limit_chars`。这同时保证 core `docnav`、SDK direct CLI 和未来其它调用方可以共享同一 protocol contract，而不会让 adapter 进程隐式读取环境配置。

## Risks / Trade-offs

- **“不启用分页”不是数学无限** -> 使用协议 `PositiveInteger` 最大值作为内部预算；文档和测试应表述为“不启用分页的对外语义”，内部仍是最大字符预算。
- **参数来源与最终预算需要区分** -> 当最终 `pagination.enabled=false` 时，`pagination.limit_chars` 已解析但不用于最终 adapter 预算；help、文档和测试必须说明需要 `--pagination enabled --limit-chars <n>` 才能临时恢复分页并使用该预算。
- **超大文档可能一次性输出过大** -> 这是禁用分页的明确后果；默认仍为 `enabled: true` 和 `limit_chars: 6000`，只有用户配置关闭时触发。
- **core 和 SDK 行为漂移** -> 通过同名 canonical key、相同优先级规则和跨 core/SDK/Markdown smoke 测试覆盖。
- **schema 与 runtime 分层混淆** -> schema/example 只作为参考和验证材料，runtime 配置读取仍不依赖 schema 加载。

## Migration Plan

1. 更新 OpenSpec delta、主规范、schema 和 examples，统一使用 `defaults.pagination` 作为分页默认值对象。
2. 更新 core 配置模型、document command parser、supported keys、get/set/unset/list、document context 输出和默认值解析，使这些 surface 支持 pagination 参数来源。
3. 更新 SDK direct CLI argv parser、config source model、field projection、merge logic、typed validation 和 help/default 文案，使这些 surface 支持 SDK direct CLI pagination 参数来源。
4. 更新 Markdown adapter config schema/example 和 smoke/matrix/Rust 测试。
5. 验证 protocol request/result schema 不新增字段，adapter invoke strict path 不读取配置。
6. 运行范围测试；跨 core、SDK、schema/example 和 Markdown smoke 时优先运行 workspace verifier。
