# Triage Cues

本引用用于根据症状定位 boundary、选择 regression guard 和识别调试风险。

## Symptom To Boundary

- Valid input 被 format/probe check 拒绝：检查 extension/content sniffing、manifest/config、registration、unsupported-format errors。
- 错误文件被 format/probe check 接受：检查 sniffing precedence 和 routing fallback behavior。
- stdin/tool invocation 失败但 direct CLI/API 可用：保存 stdin JSON 或 tool args，验证 operation args 和 envelope decoding。
- Parsed entries 缺失或多出：检查 syntax style、nesting、limits、ignored regions、escapes、generated fixture input。
- Read/detail 内容错误：对比 generated identifier、parsed identifier、selected slice、line range、body exclusion、child-section behavior。
- Pages 被截断或重复：检查 limit accounting、page math、continuation metadata、Unicode boundaries。
- Schema mismatch：先把失败字段与 owning type 对比，再修改 fixtures。
- 只有 `readable-view` framing 变化：先验证 JSON modes，再按 formatting-only change 处理。
- Bridge 与 CLI/API 不同：检查 stdio JSON、tool arg/result mapping、error mapping、child process invocation。
- Windows-only failure：在复现和 regression test 中保留精确 path form 和 shell quoting。
- Generated fixture changed：确认变化来自 generator、source fixture、schema 还是 implementation。

## Regression Guard Cues

- 解析或切片：fixture document 应小到能一眼看出 line/section boundary。
- Identifier/read：test 同时检查 generated identifier 和 read/detail result，避免只证明其中一半。
- Pagination：覆盖 page 1、continuation page 和超出范围 page。
- Output modes：JSON contract test 优先；`readable-view` snapshot 只覆盖 display/framing behavior。
- Bridge：assert tool args 到 CLI/API args/result mapping，不复制 parser expectation。
- Windows path：使用原始字符串形式作为 test case 名称或 fixture 注释，便于复现。

## Debugging Red Flags

这些信号出现时，回到 evidence record：

- 修复前没有一个稳定复现。
- 只改 expected output，缺少 generator/schema/source 的对齐证据。
- 在 bridge、formatter 或 caller 层掩盖 owning parser/domain 缺陷。
- 用 broad retry、fallback identifier 或 partial JSON 代替 structured error。
- 同时修改 parser、schema、fixtures、bridge 和 docs，但没有分边界验证。
- 错误输出中的命令、URL 或路径被直接执行。

## Recovery Moves

1. 写下 observed vs expected。
2. 选择一个相邻层比较，证明问题在哪一侧。
3. 删除与复现无关的输入，同时保留触发失败的 path/identifier/page/output mode。
4. 把修复限制在 owning boundary。
5. 让 guard 先表达失败，再验证修复。
