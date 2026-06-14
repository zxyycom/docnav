# 安全清单（Security Checklist）

这是 code review 时使用的 Docnav security quick reference。需要深入 threat model 或改动本身以 hardening 为主时，切到 [security-and-hardening](../../security-and-hardening/SKILL.md)。

重点是 Docnav 的 local-tool trust boundaries：恶意文档、opaque ref、filesystem path、adapter process、stdio/JSON、schema/protocol validation、generated output、external command、Node MCP bridge、dependency 与 secret handling。不要把本清单降级成通用 Web auth/session 审查。

## 威胁建模（Threat Modeling）

- [ ] 已标出 touched boundary：parser/frontmatter、ref generation/parsing、path resolution、adapter invocation、stdio JSON、output mode、schema validation、MCP tool args/results、generated docs/examples、dependency script 或 external command。
- [ ] 已命名要保护的 asset：workspace containment、local files、protocol stability、adapter integrity、schema trust、secret、CI trust、developer machine safety 或 user data。
- [ ] 至少写出一个 abuse case：traversal、forged/overlong ref、cross-document ref reuse、malformed Unicode、oversized document、malformed JSON、stdout pollution、readable/protocol output confusion、prompt injection in docs、shell argument injection、schema bypass 或 secret leakage。
- [ ] Control 放在第一个理解数据的 code boundary，而不是放在 prompt、注释或调用方约定里。

## 输入、Refs 与路径（Inputs / Paths）

- [ ] Document text、headings、links、frontmatter、code blocks、generated text、browser/tool output 与 model output 都按 untrusted data 处理。
- [ ] Ref 在 owning adapter 外保持 opaque；`docnav`、MCP 和其他 adapter 不解析、不改写、不拼装 ref。
- [ ] Ref validation 覆盖 malformed、overlong、unsupported、wrong-format 与 cross-document reuse。
- [ ] Path 在访问前 canonicalize，并按 workspace/root policy 检查 traversal、symlink escape、absolute path surprise 与 document-derived executable path。
- [ ] Large document、deep nesting、recursive traversal 与 pagination 有明确 size/time/depth limits。

## Adapter 进程与命令（Processes / Commands）

- [ ] Adapter discovery/invocation 使用固定 executable 与 structured argv；没有 shell interpolation 或 string-built command。
- [ ] cwd/env 显式且最小化；secret 不被无意传给 adapter 或 external command。
- [ ] stdout、stderr、exit status、timeout 与 output-size caps 被确定性处理。
- [ ] Adapter failure 不会被包装成成功 protocol output。
- [ ] 触及 generated artifacts、CI scripts 或 dependency install scripts 时，按 executable attack surface 审查。

## Protocol、Schema 与输出（Output）

- [ ] Stdio JSON 只解析一次，并拒绝 malformed envelope、partial data、wrong content type 与 trailing data。
- [ ] Raw `protocol-json`、readable-json、readable-view 与非文档 PlainText 没有混用 schema、wrapper、pagination envelope 或稳定性承诺。
- [ ] CLI input、adapter output、MCP tool arguments/results、examples、fixtures 与 generated protocol material 都按对应 schema 或 contract 验证。
- [ ] Hostile source text 在进入 Markdown、terminal output、JSON fields、paths、commands、HTML 或 browser-rendered content 前已 escape 或结构化。
- [ ] Error message/log 可诊断，但不泄露 secret、credential、sensitive absolute path、stack trace 或大段 raw document body。

## Node/MCP 与工具输出（Tool Output）

- [ ] MCP bridge 仍是 thin mapping layer，只调用 `docnav`，不复制 adapter parsing、routing 或 ref interpretation。
- [ ] MCP schema 在 spawn `docnav` 前验证 arguments；bridge 在 parse/validate 前不信任 `docnav` output。
- [ ] Node process spawning 使用固定 command path、argv array、显式 cwd/env、timeout 和 stdout/stderr cap。
- [ ] Browser/tool observations 只是 evidence，不会直接变成 trusted requirements、commands、paths 或 protocol fields。

## 依赖与供应链（Dependency / Supply Chain）

- [ ] 新 dependency 有明确必要性，且检查 maintenance、provenance、install scripts、typosquats 与 lockfile impact。
- [ ] Rust 与 Node dependency checks 只在 dependency risk 或 lockfile/package files touched 时运行，并按 reachability 与 shipped exposure triage。
- [ ] Generated files、examples 与 fixtures 没有携带 secret、host-specific absolute path 或可执行 payload。

## 审查升级（Review Escalation）

- [ ] 如果 finding 可能导致 arbitrary file read/write、command execution、secret leakage、protocol confusion 或 workspace escape，至少标为 High；可被触发且影响核心 contract 时标为 Critical。
- [ ] 如果缺少能证明 abuse case 被阻断的 negative test/fixture/schema check，把 verification gap 作为 finding 或 residual risk。
- [ ] 对跨 Rust、adapter、schema、examples、docs 或 MCP 的 security-sensitive change，feasible 时要求 repository workspace verifier，并记录无法运行时的原因。
