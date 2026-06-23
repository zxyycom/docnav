本 change 目标是统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制，让两边都使用 builder 风格的 Rust 参数定义对象驱动 CLI flag、help、配置路径、校验、来源合并和 schema metadata；本文档只是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 design，不影响现有其它文档或主规范。

## Context

Core `docnav` 现在拥有 core 配置域、document argv、`config get/set/unset/list` 和 document context 输出。`docnav-adapter-sdk` direct CLI 拥有 adapter 配置读取、direct CLI argv、help/default 文案、native options 和 warning。两边的职责边界不同，但 `defaults.output` 这类标准参数需要同一套 key、flag、help、校验和来源优先级语义。

仓库已有 `docnav-cli-args` 共享 crate，当前只承接 direct CLI loose argv helper。它是本 change 推荐的共享定义模型落点；如果实现中发现依赖方向不合适，可以使用等价共享 Rust 模块或新 crate，但 core 和 SDK 必须复用同一个定义模型。

## Goals / Non-Goals

**Goals:**

- 定义一个共享 Rust 标准参数 definition model，支持 builder-style 链式声明。
- 让 core `docnav` 和 `docnav-adapter-sdk` direct CLI 都用该模型注册标准参数。
- 让单个定义驱动 CLI flag、help/default 文案、配置文件 key、value kind、校验函数、operation applicability、source priority、finalization 和 schema metadata。
- 保留 core 和 SDK 各自的 owner：core 只注册 core-owned 标准参数；SDK 注册 direct CLI 标准参数；adapter native options 仍在 `options` object 下由 adapter/SDK native option 机制处理。
- 为后续从定义生成配置 schema 留出结构化 metadata。

**Non-Goals:**

- 不在本 change 中新增具体业务配置 key；业务参数变更由对应 change 自己声明。
- 不改变 `invoke` stdin JSON、protocol request/result schema、readable output 或 MCP bridge 映射。
- 不把 adapter native options 提升为 core 标准参数。
- 不要求 schema generation 一步完成所有历史 schema；本 change 只要求 definition model 携带可生成 schema 的元数据。

## Decisions

1. 使用 builder-style Rust definition 表达标准参数。

   标准参数定义是静态 metadata 和行为 hook，不是本次调用的参数值。目标形态类似：

   ```rust
   let params = standard_params! {
       output: StandardParam::new("defaults.output")
           .config_file("defaults.output")
           .flag("--output")
           .help("Select the document output mode")
           .value_kind(ValueKind::Enum(&["readable-view", "readable-json", "protocol-json"]))
           .parse(parse_output_mode)
           .validate(validate_output_mode)
           .applies_to(DocumentOps::ALL)
           .default(default_output),
   };
   ```

   具体 macro、struct 和 trait 名称可由实现决定；验收重点是 core 和 SDK 使用同一个定义模型，不再分别手写 flag/config/help/校验映射。具体参数 change 只需要声明自己的参数行为，不重新定义通用机制。

2. 定义和运行时来源分离。

   Definition 声明 canonical key、config path、flag、help、校验和 finalization；运行时仍把 explicit argv、项目配置、用户配置和内置默认值合并为带来源的参数值。这样 `config list`、document context 输出、warning 和测试可以同时看到“定义是什么”和“本次值来自哪里”。

3. Core 和 SDK 共享机制，但分别注册 owned 参数集合。

   Core `docnav` 注册 `defaults.adapter`、`defaults.output` 以及其它 core-owned 标准参数。SDK direct CLI 注册 `defaults.output`、配置路径参数可用 metadata 以及其它 SDK-owned 标准参数。两边同名 canonical key 必须使用相同 value kind、config path、flag semantics 和 validation semantics；不同 owner 不得把同名 key 解释成不同业务含义。

4. Schema metadata 来自 definition，但 runtime 不依赖 schema 加载。

   Definition 必须能表达 schema generation 所需信息，例如 JSON path、type、enum、minimum、maximum、description/default。生成出的 schema/example 仍是验证材料和编辑器提示，不改变 direct CLI runtime 是否读取或校验 schema 文件。

5. Native options 保持 adapter-owned pass-through。

   `options.*` 可以继续由 adapter native option specs 和 SDK native option handling 处理。标准参数 definition 不解释 Markdown `options.max_heading_level` 这类 adapter-owned key；core 也不从 manifest、配置或 definition metadata 合成 format-specific `options`。

## Risks / Trade-offs

- **定义模型过宽，拖慢具体业务 change** -> 本 change 只交付共享机制，具体参数行为仍由各自 change 独立描述和验证。
- **core 和 SDK owner 混淆** -> 定义模型共享，参数注册集合分离；主规范必须写清 core-owned、SDK-owned 和 adapter native option 边界。
- **schema generation 变成 runtime 依赖** -> definition 只提供 schema metadata；schema 文件仍只用于验证、提示和打包参考。
- **过早锁定类型名** -> OpenSpec 约束行为和数据能力，不要求固定 Rust 类型名；实现可在保持验收语义的前提下调整命名。
- **同名 key 语义漂移** -> 增加 cross core/SDK tests，证明同名 canonical key 的 flag、config path、validation 和 source priority 一致。

## Migration Plan

1. 在共享 Rust 层引入标准参数 definition model，并用现有 `defaults.output` 或等价当前参数做首批迁移。
2. 更新 core `docnav`，使 config supported keys、argv mapping、help/default 文案、document context 输出和 typed validation 消费 core 参数定义集合。
3. 更新 `docnav-adapter-sdk` direct CLI，使 config projection、argv parsing、help/default 文案、typed validation 和 operation 参数生成消费 SDK 参数定义集合。
4. 保留现有配置 key 行为，通过测试证明迁移机制不改变当前 observable behavior。
