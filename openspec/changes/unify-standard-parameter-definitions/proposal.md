本 change 目标是统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制，让两边都使用 builder 风格的 Rust 参数定义对象驱动 CLI flag、help、配置路径、校验、来源合并和 schema metadata；本文档只是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 proposal，不影响现有其它文档或主规范。

## Why

Core CLI 和 adapter SDK direct CLI 现在各自拥有配置、argv、help 和类型校验链路；`defaults.output` 这类跨两边都存在的标准参数，容易出现命名、优先级、校验和展示行为漂移。

需要把标准参数定义机制作为独立 change 交付，统一 core 和 SDK 的行为边界；具体业务参数 change 只描述自己的参数行为，不顺带定义通用抽象。

## What Changes

- 新增共享的 Rust 标准参数定义模型：每个标准参数由一个 builder-style definition 声明 canonical key、配置文件路径、可选 CLI flag、help/default 文案、value kind、校验函数、operation applicability、source priority、finalization rule 和 schema metadata。
- Core `docnav` 和 `docnav-adapter-sdk` direct CLI 都必须使用这种定义模型表达自己拥有的标准参数；两边可以有不同 owner 集合，但同名 canonical key 必须使用一致语义。
- 标准参数定义必须能驱动 argv parsing、config projection、supported key listing、help/default 文案、document context 输出、typed validation、来源合并和最终 operation 参数生成。
- 标准参数定义模型应能导出配置 schema 所需的字段形状和约束元数据；schema 生成可以分阶段落地，但定义结构不能阻断后续 schema generation。
- Native adapter options 仍位于 adapter-owned `options` object 下；它们不提升为 core 标准参数，也不由 core 解释。
- 非目标：本 change 不引入新的业务配置键，不改变 protocol request/result shape、ref 或 readable output 语义，不要求一次性把所有旧配置迁移到新 key。

## Capabilities

### New Capabilities

- 无新增 capability ID。

### Modified Capabilities

- `core-cli`：修改 core 配置、document argv、help、context 输出和最终参数解析要求，使标准参数由共享定义模型驱动。
- `adapter-protocol`：修改 adapter SDK direct CLI 配置/argv/help/validation 边界，使 SDK direct CLI 使用同一标准参数定义模型，并保留 native options 与 invoke strict protocol 边界。

## Impact

- 影响 `crates/docnav` 中 core 标准参数注册、配置 key 管理、document argv 解析、help/default 文案、`config get/set/unset/list` 和 document context 输出。
- 影响 `crates/docnav-adapter-sdk` 中 direct CLI 标准参数注册、config projection、argv parsing、help/default 文案、typed validation、warning 与 operation 参数生成。
- 可能新增共享 Rust 模块或 crate，用于定义 builder-style parameter definition；具体归属由 design 确定。
- 影响 `docs/cli.md`、`docs/architecture.md`、`docs/adapter-contract.md` 和相关测试策略说明。
- 不影响 adapter invoke stdin JSON 协议；标准参数定义 metadata 不进入 protocol request arguments。
