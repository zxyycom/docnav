# standard-parameter-resolution-core

本 change 定义标准参数来源解析核心，目标是让普通调用路径从调用方已经定义好的 `FieldDefSet` 开始。

目标流程：

```text
用户定义 FieldDefSet
    -> 标准参数层读取 schema_metadata()
    -> 标准参数层读取 strategy_metadata("direct")
    -> 标准参数层读取 strategy_metadata("config")
    -> 标准参数层内部形成 catalog/index
    -> resolve(直接输入, config 路径/descriptor 或复用的 loaded config)
    -> 返回 StandardParameterResolution
```

调用方继续用 `docnav-typed-fields` 声明字段 identity、类型、required/default、range、enum、regex、各 mapped value extraction strategy path 和 passthrough processing build。标准参数层只消费 metadata 和 caller processing result，并负责 config loading、source construction、来源合并、typed-field validation、diagnostic events 和 passthrough handoff。

普通 config 入口是 path/descriptor，由标准参数层统一执行 JSON loading、顶层 object 校验和 skipped-source diagnostic handoff。Loaded config 只用于复用同一标准参数 loader 已经产生的 loaded source；普通路径不让 caller 自行实现 JSON loading 后再传入。

Catalog/index 是标准参数层内部编译产物，只承接 typed-field metadata 到 source construction 的映射，不作为 caller 需要装配的 API 概念。

当前 change 不迁移 core CLI、adapter SDK direct CLI、adapter `invoke` 或 CLI frontend。详细范围见 `proposal.md`，设计取舍见 `design.md`，任务状态见 `tasks.md`。
