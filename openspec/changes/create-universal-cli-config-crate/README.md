# create-universal-cli-config-crate

从 Docnav typed-fields 和 parameter-resolution 抽象出可作为子仓库复用的 Rust CLI/config 解析底层 crate，统一 flag、env、config、默认值合并和来源追踪。

当前 change 只包含 change-local OpenSpec artifacts，不改变现有主规范、schema、examples、实现代码。

2026-07-09 的 prompt-optimize artifact 审阅结论：Docnav 集成采用 hard cutover，不做渐进运行时迁移。工作区实现使用 `cli-config-resolution` 作为 capability 与 crate/package 工作名；外部 package 名默认沿用 `cli-config-resolution`，子仓库化默认迁移到独立 repository。release-readiness 审计发现外部包名不可用、仓库策略冲突、发布渠道风险任一问题时，执行者必须主动向用户确认后再继续。
