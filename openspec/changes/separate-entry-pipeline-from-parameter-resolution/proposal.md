本 proposal 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是在标准入口管线与参数来源解析之间建立清晰边界；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## Why

现有“标准参数”命名容易被理解成所有对外内容的统一入口，导致 help、manifest、probe、config、adapter management、transport decode 和 document operation 参数解析的 owner 边界混淆。现在需要把入口生命周期单独建模为标准入口管线，并把现有标准参数解析收缩为入口参数来源解析：它只读取入口提供的 input view 和配置来源，产出派生值，不改写原始 argv、stdin JSON 或 protocol request。

## What Changes

- 引入标准入口管线：入口先完成命令族、transport 和 output intent 分类，再决定是否读取配置、是否进入 document semantic request、如何映射错误和输出。
- 将当前“标准参数解析”改名并收缩为“入口参数来源解析”：只接收入口 owner 提供的 direct input view、配置来源和默认值，产出 typed runtime values、source info、diagnostic handoff 和 passthrough handoff。
- adapter native options 必须作为 adapter owner 明确声明的 native option source descriptors 进入入口参数来源解析；未映射到标准参数、注册 config path 或 owner-declared native option source 的 public input 必须回到入口 owner 边界处理为 blocking input diagnostic。
- 将“配置来源合并通道”定义为入口参数来源解析的子流程：它只负责 project/user config source 读取、source issue handoff、字段投影和来源参与。
- 明确不可变输入规则：参数来源解析不得修改原始 CLI argv tokens、adapter `invoke` stdin JSON、protocol request envelope 或 request `arguments`；后续 request construction 只能消费 derived semantic values 和 owner 明确保留的 passthrough。
- 调整 core CLI、adapter SDK 和 adapter invoke 的命名、文档与测试语义，使“标准入口管线”和“入口参数来源解析”分别对应生命周期 owner 与参数来源 owner。
- 非目标：本 change 不新增文档 navigation operation，不改变 protocol result shape，不改变 adapter ref 策略，不把 manifest/probe/help 纳入 document output mode。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `core-cli`: 定义 core 标准入口管线，明确 document、config/init/doctor/version/help 的入口分类、配置读取和参数来源解析边界。
- `adapter-protocol`: 定义 adapter SDK direct CLI、manifest/probe/help 和 `invoke` 的标准入口管线边界，明确 `invoke` strict direct input 与不可变原始 request 规则。
- `standard-parameter-resolution`: 将能力语义从“标准参数解析”改为“入口参数来源解析”，保留字段身份、来源优先级、typed validation、explicit adapter native option sources、passthrough handoff 和 operation argument binding，并限制其只处理入口提供的 input view 与来源对象。
- `standard-parameter-adoption`: 更新迁移约束，要求 core CLI 和 adapter SDK 消费重命名后的入口参数来源解析结果，同时保持入口 owner 边界和原始输入不可变。
- `typed-field-definitions`: 澄清 typed-field 是字段事实源和 input-view validation metadata，不是标准入口生命周期 owner，也不得承担原始输入改写。

## Impact

- Affected docs/specs: `docs/architecture.md`, `docs/cli.md`, `docs/standard-parameters.md` or its renamed successor, `docs/adapter-contract.md`, `docs/protocol.md`, `docs/testing.md`, and related OpenSpec specs.
- Affected Rust crates: `docnav`, `docnav-adapter-sdk`, `docnav-standard-parameters` or renamed successor crate/module, `docnav-typed-fields`, and tests that assert standard parameter names or source-resolution behavior.
- Affected public surfaces: CLI help wording, docs terminology, input/error ownership descriptions, schema/example validation references, and adapter SDK contract documentation.
- Migration expectation: externally observable names in docs, test fixtures, diagnostics, crate/module paths, or generated artifacts must be updated with their validation material in the same work item.
