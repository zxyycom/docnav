本 tasks 只给出标准参数解析核心的推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [x] 1.1 阻塞级审计：确认本 change 只定义标准参数来源解析核心，不迁移 core CLI、adapter SDK、adapter direct CLI 或 `clap`。
- [x] 1.2 阻塞级审计：确认 typed field definition 是字段 metadata owner，standard parameter resolution 只消费字段 identity、schema metadata、默认值和 typed value 校验能力。
- [x] 1.3 阻塞级审计：确认 passthrough 与 owner validation 的边界没有提前限制 adapter native options、unknown argv 或未映射 invoke arguments。
- [x] 1.4 阻塞级审计：确认已废弃的 `unify-standard-parameter-definitions` 只作为历史背景，当前执行入口为本 change 的 2.x/3.x tasks。

## 2. 轮廓实现

- [ ] 2.1 决定 resolver core 的 crate/module 放置和最小可见性，先保持窄边界 API。
- [ ] 2.2 定义标准参数 registration、source kind、source object、source info、diagnostic 和 merge result 的最小结构。
- [ ] 2.3 接入 typed-field metadata，以现有 value kind、enum、range、requiredness 和 default validation 作为字段约束单一事实源。
- [ ] 2.4 实现 `direct input > project config > user config > default` 来源合并和 typed runtime value 查询。
- [ ] 2.5 实现 required/default 的 runtime 处理，确保 static/dynamic default 结果进入同一 typed-field validation。
- [ ] 2.6 实现 passthrough handoff 结构，只对已映射标准参数执行标准参数 validation。
- [ ] 2.7 建模 operation argument binding 的 identity-to-arguments-path 关系，并把 protocol request construction 留给后续 owner。

## 3. 验证

- [ ] 3.1 添加小范围 fixture，证明 direct/project/user/default 来源优先级和 source info。
- [ ] 3.2 添加 fixture，证明 required/default、typed value validation 和 invalid mapped value diagnostic。
- [ ] 3.3 添加 fixture，证明标准参数 validation 只覆盖已映射字段，passthrough 保留给 entry owner。
- [ ] 3.4 添加 fixture，证明 operation argument binding 保留 direct/config/default 的 resolved source info。
- [ ] 3.5 运行 resolver 所在 Rust crate 的 targeted tests。
- [ ] 3.6 若实现触及多个 crate 或 observable contract surface，运行 `bun run verify:docnav-workspace`。
