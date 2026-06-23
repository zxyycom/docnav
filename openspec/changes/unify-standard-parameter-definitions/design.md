本 change 目标是统一 core `docnav`、`docnav-adapter-sdk` direct CLI、adapter `invoke` 和 MCP tool mapping 的 args/config 标准参数基础层。本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 design；主规范同步由 tasks 中的文档任务承接。

## Context

Core `docnav`、adapter SDK direct CLI、adapter `invoke` 和 MCP tool mapping 都需要处理标准参数的不同 surface：CLI flag、MCP tool input、protocol/invoke argument、help/default 文案、配置路径、默认值、基础校验、来源追踪，以及最终进入 document operation 的参数。现在这些能力分散在各入口中，导致同名参数可能出现语义漂移。

共享层应该承接机械参数能力：CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值先分别投影为标准参数对象，再由同一 resolver 按统一全局来源优先级合并、校验并产出 typed runtime values。该优先级固定为直接输入值（CLI argv、MCP tool input 或 invoke request arguments）、项目配置、用户配置、默认值。Core、SDK、adapter `invoke` 和 MCP 仍拥有各自入口行为、输出包装和 request construction；但 `invoke` 的 protocol request `arguments` 不再被定义为调用方已经完成配置/default 解析的最终参数，而是和 CLI argv、MCP tool input 一样作为 direct input standard parameter object 进入同一标准参数解析。MCP 参与本 change 的目标是复用同一份标准参数 metadata 生成 tool input schema，并把 tool input 投影成标准参数对象。当前 MCP transport 仍可映射到 core CLI argv；让 JS 获得 metadata 的推荐方式是由 Rust 生成 JSON artifact，也允许 runtime metadata 或人工同步的等价方案，本 change 不提前锁死实现形态。

## Key Decision Log

这些决策是本 change 的审计入口，使用 D1-D7 编号供 tasks、review 和后续主规范同步引用。改变任一决策时，必须同步更新 proposal、spec delta、tasks 和验证材料。

1. **D1: 统一标准参数语义和来源合并，不统一入口 owner。**

   共享层拥有标准参数 base definition、registration set、source-to-standard-parameter-object projection、标准参数对象合并、typed runtime values、基础 validation 和 metadata 生成。Core 仍拥有 adapter routing、core config、request construction 和 exit behavior；SDK 仍拥有 direct CLI、adapter 配置域、warning 和 operation build；adapter `invoke` 仍拥有 protocol stdin/stdout lifecycle；MCP 仍拥有 transport、tool declaration packaging、TextContent/structuredContent 包装和 tool input transport mapping。

2. **D2: CLI、invoke 和 MCP 都是直接输入 surface。**

   CLI argv、adapter `invoke` request arguments 和 MCP tool input 映射为 direct input standard parameter object；项目配置、用户配置和内置默认值分别映射为其它标准参数对象。共享 resolver 按同一全局来源优先级合并这些对象；该顺序固定为 direct input standard parameter object、project config standard parameter object、user config standard parameter object、default standard parameter object。每个入口只声明自己的 source profile、可用 source provider 和 transport/output owner，不维护独立参数行为或独立优先级。`invoke` 的 protocol request `arguments` 是 resolver 输入来源，不是已经完成配置/default 合并的最终参数；protocol request/result envelope 保持稳定，但 arguments 的标准参数字段 requiredness、schema 和 examples 必须随该语义同步。MCP tool input schema 和 tool input projection 从 tool -> operation 映射、operation registration set 和 surface registration/metadata 生成；当前 MCP bridge 可以继续把 tool input 转成 core CLI argv 作为传输路径，但语义上不再维护独立参数定义。

   Core 和 SDK 在本入口内先运行完整标准参数 resolver，并可将 typed runtime values 用于本入口拥有的数据处理、context 输出、warning 或 operation build。Request construction 使用 operation argument binding 和来源追踪序列化需要跨 protocol 传递的 direct standard param source fields；已解析的配置值或默认值不得仅因 request construction 被重新分类为 adapter `invoke` direct source。下游 adapter `invoke` 是独立入口，会再次按共享 resolver 和同一全局来源优先级合并 request arguments、项目配置、用户配置和默认值。

