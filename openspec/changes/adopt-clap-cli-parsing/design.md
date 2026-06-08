**一句话核心：本设计说明如何用 `clap` 替代手写 Rust CLI argv parser，并将直接 CLI 容错收敛为 AI 友好的宽松解析边界。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## Context

Docnav 是 CLI-first 的文档导航系统，核心读取链路是 `outline -> ref -> read`。Rust CLI 入口既服务人类，也服务 AI agent；本项目的主要开发维护由 AI 执行，人类主要审核方向和关键取舍。

当前 adapter 直接 CLI 在 SDK 中手写解析 argv，并长期约束 unknown flag、extra positional、unused operation flag 的 ignored token、kind、reason 和消费规则。该规则对兼容性很精细，但对不熟 Rust 的审核者和 AI 维护者成本偏高，也让未来核心 `docnav` CLI、adapter 直接 CLI 和管理命令难以统一成清晰声明式结构。

`clap` 是 Rust CLI 生态中热度最高、资料最多、声明能力最完整的参数解析库。它可以通过 derive 或 builder 描述子命令、参数、默认值、枚举值、help 和类型解析，并提供 `ignore_errors(true)` 等宽松解析能力。

## Goals / Non-Goals

**Goals:**

- 使用 `clap` 作为 Rust CLI 参数解析的首选基础，降低 AI 生成、审阅和维护 CLI 入口的复杂度。
- 将 CLI 参数定义集中到结构体、枚举或清晰的 builder 代码中，减少手写 token loop。
- 支持 AI 友好容错：未知或无关 argv 不应在其它必需参数正确时阻断成功执行。
- 保持真正必要的错误边界：必需参数缺失、已知使用参数非法、业务错误和协议错误仍明确失败。
- 保持 process boundary 清晰：CLI argv 容错不进入 adapter `invoke` JSON，protocol schema 和 readable schema 不因解析库变化而改变。
- 让 CLI `--help` 成为 AI 调用纠错和人类审核的稳定入口。

**Non-Goals:**

- 不改变 `outline -> ref -> read` 业务语义。
- 不改变 adapter 生成和解析 ref 的所有权。
- 不改变 protocol envelope、manifest、probe 或 readable JSON schema。
- 不把 Markdown 或其它格式解析逻辑移入核心 `docnav`。
- 不要求继续保留旧 direct CLI warning 的精确 ignored token 分组、kind 枚举和消费顺序。

## Decisions

1. **采用 `clap` 作为 Rust CLI 首选解析库。**

   - 决策：核心 `docnav` CLI、adapter 直接 CLI 和未来 Rust CLI 扩展优先使用 `clap`。
   - 理由：`clap` 的声明式结构、自动 help、子命令、类型解析、默认值和资料规模更适合 AI 主维护、人类方向审核的协作模型。
   - 替代方案：继续手写 parser 可保持最大控制力，但会扩大 Rust 细节审核成本；`lexopt` 更适合精确自定义 token 语义，但仍要求维护者读手写 match；`argh` 更轻，但复杂子命令、容错和生态资料不如 `clap`。

2. **直接 CLI 容错目标从精确兼容改为成功优先。**

   - 决策：unknown flag、多余 positional 和无关参数可以生成 warning 或诊断，但不再要求稳定的 ignored token shape；只要 path/ref/query 等必需语义足够执行，CLI 应继续执行。
   - 理由：用户明确将核心目标定义为“忽略未知错误，在其它参数正确时尽量一次成功，减少 AI 读取次数”。精确 warning 契约不是核心产品价值。
   - 替代方案：保留旧 warning schema 能提供强兼容自动化，但实现和审计成本高，并且会限制 `clap` 的自然用法。

3. **协议层继续严格，容错只作用于 CLI argv。**

   - 决策：adapter `invoke` stdin JSON 继续按 protocol schema 严格校验，未知字段、缺字段和类型错误不得因 `clap` 迁移而宽松通过。
   - 理由：CLI 阅读层服务 AI 和人类调用，protocol 层服务机器稳定接口；两层边界是 Docnav 的核心架构约束。
   - 替代方案：让 invoke 也容错会降低自动化校验和跨进程契约稳定性，不采用。

4. **优先使用 derive；动态 adapter native options 使用 builder 或受控桥接。**

   - 决策：固定命令和固定参数使用 `#[derive(Parser, Subcommand, Args)]`；adapter native options 这类由 adapter 声明的扩展参数可以使用 `clap::Command` builder 或先解析已知共享参数后在受控结构中处理。
   - 理由：derive 最易读；builder 能处理运行时扩展参数。两者都比散落的手写 token loop 更容易审计。
   - 替代方案：完全 derive 会让动态 native options 不自然；完全 builder 会降低字段级可读性。

5. **warning 输出从稳定 schema 退化为阅读层辅助信息。**

   - 决策：text/readable 输出可继续包含 warning，但测试只断言“存在用户可理解诊断且命令成功”，不再断言 precise `ignored_tokens`、`kind`、`reason`。
   - 理由：保留提示价值，同时释放 parser 实现空间。
   - 替代方案：删除 warning 会让用户难以发现被忽略输入；保留旧 shape 会阻碍迁移。

## Risks / Trade-offs

- **Risk: `clap` 默认严格解析导致未知参数仍阻断成功路径。** → Mitigation: 在设计和测试中明确宽松解析策略，使用 `ignore_errors(true)`、外部参数捕获或预处理，只把必需语义缺失作为阻断条件。
- **Risk: 旧 smoke 测试大量断言 warning token shape，迁移后测试失败。** → Mitigation: 先修改 spec 和测试目标，再实现 parser 迁移；测试改为验证成功路径、help 可用和必要错误边界。
- **Risk: derive 与动态 native options 混用后结构分散。** → Mitigation: 固定共享参数和子命令用 derive；动态 options 单独放在 adapter-owned bridge 中，并提供集中测试。
- **Risk: 容错过宽导致拼错已知参数被静默忽略。** → Mitigation: 对常见拼写错误和无关输入输出 warning；对已知使用参数的非法值继续失败；help 中暴露正确参数名。
- **Risk: 新依赖增加编译时间和依赖面。** → Mitigation: 只启用所需 feature，避免额外 completion/color/wrap 功能，除非后续明确需要。

## Migration Plan

1. 更新 OpenSpec 和主文档，先确立 `clap` 首选路径和宽松 CLI 解析目标。
2. 引入 `clap` 依赖，优先在 adapter direct CLI 或核心 CLI 中形成最小可运行解析结构。
3. 将现有手写解析测试改写为目标行为测试：必需参数正确时 unknown argv 不阻断，必需参数缺失和已知非法值仍失败。
4. 迁移 `docnav-markdown` 直接 CLI 参数入口，保留现有 operation request 构造和输出分流。
5. 更新 smoke runner 的 argument matrix，减少对 warning token shape 的断言。
6. 运行局部 Rust 测试、Markdown CLI smoke 和 workspace 验证。
7. 若迁移引入阻塞，可回滚到旧 parser，但保留新 spec 中的宽松目标，后续再用 builder 或预处理方式实现。

## Open Questions

- 是否保留 readable-json 顶层 `warnings` 字段作为非稳定阅读层字段，还是只保留 text/stderr warning？
- adapter native options 的最终形态是全部纳入 `clap::Command` builder，还是先解析共享参数再交给 adapter-owned option parser？
- 核心 `docnav` CLI 与 adapter 直接 CLI 是否共用同一套 `clap` 参数结构，还是只共享行为测试和输出契约？
