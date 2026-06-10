# Docnav Debug Playbook

本引用用于需要 focused commands、adapter replay、failure map 或验证矩阵时。

## Focused Commands

优先选择最窄命令：

```bash
cargo test -p docnav-markdown --test adapter -- exact_case_name
cargo test -p docnav-markdown --test cli -- exact_case_name
cargo test -p docnav -- exact_case_name
pnpm run smoke:docnav-markdown
pnpm run smoke:docnav-core
```

Markdown adapter 行为直接重放 operation：

```bash
target/debug/docnav-markdown.exe info path/to/file.md --output readable-json
target/debug/docnav-markdown.exe outline path/to/file.md --output protocol-json --limit-chars 1000
target/debug/docnav-markdown.exe read path/to/file.md --ref "L1:Heading" --page 1 --limit-chars 1000 --output protocol-json
target/debug/docnav-markdown.exe find path/to/file.md --query "needle" --output protocol-json
target/debug/docnav-markdown.exe probe path/to/file.md
Get-Content request.json | target/debug/docnav-markdown.exe invoke
```

MCP 失败时保存 tool args，把 MCP result 与等价 `docnav` 或 `docnav-markdown.exe` 命令对比，再改 bridge code。

## Adjacent-Layer Comparisons

- Direct adapter 通过、core CLI 失败：检查 routing、config、adapter discovery、process invocation、path resolution、output mapping。
- Core CLI 通过、MCP 失败：检查 stdio framing、JSON serialization、tool names、argument names、result wrapping、error mapping。
- `text` 不同但 JSON modes 一致：按 display formatting 处理。
- `readable-json` 与 `protocol-json` 不同：检查 envelope/wrapper、schema fields、warnings、errors、page metadata。
- `outline` 正确但 `read` 错误：检查 ref generation、ref parsing、region lookup、slicing、pagination。
- `probe` 与直接命令不一致：检查 manifest registration、extension/content sniffing、unsupported-format behavior、core routing。

## Input Isolation

按失败类型缩小输入：

- Markdown parser：保留 headings、fences、escaped markers、blank lines、Unicode text 中触发失败的最小组合。
- Ref/read：保留一个 outline ref、一个 read command、一个 page 和一个 limit。
- Pagination：聚焦 `limit-chars`、page continuation、truncation 和 multibyte boundary。
- Schema/output：用最小 protocol object 对照 schema、example 或 fixture。
- Generated fixture：先运行或检查 generator path，再接受 snapshot churn。
- Windows path：保留 drive letter、backslashes、spaces、quotes 和 relative path form。

间歇失败要比较 OS、shell、Rust toolchain、Node/pnpm 版本、cwd、env vars、config store、generated files 和 test order。临时 instrumentation 放在疑似边界附近，修复后移除或转成 structured diagnostics。

## Output Mode Replay

对 navigation 行为至少检查关键 modes：

```bash
target/debug/docnav-markdown.exe outline path/to/file.md --output text
target/debug/docnav-markdown.exe outline path/to/file.md --output readable-json
target/debug/docnav-markdown.exe outline path/to/file.md --output protocol-json
target/debug/docnav-markdown.exe read path/to/file.md --ref "L1:Heading" --output protocol-json
```

当 raw protocol 字段变化时，把 schema、examples、generated fixtures 和 MCP mapping 纳入同一验证计划。

## Workspace Verification Triggers

触碰这些边界后运行 `pnpm run verify:docnav-workspace`，或记录跳过原因：

- Rust crates 跨 `docnav` 与 adapter。
- CLI behavior 或 adapter contract。
- schemas、examples、generated fixtures。
- docs 中公开的 protocol、output mode 或 command behavior。
- MCP tool mapping、stdio/JSON behavior 或 smoke coverage。
