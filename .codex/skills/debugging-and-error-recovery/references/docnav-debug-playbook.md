# Docnav Debug Playbook

本引用用于 Docnav 仓库内需要 focused commands、adapter replay、failure map 或验证矩阵时。命令块展示 command shape；实际可执行命令应来自当前仓库脚本、AGENTS 规则、构建产物或相邻测试，不把 build output path 写成通用规则。

## Focused Commands

优先选择最窄命令：

```bash
<rust-test-command> -- exact_case_name
<adapter-smoke-script>
<core-smoke-script>
```

Markdown adapter 行为直接重放 operation：

```bash
<adapter-or-core-cli> info path/to/file.md --output readable-view
<adapter-cli> outline path/to/file.md --output protocol-json --limit-chars 1000
<adapter-cli> read path/to/file.md --ref "<ref-from-outline>" --page 1 --limit-chars 1000 --output protocol-json
<adapter-cli> find path/to/file.md --query "needle" --output protocol-json
<adapter-cli> probe path/to/file.md
<stdin-json> | <adapter-cli> invoke
```


## Adjacent-Layer Comparisons

- Direct adapter 通过、core CLI 失败：检查 routing、config、adapter discovery、process invocation、path resolution、output mapping。
- `protocol-json` facts 正确但 `readable-view` 异常：检查 built-in mapping、header fields 和 block framing；raw facts 异常时检查 protocol envelope、schema fields、errors 和 page metadata。
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
<adapter-or-core-cli> outline path/to/file.md --output readable-view
<adapter-or-core-cli> outline path/to/file.md --output protocol-json
<adapter-or-core-cli> read path/to/file.md --ref "<ref-from-outline>" --output readable-view
```


## Workspace Verification Triggers

触碰这些边界后运行 repository workspace verifier，或记录跳过原因：

- Rust crates 跨 `docnav` 与 adapter。
- CLI behavior 或 adapter contract。
- schemas、examples、generated fixtures。
- docs 中公开的 protocol、output mode 或 command behavior。
