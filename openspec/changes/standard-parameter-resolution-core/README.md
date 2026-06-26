# standard-parameter-resolution-core

本 change 在 typed fields 之上定义标准参数来源解析核心。范围包括 source construction、配置 source 读取、来源合并、typed runtime values、diagnostics、passthrough 和 operation argument binding。

当前不迁移 core CLI、adapter SDK direct CLI、adapter `invoke` 或 CLI frontend。详细范围见 `proposal.md`，设计取舍见 `design.md`，任务状态见 `tasks.md`。
