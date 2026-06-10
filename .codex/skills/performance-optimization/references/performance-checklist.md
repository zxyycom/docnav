# Docnav Performance Checklist

此 reference 保存 Docnav 性能工作的细节。先用 `SKILL.md` 确认触发和流程，再按本文件执行测量、triage、修复和验证。

## 目录

- [Baseline](#baseline)
- [Fixture Shape](#fixture-shape)
- [Bottleneck Triage](#bottleneck-triage)
- [Command Templates](#command-templates)
- [Budget Template](#budget-template)
- [Fix Checklist](#fix-checklist)
- [Common Rationalizations](#common-rationalizations)
- [Red Flags](#red-flags)
- [Verification](#verification)

## Baseline

- [ ] 记录 binary、subcommand、flags、path、output mode、page、limit、query 和 `ref`。
- [ ] 记录 build profile：debug 或 release；性能数字默认优先 release。
- [ ] 记录 fixture size、heading count、nesting depth、重复 heading、长 section 和关键 Markdown feature。
- [ ] 多次运行并记录 wall time；可用时记录 median 或保守范围。
- [ ] 当问题涉及资源增长时，记录 CPU、working set、peak memory。
- [ ] 当 output 或 pagination 可能主导成本时，记录 stdout size 和 page 信息。
- [ ] 保留 before 命令原文，确保 after measurement 可以逐项复现。

## Fixture Shape

- [ ] 使用大 Markdown 文件，不只用短 smoke document。
- [ ] 覆盖大量 heading、多个层级和重复 heading 文本。
- [ ] 覆盖长 section，验证 `read` slicing。
- [ ] 包含表格、代码块、列表、链接、frontmatter 等真实结构。
- [ ] `find` query 覆盖零结果、少量结果和大量结果。
- [ ] `pagination` 覆盖 page 1 和 later pages。
- [ ] 超大 fixture 不宜直接入库时，记录生成脚本或复现步骤。

## Bottleneck Triage

先比较 direct adapter CLI 和 core CLI：

```powershell
Measure-Command { .\target\release\docnav-markdown.exe outline .\fixtures\large.md | Out-Null }
Measure-Command { .\target\release\docnav.exe outline .\fixtures\large.md | Out-Null }
```

| 分类 | 信号 | 优先检查 |
|---|---|---|
| Adapter parsing/navigation | direct adapter CLI 已慢 | parser pass、heading tree、section slicing、重复 full-document scan |
| Core CLI routing | `docnav` 明显慢于 adapter CLI | format detection、adapter routing、config lookup、default resolution、error mapping |
| IO/process | 首次运行或重复运行被启动/读文件主导 | binary startup、filesystem read、adapter process、stdio size |
| Output layer | `protocol-json` 或 readable output 明显慢 | serialization、pretty formatting、large snippets、pagination metadata |
| Ref lookup | `read --ref` 随 heading 数增长变慢 | ref parsing、heading lookup、section range lookup、duplicate heading handling |
| Find | broad query 或 repeated search 慢 | search scope、normalization、allocation churn、result limit |
| Pagination | later pages 昂贵或不稳定 | page slicing、result counting、continuation state、重复 rendering |
| MCP bridge | MCP tool call 慢于等价 CLI | Node bridge、process spawn、stdio JSON、repeated adapter calls |
| Memory | working set 随文档或重复调用增长 | unbounded buffers、cloned text、cached parse tree、limit 前收集全部结果 |

如果 adapter 快而 `docnav` 慢，先看 core routing 和 process boundary。如果两者都慢，优先看 adapter parsing、lookup 或 output。

## Command Templates

先构建 release binary：

```powershell
cargo build --release -p docnav -p docnav-markdown
```

有 `hyperfine` 时优先多轮测量：

```powershell
hyperfine --warmup 3 '.\target\release\docnav.exe outline .\fixtures\large.md --output protocol-json --limit-chars 8000'
hyperfine --warmup 3 '.\target\release\docnav-markdown.exe find .\fixtures\large.md --query "navigation" --output readable-json'
```

没有 `hyperfine` 时使用 PowerShell：

```powershell
Measure-Command {
  .\target\release\docnav.exe outline .\fixtures\large.md --output protocol-json --limit-chars 8000 > $null
}

Measure-Command {
  .\target\release\docnav.exe find .\fixtures\large.md --query "navigation" --output protocol-json > $null
}

Measure-Command {
  .\target\release\docnav.exe read .\fixtures\large.md --ref "L120:Guide > Adapter" --output protocol-json > $null
}

Measure-Command {
  .\target\release\docnav.exe outline .\fixtures\large.md --page 3 --limit-chars 4000 > $null
}
```

Windows memory sampling：

```powershell
$p = Start-Process .\target\release\docnav.exe -ArgumentList @(
  "outline", ".\fixtures\large.md", "--output", "protocol-json"
) -PassThru -NoNewWindow
while (-not $p.HasExited) {
  Get-Process -Id $p.Id | Select-Object Id,CPU,WorkingSet64,PeakWorkingSet64
  Start-Sleep -Milliseconds 100
}
```

## Budget Template

```text
Command: docnav outline fixtures/large.md --output protocol-json --limit-chars 8000
Fixture: 5 MB Markdown, 5,000 headings, repeated heading names, long sections
Build: release
Host: agreed local benchmark machine, warm cache
Budget: p50 <= 300 ms, variance <= 10%
Memory: peak working set <= 150 MB
Guard: adapter CLI smoke fixture plus ref lookup unit test
```

Budget 必须写明 command、fixture、output mode、page/limit、build profile、host 假设和允许噪声；否则不同测量不可比较。

## Fix Checklist

- [ ] 改动只命中已测出的 bottleneck。
- [ ] Adapter-owned `ref` 在 core CLI、MCP 和其它接入层保持 opaque。
- [ ] Output schema、ordering、pagination、continuation 和 error behavior 保持稳定。
- [ ] 协议允许时，先应用 limit，再做昂贵 formatting。
- [ ] 不为每个 result clone full document text，除非已有测量和理由。
- [ ] Cache 有明确 lifecycle、invalidation、memory bound 和 cross-call 行为。
- [ ] 大型 intermediate result list 已被 bound、stream 或避免。

## Common Rationalizations

| 说法 | 处理方式 |
|---|---|
| "parser 应该是问题" | 先比较 adapter CLI、core CLI 和 MCP path。 |
| "output 大，所以慢是正常的" | 仍要测 pagination、limit 和 output construction。 |
| "cache 会解决" | CLI process 短生命周期；只有 lifecycle 匹配 workload 时 cache 才有效。 |
| "换更快的 ref 格式" | `ref` 是 adapter-owned protocol value；保持 opacity 和兼容性。 |
| "小文件很快" | Docnav 面向大型文档导航；用真实规模和结构测。 |

## Red Flags

- 没有 before/after numbers 就优化。
- 用 tiny document 证明 large-document navigation 问题。
- debug baseline 和 release after 混比。
- 为速度改变 output shape、ordering、`ref`、pagination 或 error mapping。
- core 或 MCP 解析 adapter-owned `ref`。
- 在 limit 前无界收集全部 results。
- 为每个 match/read result clone full document text。
- 加 global cache 但没有 invalidation、memory bound 或 lifecycle rationale。

## Verification

按改动范围选择最窄验证：

```powershell
cargo test -p docnav-markdown
cargo test -p docnav
pnpm run smoke:docnav-markdown
pnpm run smoke:docnav-core
```

跨 Rust crates、schema/examples、adapter output、CLI behavior、MCP mapping 或 docs 边界时，优先运行：

```powershell
pnpm run verify:docnav-workspace
```

交付前：

- [ ] before/after 使用同一 fixture、command、output mode、build profile 和机器条件。
- [ ] improvement 大于 measurement noise。
- [ ] regression guard 覆盖 optimized code path。
- [ ] 无法自动化的 performance budget 已写明精确复现步骤。