3. **D3: 标准参数模型分三层。**

   Base definition 固定 `ParamKey<T>`、canonical key、value type、default facet、schema facet、基础 validator 和 schema metadata。Registration set 声明 consumer、CLI command、operation 或 MCP tool 暴露的 config path、CLI surface、operation argument binding 或 MCP tool input mapping。Typed runtime values 是本次调用解析后的结果，供 core、SDK operation builder、adapter invoke handler、MCP mapping、context 输出和测试复用。

4. **D4: `standard_params` 的运行时输出必须可 typed 复用。**

   共享 resolver 返回 `ResolvedStandardParams` 或等价 typed object。调用方通过 `ParamKey<T>` 获取已校验的 `T` 值；core、SDK、adapter invoke projection、MCP mapping 和 tests 不重复解析 raw argv、raw JSON、tool input 或 untyped value。

5. **D5: Base definition 不使用全局 `.applies_to`。**

   每个入口的参数集合由 registration set 或 tool mapping 声明。Operation 参数由 operation registration set 决定；CLI 参数由 CLI registration 决定；MCP tool input 由 tool -> operation 映射、operation registration set 和 MCP/CLI surface metadata 决定。

6. **D6: CLI、config、protocol 和 MCP surface 分别拥有 owner。**

   它们可以从同一个 stable name 或 `ParamKey<T>` 派生，但 owner 必须分开：CLI registration 拥有 flag/help/default surface；config registration 拥有 typed config path；operation argument binding 拥有 protocol `arguments` path；MCP tool metadata 拥有 tool input schema、tool input path 和 tool input projection。所有 surface 最终都投影到同一个标准化参数语义。

7. **D7: Schema facet 是 runtime validator 和 schema metadata 的共同来源。**

   Runtime 使用 definition 内的 schema facet 或由它编译出的 validator 校验 CLI argv value、invoke request argument value、MCP tool input value、配置值和动态默认值。Protocol request schema、MCP tool schema 和配置 schema 由同一 metadata 派生或人工同步，但 MUST 使用各自的 schema view：protocol request schema view 只校验 envelope、operation、document path、raw arguments object、已出现标准参数字段的基础 JSON 类型和字段可识别性；resolver、MCP tool 和 config schema view 可以表达更完整的 enum、range、requiredness 和 default metadata。生成后的 schema/example 只作为验证材料、编辑器提示或打包参考，不成为 runtime file dependency。

## Decisions

1. 标准参数分为 base definition、surface registration 和 runtime typed values。

   Base definition 保存稳定语义和类型能力，不表示某个入口或 operation 一定暴露该参数。目标形态类似：

   ```rust
   let limit_chars = standard_param_bases::limit_chars()
       .key(ParamKey::<PositiveInteger>::defaults_limit_chars())
       .default(DefaultFacet::dynamic(default_limit_chars))
       .schema(SchemaFacet::integer().minimum(1));

   let core_params = standard_params!(ParamOwner::Core, [
       limit_chars
           .register()
           .config_path(config::defaults().limit_chars())
           .cli(CliFacet::long("limit-chars").value())
   ]);

   let read_params = operation_params!(Operation::Read, [
       standard_param_bases::ref_param()
           .register()
           .operation_arg(protocol::arguments().ref_()),
       limit_chars
           .register()
           .operation_arg(protocol::arguments().limit_chars()),
       standard_param_bases::page()
           .register()
           .operation_arg(protocol::arguments().page()),
   ]);
   ```

   具体 Rust 类型名可由实现决定；验收关注语义分层：

   1. Base definition：`ParamKey<T>` / canonical key、value type、default facet、schema facet、基础 validator 和 schema metadata。
   2. Config registration：typed config path 或 no-config。
   3. CLI registration：CLI long flag 派生/覆盖、flag argv facet、help/default 文案或 no-flag。
   4. Direct input registration：operation argument binding 和 MCP tool input mapping；只有会暴露到对应 direct input surface 的标准参数才注册。
   5. Runtime typed values：共享解析 pipeline 返回 `ResolvedStandardParams` 或等价 typed object，调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值。

