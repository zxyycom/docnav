本 design 记录标准参数来源解析核心的设计取向。

## 上下文

标准参数和 typed-field 的边界需要清晰分离：

- `docnav-typed-fields` 描述字段 identity、processing strategy、schema metadata、默认值、typed value validation 和同一 processing id 的 caller processing result。
- `docnav-standard-parameters` 描述 direct/config source role、config source loading、source construction、来源优先级、operation binding、typed runtime values、diagnostic handoff 和 passthrough handoff。

`docs/standard-parameters.md` 是长期行为 owner。标准参数层在返回 typed values、source info、diagnostic events 和 passthrough 后结束；Core CLI、adapter SDK、protocol request construction 和 readable/raw output 只消费结果，不重新实现标准参数来源规则。

## 目标流程

```text
用户定义 FieldDefSet
    -> 标准参数层读取 schema_metadata()
    -> 标准参数层读取 processing_metadata("direct")
    -> 标准参数层读取 processing_metadata("config")
    -> 标准参数层内部形成 catalog/index
    -> resolve(直接输入, config 路径/descriptor 或复用的 loaded config)
    -> 返回 StandardParameterResolution
```

目标 API 形态：

```rust
let fields = Params::field_defs()?;

let resolution = StandardParameterPipeline::new(&fields)
    .with_direct_input_processing_id("direct")
    .with_config_processing_id("config")
    .with_project_config_path(project_config_path)
    .with_user_config_path(user_config_path)
    .resolve(direct_input)?;
```

`Params` 仍由 caller 通过 typed-fields 定义。Pipeline 只消费 metadata，不替 caller 定义参数。

普通 config 输入是 path/descriptor。Pipeline 负责 JSON loading、顶层 object 校验、skipped-source diagnostics 和 config source conversion。Loaded config 只用于复用 `LoadedStandardParameterConfigSource`，且该 loaded source 必须由标准参数 loader 产生；它不是 caller 自行实现 JSON loading 的扩展点。

## 目标 / 非目标

**目标：**

- 建立标准参数来源模型：direct input、project config、user config 和 default。
- 提供以 caller-defined `FieldDefSet` 为入口的 pipeline facade。
- 通过 `schema_metadata()`、`processing_metadata("direct")` 和 `processing_metadata("config")` 在 `docnav-standard-parameters` 内部形成 catalog/index。
- 从 catalog/index 和 caller 输入构造 direct/config/default sources。
- 从 path/descriptor 读取 project/user config sources，并交接 source-skipped diagnostic events。
- 支持复用标准参数 loader 产生的 loaded config source，并保持与 path-based pipeline 相同的 post-load source construction 语义。
- 按 `direct input > project config > user config > default` 合并 sources。
- 对所有 mapped values 和 defaults 复用 typed-field validation。
- 返回 typed values、source info、diagnostic events 和 passthrough handoff。
- 让 operation argument binding 仅作为 identity-to-protocol-arguments-path metadata。

**非目标：**

- Consumer migration：core CLI、adapter SDK direct CLI、adapter `invoke` 和现有 config command behavior 留给后续 change。
- CLI frontend：本 change 不选择或替换 CLI parser。
- Field declaration helpers：标准参数层不定义 caller fields、types、validation constraints 或 processing paths。
- Non-standard-parameter JSON：manifest、probe、protocol response 等 JSON contract 留给各自 owner。
- Observable contract：public schema、examples、readable/raw output、diagnostic text、stable warning/error id、stable error code 和 protocol envelope 保持当前 owner。
- Entry-specific policy：unknown argv tokenization、ignored-argv warning ownership、native option semantic validation、exit code 和 stdout/stderr placement 留给入口 owner。

## 决策

