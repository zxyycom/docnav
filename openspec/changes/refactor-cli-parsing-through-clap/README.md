# refactor-cli-parsing-through-clap

本 change 规划一次 CLI 解析 hard cutover：Clap 负责命令结构和参数解码，`cli-config-resolution-clap` 负责动态 canonical fields 的通用投影，Docnav 保留产品命令、adapter 选择和诊断策略。

状态：未审核临时 change；实现前必须先完成 [tasks.md](tasks.md) 的阻塞级审计。

建议阅读顺序：

1. [proposal.md](proposal.md)：目标、范围和影响。
2. [design.md](design.md)：owner 边界、目标流程和技术决策。
3. `specs/*/spec.md`：可验证的 capability deltas。
4. [tasks.md](tasks.md)：带依赖和门禁的实施清单。
