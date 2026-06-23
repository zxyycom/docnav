本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并建立标准参数映射机制，让 core `docnav` 和 adapter SDK direct CLI 都能用同一配置/argv 参数模型表达是否启用分页和字符预算；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 design，不影响现有其它文档或主规范。

## Context

分页位置 `page` 固定从 `1` 开始，invoke request 必须显式包含正整数 `limit_chars` 和 `page`，adapter 在生成 outline/read/find 结果时按字符预算分页并返回下一页 `page`。本 change 将分页是否启用和分页预算建模为同一个标准参数对象：`defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。

用户需要配置文件和显式 argv 都能表达相同的 pagination 参数来源，同时不改变 protocol shape、readable output shape 或 adapter 内部分页 helper 的核心行为。

后续标准配置项会继续增加。这个 change 将 pagination 作为首个完整样例，引入标准参数定义机制，由同一个 owner 定义驱动 CLI parser、配置投影、help、supported keys 和最终归一逻辑。

## Goals / Non-Goals

**Goals:**

- 为 core `docnav` 配置域和 adapter SDK direct CLI 配置域统一引入 `defaults.pagination.enabled` 与 `defaults.pagination.limit_chars`。
- 建立标准参数定义机制，让配置路径和 CLI flag 都投影到同一个标准参数来源模型。
- 为 core `docnav` document commands 和 adapter SDK direct CLI document commands 统一引入 `--pagination enabled|disabled`。
- 在分页前初始化最终 `limit_chars`：`enabled: true` 时使用配置预算，`enabled: false` 时使用协议 `PositiveInteger` 可表示的最大值。
- 保持 invoke protocol 仍只接收显式正整数 `limit_chars` 和 `page`，不新增 `pagination` 字段。
- 保持 `page` 不可配置，省略时仍固定为 `1`。
- 让 SDK direct CLI 用户，例如 `docnav-markdown`，通过相同配置对象获得同样的“不启用分页”行为。

**Non-Goals:**

- 不改变 `--limit-chars` 的预算含义；它只提供 `defaults.pagination.limit_chars` 的显式来源，最终是否使用该预算由解析后的 `pagination.enabled` 决定。
- 不把 adapter native options 全部提升为 core 标准参数；native option 仍由 adapter/SDK 的 options 注册与适用性处理拥有。
- 不改变 adapter `invoke` stdin JSON strict validation；invoke 不读取配置、不补默认值。
- 不改变 `ReadResult`、`OutlineResult`、`FindResult` 的 `page` 字段语义。
- 不把分页逻辑移动到 readable renderer 或 MCP bridge。

## Decisions

1. 使用 `defaults.pagination` 作为唯一分页默认值对象。

   `defaults.pagination` 是分页默认值的 owner，包含 `enabled` 和 `limit_chars`。配置、argv、context 输出和 tests 都围绕这个对象表达分页状态和预算，避免同一行为分散到多个默认值字段。

2. `enabled: false` 映射为协议正整数域的最大预算，而不是改变协议字段。

   Protocol request 当前要求 `limit_chars` 是正整数，adapter paging helper 也围绕正整数预算实现。禁用分页时在 core/SDK direct CLI 参数归一阶段把最终 `limit_chars` 设为 `PositiveInteger` 可表示的最大值，可以保持 invoke、adapter handler、paging helper、protocol schema 和 readable 输出不变。

   备选方案是在 protocol arguments 中新增 `pagination.enabled`。该方案会改变 machine-readable contract，并要求所有 adapter 和 schema 同步迁移，复杂度和风险明显更高。

3. 使用 `--pagination enabled|disabled` 作为显式 CLI 参数来源。

   Pagination enabled 是二值状态，显式 argv 必须能提供和配置文件同名同义的参数来源。`--pagination enabled|disabled` 映射到 `defaults.pagination.enabled` 的 explicit source；`--limit-chars <n>` 映射到 `defaults.pagination.limit_chars` 的 explicit source。两个 flag 的映射都由对应标准参数定义声明。

4. 标准参数定义是配置路径和 CLI flag 的单一映射来源。

   每个标准参数定义必须声明 canonical path，例如 `defaults.pagination.enabled`，以及 value kind/validator、argv flag 和 parser、config projection path、operation applicability、default provider、source priority、help/context 展示元数据和 finalization hook。配置文件和显式 argv 只产生不同来源的同一个标准参数值；最终 operation 参数只能从标准参数归一结果生成。

   Core `docnav` 拥有 core 标准参数定义，例如 `defaults.adapter`、`defaults.pagination.enabled`、`defaults.pagination.limit_chars` 和 `defaults.output`。`docnav-adapter-sdk` 拥有 direct CLI 标准参数定义，例如 `defaults.pagination.enabled`、`defaults.pagination.limit_chars` 和 `defaults.output`。两层使用相同 canonical path 和优先级语义，但实现 owner 分开，避免 SDK adapter native options 泄漏到 core。

   后续新增标准参数时，必须新增或更新对应参数定义，并通过同一机制驱动 argv parsing、config projection、supported key listing、help/default 文案、context 输出和 typed validation。同一标准参数的各个 surface 共享同一个定义 owner。

5. `--pagination` 和 `--limit-chars` 都进入同一个标准参数来源模型。

   `--pagination enabled|disabled` 映射为 `defaults.pagination.enabled` 的显式来源，`--limit-chars <n>` 映射为 `defaults.pagination.limit_chars` 的显式来源。两者和配置文件字段共享同一归一链路：先按字段分别解析来源优先级，再根据最终 `pagination.enabled` 初始化最终 `limit_chars`。

   因此 `--pagination disabled --limit-chars <n>` 是合法输入：最终 `pagination.enabled=false`，`pagination.limit_chars=<n>` 作为已解析预算保留，但最终发送给 adapter 的 `limit_chars` 仍归一为 `PositiveInteger` 最大值。若项目配置关闭分页，调用方需要使用 `--pagination enabled --limit-chars <n>` 才能让本次调用按 `<n>` 分页。

6. SDK direct CLI 配置读取层只投影标准参数来源，最终类型校验留给标准参数处理链路。

   配置读取层继续只负责 JSON 读取、标准参数定义驱动的字段投影、来源优先级和 warning。`pagination.limit_chars` 的正整数校验、`pagination.enabled` 的布尔校验和最终 `limit_chars` 显式化应在标准 direct CLI 参数处理链路完成，保持 config loading 和 operation semantics 分层。

7. `adapter invoke` 不读取新配置。

   invoke stdin JSON 是严格 protocol input；调用方必须在进入 invoke 前完成默认值解析，并显式写入 `limit_chars`。这同时保证 core `docnav`、SDK direct CLI 和未来其它调用方可以共享同一 protocol contract，而不会让 adapter 进程隐式读取环境配置。

## Risks / Trade-offs

- **“不启用分页”不是数学无限** -> 使用协议 `PositiveInteger` 最大值作为内部预算；文档和测试应表述为“不启用分页的对外语义”，内部仍是最大字符预算。
- **参数来源与最终预算需要区分** -> 当最终 `pagination.enabled=false` 时，`pagination.limit_chars` 已解析但不用于最终 adapter 预算；help、文档和测试必须说明需要 `--pagination enabled --limit-chars <n>` 才能临时恢复分页并使用该预算。
- **超大文档可能一次性输出过大** -> 这是禁用分页的明确后果；默认仍为 `enabled: true` 和 `limit_chars: 6000`，只有用户配置关闭时触发。
- **core 和 SDK 行为漂移** -> 通过共享命名、相同优先级规则和跨 core/SDK/Markdown smoke 测试覆盖。
- **标准参数定义与各 surface 漂移** -> argv parser、config projection、help、supported keys、context 输出和 typed validation 都必须消费同一 owner 下的参数定义；测试覆盖新增参数只需补定义即可暴露到目标 surface。
- **schema 与 runtime 分层混淆** -> schema/example 只作为参考和验证材料，runtime 配置读取仍不依赖 schema 加载。

## Migration Plan

1. 更新 OpenSpec delta、主规范、schema 和 examples，统一使用 `defaults.pagination` 作为分页默认值对象。
2. 在 core 和 SDK direct CLI 中建立标准参数定义机制，并把 pagination 参数作为首批定义。
3. 更新 core 配置模型、document command parser、supported keys、get/set/unset/list、document context 输出和默认值解析，使这些 surface 消费 core 标准参数定义。
4. 更新 SDK direct CLI argv parser、config source model、definition-driven field projection、merge logic、typed validation 和 help/default 文案，使这些 surface 消费 SDK direct CLI 标准参数定义。
5. 更新 Markdown adapter config schema/example 和 smoke/matrix/Rust 测试。
6. 验证 protocol request/result schema 不新增字段，adapter invoke strict path 不读取配置。
7. 运行范围测试；跨 core、SDK、schema/example 和 Markdown smoke 时优先运行 workspace verifier。
