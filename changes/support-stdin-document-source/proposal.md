# Proposal

本 Draft 记录一个通用文档输入方向：让 `docnav` 的 linked format adapters 都能像读取文件一样读取显式传入的 standard input，而不建立 JSON-only 特例。本文件只拥有当前 Change 的临时目标，不表示 Current contract、实施优先级或实施授权。

## Why

Current document operation 只能从 `<path>` 取得文档内容，但 shell pipeline、HTTP response、生成器输出和前置转换结果经常已经以字节流存在。要求 caller 为这些内容先创建临时文件，会增加无业务价值的落盘、命名和清理步骤，也削弱 CLI-first 组合能力。

这项能力的共同语义是“为一次 document invocation 提供完整内容”，而不是 JSON grammar 的特殊行为。若只在 `docnav-json` 内读取 stdin，core、adapter contract 和后续格式会形成 adapter-specific 分支，并让 adapter 直接依赖进程级 I/O。把 stdin 建模为共享 document source，可以让当前 Markdown、JSON 以及采用同一 document contract 的未来 adapter 复用同一入口，同时继续由各 adapter 拥有格式校验、解析、导航和 ref 语义。

## Outcome

目标状态是：caller 可以用显式的 `-` document operand 和 selected adapter，把 standard input 作为一次 invocation 的文档来源；core/navigation 负责来源与生命周期，selected adapter 仍只处理其格式内容。同一 invocation 内的 outline、read、find、info、full-read 和 eligible auto-read 复用同一份输入视图，不新增 JSON-only CLI、配置或 adapter 行为。
