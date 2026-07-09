# create-universal-cli-config-crate

从 Docnav typed-fields 和 parameter-resolution 抽象出可作为子仓库复用的 Rust CLI/config 解析底层 crate，统一 flag、env、config、默认值合并和来源追踪。

当前 change 只包含未审核 OpenSpec artifacts。实现前必须先完成 `tasks.md` 中的阻塞级审计。
