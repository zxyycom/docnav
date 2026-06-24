
## Context



## Key Decision Log

这些决策是本 change 的审计入口，使用 D1-D7 编号供 tasks、review 和后续主规范同步引用。改变任一决策时，必须同步更新 proposal、spec delta、tasks 和验证材料。

1. **D1: 统一标准参数语义和来源合并，不统一入口 owner。**




   Core 和 SDK 在本入口内先运行完整标准参数解析，并可将类型化结果用于本入口拥有的数据处理、context 输出、warning 或 operation build。Request construction 使用 operation argument binding 和来源追踪序列化需要跨 protocol 传递的显式字段，以及当前入口策略明确保留的透传字段；已解析的配置值或默认值不得仅因 request construction 被重新分类为 adapter `invoke` direct source。下游 adapter `invoke` 是独立入口，会再次按共享规则处理 request arguments、项目配置、用户配置、默认值和透传字段。

3. **D3: 标准参数模型分三层。**


4. **D4: `standard_params` 的运行时输出必须可 typed 复用。**


5. **D5: Base definition 不使用全局 `.applies_to`。**




7. **D7: Schema facet 是 runtime validator 和 schema metadata 的共同来源。**


## Decisions

1. 标准参数分为 base definition、entry registration 和 runtime typed values。

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
   5. Runtime typed values：共享解析 pipeline 返回 `ResolvedStandardParams` 或等价 typed object，调用方可通过 `ParamKey<T>` 取得已校验的 `T` 值。

2. `standard_params` 生成的运行时结果可以作为类型化参数对象复用。

   Definition 和 registration 是静态 metadata；本次调用的最终值由共享解析 pipeline 生成。共享层返回的 typed runtime object 必须能被 core、SDK operation builder、配置 context 输出和测试复用，避免每个 consumer 重新声明同一参数的类型检查。

   目标能力：

   ```rust
   let resolved: ResolvedStandardParams = resolver.resolve(argv, config)?;
   let limit_chars: PositiveInteger = resolved.require(ParamKey::defaults_limit_chars())?;
   let output: OutputMode = resolved.get_or_default(ParamKey::defaults_output())?;
   ```

   `ParamKey<T>` 必须让调用点获得 typed value；schema facet 或由它编译出的 validator 是配置值、CLI value、operation argument value 和默认值的共同基础校验来源。Schema metadata 输出必须保留 entry-specific schema view，避免 protocol request schema 承担解析器才拥有的 default、requiredness、range 或 enum 决策。

3. Config path 只接受 typed path builder。

   Registration 内部的 config path 输入来源是 typed path builder。Typed path 必须能导出 path segments、显示用 dotted path 和 schema 生成位置。

4. 运行时 pipeline 由共享层完成机械处理。



5. Direct input binding 统一拥有入口字段到标准参数的映射。


   这意味着：

   - 特殊参数显式声明 typed operation argument path。

6. 跨 consumer 标准参数复用共享 base definition。

   Core 和 SDK 的参数集合可以不同。跨 consumer 复用的 canonical key 通过共享 base definition 或 builder factory 派生，预先固定 canonical key、value type、default facet、schema facet 和 validation semantics。Consumer registration 可以选择是否暴露 config、CLI 或 operation 字段映射，并补充 owner-specific 配置来源描述。若某个入口字段映射需要保持跨 consumer 不漂移，则默认 registration template 也应由共享 base 或 builder factory 提供。

   Build/register 阶段校验同名 key 的 base identity/fingerprint。两个 registration 声明同一个 canonical key 时，registry 只接受来自同一个 base identity 且 invariant slots 保持一致的 definition。Consumer 可以收窄自己的 operation registration set；扩大暴露范围或改变 invariant slots 必须通过新的 registration 或升级 base definition 表达。

   `--project-config-path`、`--user-config-path` 这类 control 参数可以复用 builder/help metadata，并以 control 参数身份参与 CLI flag 生成；它们不需要 operation argument binding。

## Boundaries

- Core 继续拥有 `docnav config get/set/unset/list` 命令入口、adapter selection、document context 输出、request construction 和 exit behavior；core 配置路径、类型化配置路径和来源合并规则由 `docs/standard-parameters.md` 承接。
- SDK 继续拥有 adapter direct CLI 命令分发、warning 承载、request construction 和 operation build；adapter direct CLI 配置项目根发现、配置路径、类型化配置路径和来源合并规则由 `docs/standard-parameters.md` 承接。

## Risks / Trade-offs

- **共享层过宽**：只承接机械标准参数能力；业务归一化和 operation build 留在 consumer。
- **同名 key 漂移**：跨 consumer key 复用共享 base definition；registry 用 definition fingerprint 校验同名 key，测试覆盖 base reuse 和 fingerprint mismatch。
- **schema 变成 runtime 文件依赖**：runtime 使用 definition 内的 schema facet 或编译 validator；生成后的 schema 文件用于验证、编辑器提示和打包参考。
- **默认值绕过校验**：静态默认值 build/register 校验，动态默认值 runtime 校验。

## Migration Plan

1. 新增 `docs/standard-parameters.md`，完整承接标准参数机制，并更新 `docs/navigation.md` 规则所有权；入口主规范同步为消费方说明。
2. 引入共享标准参数实现，先覆盖当前 `defaults.output`、`defaults.limit_chars` 或等价现有标准参数。
3. 迁移 core `docnav` 的配置读取、argv、help、context 输出和 request argument construction。
4. 迁移 `docnav-adapter-sdk` direct CLI 和 adapter `invoke` 的入口策略、配置读取、request arguments、schema-backed validation 和类型化结果。
5. 同步 protocol request `arguments` 主规范、schema 和 examples，表达标准参数字段是 adapter `invoke` 显式输入，不是调用方最终 resolved params；protocol request schema 使用较窄 schema view，解析器继续拥有标准参数 required/default/range/enum 校验，未映射 argument 字段由入口策略处理。
7. 用 focused tests/smoke 证明迁移后当前 observable behavior 按新契约保持稳定。