2. `standard_params` 生成的运行时结果可以作为类型化参数对象复用。

   Definition 和 registration 是静态 metadata；本次调用的最终值由共享解析 pipeline 生成。共享层返回的 typed runtime object 必须能被 core、SDK operation builder、配置 context 输出和测试复用，避免每个 consumer 重新声明同一参数的类型检查。

   目标能力：

   ```rust
   let resolved: ResolvedStandardParams = resolver.resolve(argv, config)?;
   let limit_chars: PositiveInteger = resolved.require(ParamKey::defaults_limit_chars())?;
   let output: OutputMode = resolved.get_or_default(ParamKey::defaults_output())?;
   ```

   `ParamKey<T>` 必须让调用点获得 typed value；schema facet 或由它编译出的 validator 是配置值、CLI value、operation argument value 和默认值的共同基础校验来源。Schema metadata 输出必须保留 surface-specific schema view，避免 protocol request schema 承担 resolver 才拥有的 default、requiredness、range 或 enum 决策。

3. Config path 只接受 typed path builder。

   Registration 内部的 config path 输入来源是 typed path builder。Typed path 必须能导出 path segments、显示用 dotted path 和 schema 生成位置。

4. 运行时 pipeline 由共享层完成机械处理。

   Core、SDK、adapter invoke projection 和 MCP mapping 先把各自 direct input surface 投影为 direct input standard parameter object。每个入口提供 source profile，声明自己会读取的项目/用户配置、显式覆盖路径、默认 provider 和 transport metadata。共享层按 source profile 读取配置源并校验顶层 object；项目配置、用户配置和默认 provider 也分别按 registration 投影为 project config、user config 和 default standard parameter object。后续只合并标准参数对象。

   运行时值来源包括显式 argv、MCP tool input、operation/invoke argument projection、项目配置、用户配置和默认值。共享层把每类来源映射为标准参数对象，按统一全局来源优先级合并为最终标准参数对象，执行 schema-backed validation，并返回 typed runtime values、来源信息和配置源诊断。具体入口只提供自己拥有的 source provider；CLI、MCP 和 invoke 都必须走同一 projection/merge/validation pipeline，并使用同一全局来源优先级。Request construction 消费 operation argument binding 和来源信息完成 direct source 序列化，不把配置/default completion 重新标成 direct source。

5. Direct input binding 统一拥有 surface 到标准参数的映射。

   Operation registration 把标准参数绑定到 typed protocol argument slot。CLI flag 是 CLI registration 的 surface；MCP tool input 是 tool -> operation 映射、operation registration set 和 MCP surface metadata 的直接输入 surface。三者都先投影为 direct standard param source，再参与同一 resolver。

   这意味着：

   - 简单参数可以复用同一个 stable name 派生 `--limit-chars`、`arguments.limit_chars` 和 MCP tool input path，但三者分别属于 CLI registration、operation registration 和 MCP tool metadata。
   - 特殊参数显式声明 typed operation argument path。
   - MCP tool 声明 `document_read -> read` 这类 tool-level operation 映射。
   - `document_read.limit_chars` 从 read operation 的 registered parameter set 和 MCP tool metadata 生成。
   - MCP bridge 使用 metadata 生成 tool input schema，并把 tool input 投影为 direct standard param source；当前 transport 可以继续映射到 core CLI argv，但 CLI argv spelling 只作为 transport projection metadata，不是 MCP 标准参数语义的 owner。

