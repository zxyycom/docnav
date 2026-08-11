## Why

Markdown v0 adapter 已经有单元、adapter 级、invoke 和直接 CLI 测试，但测试契约还没有明确要求完整黑盒 CLI smoke corpus 和系统化负向 CLI 矩阵。现在补齐这一层，可以更早发现可执行文件打包、输出模式、参数校验、stdout/stderr 边界、退出码和 schema 形状回归。

## What Changes

- 新增 `docnav-markdown` 黑盒 CLI smoke 测试要求，测试必须由 Node.js 脚本启动构建后的真实 adapter 可执行文件。
- 要求 fixture corpus 以项目文件形式固定放在指定测试目录，覆盖 normal、duplicate headings、frontmatter、code fence、deep headings、no headings、unicode、large pagination、non-UTF-8。
- 额外补充 UTF-8 BOM、CRLF 行尾、`.MD` 大小写扩展名和 `.markdown` 扩展名 fixture/场景。
- 要求 smoke flow 覆盖 `outline -> ref -> read`，以及 `find`、`info`、`probe`、`manifest`。
- 要求覆盖 `text`、`readable-json`、`protocol-json` 三种输出模式，并校验 JSON 输出通过对应 schema 或等价结构断言。
- 要求覆盖有效 `invoke` 请求的黑盒路径，确认 adapter stdin/stdout 协议入口仍可工作。
- 新增负向 CLI 矩阵：缺 path、缺 `--ref`、缺 `--query`、unknown flag、page/limit 为 0 或非数字、`max-heading-level` 越界、missing file、invalid ref、non-UTF-8、malformed invoke JSON。
- 要求所有正向和负向场景都断言 stdout、stderr 和 exit code。
- 要求 Node.js runner 将每条测试命令和结果写入审计日志，包括命令、cwd、exit code、stdout、stderr 和断言摘要，便于失败后复查。
- 非目标：本 change 不增加核心 `docnav` 路由 smoke，不增加 adapter install 管理 smoke，也不增加 MCP bridge smoke；这些留在对应 active change 中处理。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter-v0-implementation`：补充 `docnav-markdown` 黑盒 CLI smoke、fixture corpus、输出层/schema 断言和负向矩阵测试要求。

## Impact

- 影响 `crates/docnav-markdown` 下的固定 fixture 文件，以及 `scripts/` 或 adapter 测试目录下的 Node.js 黑盒测试脚本。
- 可能新增 package script，用于先构建 `docnav-markdown`，再把构建产物路径传给 Node.js smoke runner。
- Node.js 测试 helper 负责启动 adapter executable、喂 stdin、解析 JSON、校验 schema/字段、检查 stdout/stderr/exit code，并写入 `.log/` 下的审计日志。
- 不改变 runtime protocol、manifest、probe schema 或 adapter 对外行为；若新增测试暴露已有缺陷，可在实现阶段按既有主规范修复。
