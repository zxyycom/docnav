本 design 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是在标准入口管线与参数来源解析之间建立清晰实现边界；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## Context

当前文档和实现已经把 document operations、help、manifest、probe、config/init/doctor/version、adapter management 和 adapter `invoke` 分成不同入口，但“标准参数”这个名称容易被理解为所有对外内容都必须先进入的统一入口。实际实现中，core document command、adapter direct document command 和 adapter `invoke` 会消费同一类来源解析结果；help、manifest、probe 和 non-document commands 不读取 document operation 配置，也不进入 document output mode。

本 change 使用以下术语分层：

- 标准入口管线：入口生命周期 owner，负责 command/transport 分类、配置读取决策、handler dispatch 和 output/error projection。
- 入口参数来源解析：参数来源 owner，负责 direct input view、project/user config source 和 defaults 的合并、校验与 handoff。
- 配置来源合并通道：入口参数来源解析中的 config 子流程，只负责 project/user config source。
- 标准参数身份：跨 CLI/config/invoke 复用的参数 identity，不代表入口生命周期。

## Goals / Non-Goals

**Goals:**

- 定义标准入口管线：command/transport 分类、output intent 捕获、配置读取决策、参数来源解析调用、semantic request 构造、handler dispatch、error/output projection。
- 将现有标准参数解析重命名和收缩为入口参数来源解析：只处理入口 owner 提供的 direct input view、project/user config source 和 defaults。
- 明确不可变输入规则：解析过程不得改写原始 argv tokens、stdin JSON、protocol envelope 或 raw `arguments`。
- 保留标准参数 identity 概念，并把它从入口生命周期 owner 中拆出来。
- 为 docs、OpenSpec specs、Rust crate/module/type 命名和测试断言提供迁移路径。

**Non-Goals:**

- 不改变 `outline -> ref -> read` 导航模型。
- 不改变 protocol response/result shape、readable output shape 或 adapter ref ownership。
- 不把 help、version、manifest、probe、adapter management 或 config 命令纳入 document output mode。
- 不让 `docnav` core 解释 adapter-owned native options。

## Decisions

### Decision 1: 标准入口管线拥有生命周期，参数来源解析只拥有来源合并与校验

决定把入口生命周期命名为“标准入口管线”。该管线由 core CLI 或 adapter SDK owner 负责，先分类入口，再决定是否调用参数来源解析。影响：help、manifest、probe 和 non-document commands 可以保留各自 output/error owner。

Alternative considered: 把现有“标准参数流程”提升为所有入口的统一入口。该方案会让 help、manifest/probe、config/init/doctor/version 误读配置和 document output mode，破坏现有 owner 边界。

### Decision 2: 使用“入口参数来源解析”作为当前标准参数解析的新语义名

决定把当前 resolver 的语义描述改为“入口参数来源解析”。它覆盖 direct input、project config、user config 和 default 的合并、typed validation、source info、diagnostic handoff 和 passthrough handoff。影响：配置合并只能作为 config source 子流程出现，不能代表完整 resolver。

Alternative considered: 使用“配置来源合并通道”作为总名。该名称能表达用户关注的配置部分，但会遗漏 direct input、default、requiredness、typed validation 和 operation argument binding，因此只作为子流程名称使用。

### Decision 3: 原始输入保持不可变，解析结果是 derived values

决定要求参数来源解析只读取入口 owner 构造的 input view 或 loaded config source，不得修改原始 argv、stdin JSON、protocol envelope 或 `arguments`。后续 request construction 只能消费 typed runtime values、source info 和 owner 明确保留的 passthrough。影响：config/default 补足只表现为派生值，不会被写回 raw input。

Alternative considered: 在 resolver 内直接补全 request JSON 或删除 unknown fields。该方案会混淆 raw input、semantic values 和 output/debug surfaces，也会让 config/default 值被误认为 direct input。

### Decision 4: 渐进式重命名，先保留兼容 shim

决定先在 docs/specs/tests 中建立新术语，再逐步重命名 Rust module/type/crate 或提供兼容 alias。公开 crate 名、diagnostic code、schema/example path 如受影响，需要明确兼容窗口或一次性 breaking 说明。

Alternative considered: 一次性重命名所有 crate、module、types 和 docs。该方案风险较高，容易造成 OpenSpec、schema、test fixture 和 release artifact 同步失败。

## Risks / Trade-offs

- [Risk] 新旧术语并存会短期增加阅读成本。→ Mitigation: 文档中建立术语映射表，并在 migration tasks 中逐项删除旧名误导用法。
- [Risk] 只改名不改边界会留下误解。→ Mitigation: specs 明确标准入口管线和不可变输入规则，测试覆盖 help/manifest/probe/config 不进入 document parameter resolution。
- [Risk] crate/module 重命名可能造成大范围 diff。→ Mitigation: 先以 module/type alias 或 wrapper 做行为迁移，再决定是否重命名 crate。
- [Risk] “配置来源合并通道”被误认为完整 resolver。→ Mitigation: 只把它定义为 project/user config source 子流程，不用于 direct input/default/operation binding。

## Migration Plan

1. 更新 docs 和 OpenSpec specs：引入“标准入口管线”“入口参数来源解析”“配置来源合并通道”“标准参数身份”四个术语，并标注旧“标准参数流程”的迁移含义。
2. 微调实现命名：在 core 和 adapter SDK 中把调用边界命名为 entry pipeline / parameter source resolution，保留必要兼容 alias。
3. 确认 resolver 不直接修改 raw argv/stdin/request：新增或调整测试，断言 config/default 补足只影响 derived operation values，不回写 raw protocol JSON。
4. 同步测试策略、case 文案和 schema/example validator 引用。
5. 运行 workspace 验证，审查 diff 只触及命名、边界和对应测试。

Rollback strategy: 若重命名实现导致范围过大，保留 Rust crate/module 旧名作为内部兼容层，但 docs/specs 和 public explanation 仍使用新边界术语；行为边界测试必须保留。

## Open Questions

无未回答开放问题，可以进入实现前审计。
