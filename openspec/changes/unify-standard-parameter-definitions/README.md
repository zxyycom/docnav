# unify-standard-parameter-definitions

统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制。

本 change 只规划共享 builder-style Rust definition model，以及 core/SDK 如何用同一模型驱动 flag、help、配置 path、校验、来源合并和 schema metadata；具体业务参数变更由各自独立 change 承接。