1. Typed fields 是字段事实源。
   - Decision: Field identity、type、required/default、range、enum、regex、mapped value processing strategy paths 和 processing build 都通过 `docnav-typed-fields` 声明或承载；typed-fields 的 `process` 在同一 processing id 下返回 extraction result 和 caller processing result。
   - Rationale: 标准参数层只消费字段 metadata 和 caller passthrough processing result，并拥有来源解析；重复字段事实或在标准参数层重建处理语义会形成两套事实源。

2. Pipeline 固定读取 direct/config processing metadata。
   - Decision: Pipeline 读取 `schema_metadata()`、direct processing metadata 和 config processing metadata 来形成 catalog/index。
   - Rationale: Direct input path 和 config path 是字段 processing facts。Default 不作为第三个 processing role；它来自 typed-field defaults 和 caller-provided dynamic defaults。

3. Pipeline facade 是普通 caller 边界。
   - Decision: 普通 caller 传入 `FieldDefSet`、direct/config processing ids、direct input、config source descriptors 或 paths、dynamic defaults 和 passthrough policy。Pipeline 返回 `StandardParameterResolution`。
   - Rationale: 这样可以去掉重复 caller glue，同时保持字段定义 ownership 在 typed-fields。

4. Catalog/index 是内部编译层。
   - Decision: Pipeline 内部构建 catalog/index，用于表达 per-field source binding、operation argument binding 和 conflict checking。
   - Rationale: 这些信息是 source construction 的内部索引，不是 caller 的装配模型。

5. Config loading 属于标准参数层。
   - Decision: 普通 config 输入是 path/descriptor。Pipeline 读取 JSON、校验顶层 object，并产生 source-skipped diagnostic events。Already loaded config 只接受由同一标准参数 loader 产生的 `LoadedStandardParameterConfigSource`。
   - Rationale: 如果普通路径允许 caller 自行加载任意 JSON，会在 shared post-load source construction 之前产生多套 load/error/diagnostic 行为。

6. 来源优先级固定。
   - Decision: Resolution order 是 direct input、project config、user config、default。
   - Rationale: 标准参数行为不应随 entrypoint 漂移。

7. Passthrough 不参与标准参数 validation。
   - Decision: Direct input、project config 和 user config 的 passthrough 按 caller passthrough processing result 和 entry passthrough policy 以 source scope 返回，不作为标准参数校验。
   - Rationale: Adapter native options、raw-minus-mapped、locator、删除逻辑和未来扩展字段仍由对应 entry 或 adapter owner 解释；标准参数层只交接处理结果，不重组 JSON 子树。

8. Operation argument binding 保留来源语义。
   - Decision: Operation binding 记录 identity-to-arguments-path metadata，并携带 resolved source info。
   - Rationale: Request construction 发生在标准参数解析之后，不能把 config/default values 重新分类为 direct input。

9. Diagnostics handoff 只交接事件。
   - Decision: Standard parameter validation failures 和 recoverable config-source skipped warnings 通过 shared diagnostics path 返回。Formatting、channel placement 和 exit behavior 留给 entry owners。
   - Rationale: 标准参数层拥有机械的 source/validation facts；输出策略仍由统一 owner 处理。

## 风险 / 取舍

- Implementation scope 可能扩张到 consumer migration。Mitigation: 本 change 停在 reusable pipeline，core/SDK migration 留给后续 change。
- 实现验收可能停留在中间产物测试。Mitigation: 验证必须覆盖 `FieldDefSet` pipeline 的普通调用路径。
- Config loading 可能在 source construction 前分裂。Mitigation: config path/descriptor 是普通入口；loaded config 只复用标准参数 loader 的结果。
- Config source warning handling 可能形成第二条 diagnostics 通道。Mitigation: source-skipped warnings 使用 shared diagnostic event metadata，输出策略留给 entry owners。
- Passthrough policy 可能误校验 native options。Mitigation: 标准参数 validation 只运行在 mapped identities 上。
- Source info 未来可能进入 observable output。Mitigation: 本 change 只保留内部 attribution；任何 output surface 需要独立更新 docs/examples/tests。
