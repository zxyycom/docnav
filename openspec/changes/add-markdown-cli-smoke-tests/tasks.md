一句话核心：为 `docnav-markdown` 补齐完整黑盒 CLI smoke 和负向矩阵测试。

## 0. 审计门禁

- [x] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现。

## 1. Fixture 与黑盒测试基础

- [x] 1.1 建立 Node.js 黑盒 smoke runner，通过构建后的 `docnav-markdown` binary 启动真实进程，并支持 stdin 输入。
- [x] 1.2 增加 package script 或验证脚本入口，先构建 `docnav-markdown`，再把 binary 路径传给 Node.js smoke runner。
- [x] 1.3 在固定项目目录准备 fixture corpus：normal、duplicate headings、frontmatter、code fence、deep headings、no headings、unicode、large pagination、non-UTF8。
- [x] 1.4 以固定项目文件补充 UTF-8 BOM、CRLF、`.MD` 和 `.markdown` fixture，不在测试运行时临时生成核心案例。
- [x] 1.5 为 Node.js runner 增加 stdout、stderr、exit code、JSON 解析、schema/字段结构和 protocol/readable envelope 边界断言工具。
- [x] 1.6 为 Node.js runner 增加审计日志输出，写入 `.log/docnav-markdown-cli-smoke/latest.log` 和时间戳日志。

## 2. 正向 CLI Smoke

- [x] 2.1 覆盖 `outline --output readable-json -> extract ref -> read --output readable-json`，校验 content、content_type、page 和无 protocol envelope。
- [x] 2.2 覆盖 `outline/read/find/info` 的 `text` 输出，校验 ref、内容、content_type/cost/capabilities/page 的关键可读信息和成功路径空 stderr。
- [x] 2.3 覆盖 `outline/read/find/info` 的 `protocol-json` 输出，校验成功 envelope、operation、ok、result 形状和 stderr 边界。
- [x] 2.4 覆盖 `manifest` 和 `probe` 的 protocol-json 输出，校验 manifest capabilities/recommended parameters 与 probe evidence 且无导航 payload。
- [x] 2.5 覆盖有效 `invoke` 黑盒请求，校验 stdin 到 stdout 的成功 protocol envelope。
- [x] 2.6 覆盖 duplicate headings、frontmatter、code fence、deep/no headings fallback、unicode、large pagination、UTF-8 BOM、CRLF、`.MD` 和 `.markdown` 的黑盒行为。
- [x] 2.7 覆盖分页继续和超过末尾 page：使用返回 page 继续读取，越界 page 返回空结果和 `page: null`。
- [x] 2.8 对 readable-json、protocol-json、manifest 和 probe JSON 输出执行 schema 校验或等价字段集合断言。
- [x] 2.9 在正向 smoke 中校验每条命令的 command、cwd、exit code、stdout、stderr 和断言摘要已写入审计日志。

## 3. 负向 CLI 矩阵

- [x] 3.1 覆盖缺 path、缺 `--ref`、缺 `--query`、unknown flag 的非零退出、stderr 诊断和空 stdout。
- [x] 3.2 覆盖 `page`/`limit_chars` 为 0 或非数字、`max_heading_level` 越界的非零退出、stderr 诊断和空 stdout。
- [x] 3.3 覆盖 missing file、invalid ref、non-UTF8 在 `readable-json` 下返回 stdout readable error，保留 code/details/guidance 且不含 protocol envelope。
- [x] 3.4 覆盖 invalid ref、non-UTF8 在 `protocol-json` 下返回 failure envelope，保留 request operation 和 stable details。
- [x] 3.5 覆盖 malformed invoke JSON，校验 stdout 为 `operation: null` 的 `INVALID_REQUEST` protocol failure，stderr 仅诊断，退出非零。
- [x] 3.6 覆盖合法 JSON 但 schema/参数错误的 invoke 请求，校验 `INVALID_REQUEST` failure envelope、operation 保留规则和非零退出。
- [x] 3.7 在负向矩阵中校验失败命令的 stdout、stderr、exit code 和断言结果已写入审计日志。

## 4. 验证与审计

- [x] 4.1 运行新增 Markdown CLI smoke script。
- [x] 4.2 运行 `cargo test -p docnav-markdown`。
- [x] 4.3 运行 `pnpm run verify:docnav-workspace`。
- [x] 4.4 人工抽查 `.log/docnav-markdown-cli-smoke/latest.log`，确认命令、结果和失败详情可用于审计。
- [x] 4.5 用局部 diff 确认只修改 Markdown adapter 测试、固定 fixtures、Node.js runner、验证入口和本 change 范围。