6. 跨 consumer 标准参数复用共享 base definition。

   Core 和 SDK 的参数集合可以不同。跨 consumer 复用的 canonical key 通过共享 base definition 或 builder factory 派生，预先固定 canonical key、value type、default facet、schema facet 和 validation semantics。Consumer registration 可以选择是否暴露 config、CLI 或 operation surface，并补充 owner-specific 配置域描述。若某个 surface 需要保持跨 consumer 不漂移，则该 surface 的默认 registration template 也应由共享 base 或 builder factory 提供。

   Build/register 阶段校验同名 key 的 base identity/fingerprint。两个 registration 声明同一个 canonical key 时，registry 只接受来自同一个 base identity 且 invariant slots 保持一致的 definition。Consumer 可以收窄自己的 operation registration set；扩大暴露范围或改变 invariant slots 必须通过新的 registration 或升级 base definition 表达。

   `--project-config-path`、`--user-config-path` 这类 control 参数可以复用 builder/help metadata，并以 control 参数身份参与 CLI surface 生成；它们不需要 operation argument binding。

## Boundaries

- 共享层拥有标准参数 base definition、registration set、source-to-standard-parameter-object projection、配置读取与投影、标准参数对象合并、来源追踪、schema-backed validation、operation argument binding metadata、MCP tool metadata、typed runtime values 和 schema metadata。
- Core 继续拥有 `docnav config get/set/unset/list`、项目根和用户配置位置、adapter selection、document context 输出、request construction 和 exit behavior。
- SDK 继续拥有 adapter direct CLI 命令分发、配置项目根发现、warning 承载、request construction 和 operation build。
- MCP bridge 继续拥有 MCP transport、tool declaration packaging、TextContent/structuredContent 包装和 tool input transport mapping。当前 transport 可以是 tool input -> core CLI argv；MCP 不拥有 adapter invoke request construction，除非后续 change 明确改变 MCP transport path。

## Risks / Trade-offs

- **共享层过宽**：只承接机械 args/config 参数能力；业务归一化和 operation build 留在 consumer。
- **同名 key 漂移**：跨 consumer key 复用共享 base definition；registry 用 definition fingerprint 校验同名 key，测试覆盖 base reuse 和 fingerprint mismatch。
- **入口参数集合不同**：base definition 不携带全局 `.applies_to`；各 consumer、CLI command、operation 和 MCP tool mapping 通过 registration set 或 metadata 明确暴露参数。
- **MCP 绑定重复**：MCP tool input schema 和 tool input projection 从 tool -> operation 映射、operation registration set 和 MCP/CLI surface metadata 生成；CLI argv spelling 只作为当前 transport projection metadata。JS 获取 metadata 的推荐路径是 Rust 生成 JSON artifact，也允许 runtime metadata 或人工同步的等价方案，人工同步必须有 artifact/schema diff 或映射测试兜底。
- **schema 变成 runtime 文件依赖**：runtime 使用 definition 内的 schema facet 或编译 validator；生成后的 schema 文件用于验证、编辑器提示和打包参考。
- **protocol schema 过度校验标准参数**：schema metadata 输出区分 protocol request、resolver、MCP tool 和 config schema view；protocol request schema view 只守住 protocol envelope 和 raw arguments 基础形状，标准参数 required/default/range/enum 由 resolver 校验。
- **默认值绕过校验**：静态默认值 build/register 校验，动态默认值 runtime 校验。

## Migration Plan

1. 引入共享 args/config 参数 owner，先覆盖当前 `defaults.output`、`defaults.limit_chars` 或等价现有标准参数。
2. 迁移 core `docnav` 的配置读取与标准参数对象投影、argv、help、context 输出和 request argument projection。
3. 迁移 `docnav-adapter-sdk` direct CLI 和 adapter `invoke` 的 source profile、配置读取与标准参数对象投影、request arguments projection、schema-backed validation 和 typed runtime values。
4. 同步 protocol request `arguments` 主规范、schema 和 examples，表达标准参数字段是 resolver direct input source，不是调用方最终 resolved params；protocol request schema 使用较窄 schema view，resolver 继续拥有标准参数 required/default/range/enum 校验。
5. 为 MCP 生成或暴露 tool metadata，使 tool input schema 和 tool input -> direct standard param source 映射消费 operation registration metadata；当前 transport 可继续投影到 core CLI argv。
6. 用 focused tests/smoke 证明迁移后当前 observable behavior 按新契约保持稳定。
