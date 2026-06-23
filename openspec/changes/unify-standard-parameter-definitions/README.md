# unify-standard-parameter-definitions

统一 core `docnav`、`docnav-adapter-sdk` direct CLI、adapter `invoke` 和 MCP tool mapping 的 args/config 标准参数基础层。

本 change 规划一个共享 Rust owner，用标准参数 base definition、registration set、standard parameter object projection 和 typed runtime values 驱动 CLI flag/help、typed 配置路径、配置读取与投影、MCP tool input、invoke arguments、schema-backed validation、来源追踪、operation argument binding 和 schema metadata。CLI argv、MCP tool input、invoke request arguments、项目配置、用户配置和默认值分别映射为标准参数对象；共享 resolver 按统一全局来源优先级合并这些对象：直接输入值（CLI argv、MCP tool input 或 invoke request arguments）、项目配置、用户配置、默认值，再进入统一校验和正常调用逻辑。Protocol request/result envelope 不变，但 `arguments` 的标准参数语义从调用方最终参数改为 resolver 的直接输入来源。

关键决策记录在 [design.md](design.md#key-decision-log)，并以 D1-D7 编号供 tasks 和 review 引用。后续若改变任一决策，必须同步更新 proposal、spec delta、tasks 和对应验证材料。
