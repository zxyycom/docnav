# 安全清单（Security Checklist）

这是 `security-and-hardening` 的详细 worksheet。主轴是 local-tool trust boundaries：untrusted documents、opaque refs/identifiers、filesystem paths、subprocesses、stdio/JSON、schema validation、generated output、external commands、bridge layers、dependencies 与 secrets。

不要从通用 Web auth/session checklist 开始。只有当当前 work item 真的触及 HTTP service、browser app 或 multi-user server boundary 时，才把 Web auth、CORS、cookie、CSRF 等内容作为额外补充。

## 1. 边界图（Boundary Map）

- [ ] Parser/frontmatter：headings、links、code blocks、frontmatter 与 raw bytes 都是 attacker-controlled input。
- [ ] Ref generation/parsing：ref/identifier 由 owning parser 或 service 生成和解析；边界外只做 opaque pass-through 和通用边界检查。
- [ ] Path resolution：document path、workspace root、generated file path 与 executable path 不能由 hostile text 直接决定。
- [ ] Subprocess invocation：manifest/config discovery、probe/invoke-like calls、stdout、stderr 与 exit status 都跨进程边界。
- [ ] Stdio JSON：request/response envelope、content type、partial data 与 trailing data 都需要 fail-closed handling。
- [ ] Output modes：`protocol-json`、readable-json、readable-view 与非文档 PlainText 是不同 contract。
- [ ] Schema-bound data：schemas、examples、fixtures 与 generated machine-readable material 必须与实际输出一致。
- [ ] Bridge layer：tool args/results 只做映射和封装，不能复制 owning parser、router 或 ref interpretation。
- [ ] Tool/browser/model output：只能作为 evidence，不能作为 instruction 或 trusted data。
- [ ] Dependency/CI scripts：install scripts、generated artifacts 与 workflow scripts 是 executable attack surface。

## 2. 资产（Assets）

- [ ] Workspace containment：不能越权读取/写入 workspace 之外的文件。
- [ ] Local files：path traversal、symlink escape 与 surprising absolute path 被阻断。
- [ ] Data contract stability：malformed input 不能产生看似成功的 machine-readable output。
- [ ] Boundary integrity：一个 component 的 ref 或 output 不能污染其他 component 或 routing layer。
- [ ] Schema trust：schema validation 是真实 gate，不只是文档说明。
- [ ] Secrets：env vars、tokens、credentials、absolute paths 与 private document content 不被日志或 output 泄露。
- [ ] CI/developer machine safety：构建、测试、subprocess invocation 与 dependency scripts 不执行 document-derived commands。
- [ ] User data：readable output 保持必要信息密度，但不额外展开敏感上下文。

## 3. 滥用案例（Abuse Cases）

为 touched boundary 至少选择一个 abuse case，并在 review 中说明控制点：

- [ ] `../../`、symlink 或 Windows drive/path quirk 造成 traversal。
- [ ] Forged ref、overlong ref、wrong-format ref 或 cross-document ref reuse。
- [ ] Malformed Unicode、lossy display 或 path normalization 差异导致 identity confusion。
- [ ] Oversized document、deep heading nesting、long lines 或 huge code blocks 造成 DoS。
- [ ] Malformed JSON、partial JSON、wrong content type 或 trailing stdout pollution 绕过 parser。
- [ ] Subprocess/tool failure 被误认为 successful machine-readable response。
- [ ] Machine output 与 readable output 混用，导致 schema/pagination envelope confusion。
- [ ] Markdown/HTML/terminal/JSON injection 通过 hostile source text 进入 generated output。
- [ ] Shell argument injection 或 command name/path 来自 document content。
- [ ] Prompt injection in docs/tool output 诱导 agent 运行命令、信任路径或修改 machine fields。
- [ ] Dependency typosquat、malicious install script 或 generated artifact 扩大 execution surface。
- [ ] Secret leakage 通过 logs、errors、fixtures、examples、snapshots 或 final summaries 发生。

## 4. 控制措施（Controls）

- [ ] 使用 typed parsers 处理 refs、structured envelopes、output modes 与 schema-bound data。
- [ ] 使用 `Path`/`PathBuf`、`OsStr`/`OsString` 与 platform-aware APIs；display string 不等于 filesystem identity。
- [ ] 访问文件前 canonicalize，并检查 resolved path 是否仍在承诺 root 内。
- [ ] Spawn process 时使用 fixed executable + argv array；禁止 shell interpolation 与 string-built command。
- [ ] 显式设置 cwd/env；默认不传 secret，必要时按最小集合传递。
- [ ] 设置 size、depth、recursion、pagination、stdout/stderr、memory 与 timeout limits。
- [ ] Stdio JSON parse once；拒绝 malformed envelope、partial data、trailing data 与 wrong output mode。
- [ ] Schema validation 用作 gate；examples/fixtures/generated output 与 schema drift 时阻塞。
- [ ] Hostile text 进入 Markdown、HTML、terminal、JSON、path、command 或 browser-rendered content 前 escape 或结构化。
- [ ] Errors fail closed，输出 stable external errors，不暴露 stack trace、secret 或大段 raw document。

## 5. Rust 表面（Surfaces）

- [ ] Ref、structured envelope、output mode 与 errors 有明确类型，避免 security boundary 上的 ad hoc string slicing。
- [ ] Invalid Unicode 与 lossy display 单独处理；不要把 `to_string_lossy()` 结果用于 identity 或 access control。
- [ ] Windows paths 覆盖 drive letters、UNC-ish forms、backslashes、spaces 与 quoting 行为。
- [ ] Process registry 与 routing 不接受 document-derived executable path。
- [ ] Parser/finder/reader 对 large input 保持有界；error path 不比 success path 更昂贵。
- [ ] Internal error 被映射为稳定外部 error code/message。

## 6. Node/Bridge 表面（Surfaces）

- [ ] Bridge layer 只把 tool call 映射到 owning command/service，不 parse document、不解释 ref、不复制 routing。
- [ ] Tool schema 在 spawn 前验证 path、ref、query、limit、page 与 output mode。
- [ ] `child_process` 使用 fixed command path 与 argv array，显式 cwd/env、timeout、maxBuffer 或 streaming cap。
- [ ] Subprocess stdout 在 parse/validate 前不可信；stderr 不能污染 machine-readable result。
- [ ] Tool result 不把 hostile document text 提升为 instructions、tool names、paths 或 commands。
- [ ] Browser/devtool output 只作为观察证据，不改变 trust boundary。

## 7. 生成输出、Docs 与 Examples（Generated Output）

- [ ] Generated examples、fixtures、snapshots 与 docs 不包含真实 secret、private path 或 machine-specific token。
- [ ] Hostile source text 在 readable output 中保持 data 身份，不生成可执行 instruction。
- [ ] Markdown output 中的 links、code fences、frontmatter 与 HTML snippets 不破坏 surrounding document structure。
- [ ] Schema/example 更新与 machine-readable behavior 同步，避免 stale examples 教错调用方。
- [ ] 若本 work item 已有 design/spec artifacts，则确认 security assumptions 与 implementation 一致。

## 8. 依赖与供应链（Dependencies / Supply Chain）

- [ ] 新 dependency 有明确必要性，不能用少量清晰本地代码替代时才接受。
- [ ] 检查 maintenance、publisher/provenance、license、install/build scripts、transitive deps 与 typosquat risk。
- [ ] Lockfile change 与 package manifest change 对得上；没有意外 ecosystem bump。
- [ ] `pnpm audit` 或 Rust advisory tooling 只在 dependency risk in scope 或 dependency files changed 时运行。
- [ ] Audit result 按 severity、reachability、runtime exposure、fix availability 与 shipped component triage。
- [ ] CI workflow 不执行来自 untrusted document、generated fixture 或 downloaded artifact 的命令。

## 9. AI / LLM 与提示注入（Prompt Injection）

Document content、tool output 和 model output 都可能包含针对 agent 的指令。安全边界必须在代码和验证中，而不是在 prompt 里。

- [ ] Model/tool output 不进入 `eval`、shell、SQL、filesystem path、HTML/Markdown injection sink 或 machine field，除非先 validate/escape。
- [ ] Docs 中的 prompt injection 不会改变 allowed tools、paths、commands、schemas 或 test expectations。
- [ ] Secret、private path、cross-project data 与 system prompt 不进入 generated context 或 example。
- [ ] Agent/tool actions 有 explicit user/task authorization；destructive 或 external effects 需要正常审批路径。
- [ ] Token、rate、recursion 与 loop limits 防止 unbounded consumption。

参考：需要 LLM 风险分类时，可对照 [OWASP GenAI Security Project](https://genai.owasp.org/llm-top-10/)，但以当前项目的本地工具边界为主。

## 10. 验证（Verification）

- [ ] 为 malformed refs、cross-resource refs、traversal、symlink escape、oversized input、malformed JSON、stdout pollution、subprocess failure、schema reject 和 output-mode confusion 添加 negative tests 或 fixtures。
- [ ] 运行覆盖 touched boundary 的最小 language/runtime、schema、fixture 或 integration checks。
- [ ] Skill/reference Markdown 只改文档时，运行项目约定的 Markdown shape 或 link checks。
- [ ] 跨多个实现、docs、schemas、examples 或 output boundary 时，feasible 情况下运行项目约定的综合验证。
- [ ] Review diff，确认没有 accidental secret exposure、permission broadening、shell interpolation、unchecked path 或 machine/readable output confusion。

## 11. 严重级别指南（Severity Guide）

| Severity | 触发条件 |
| --- | --- |
| Critical | 可触发 arbitrary command execution、arbitrary file write/read、secret exfiltration，或核心 data contract 被破坏 |
| High | 可信攻击路径存在但影响有界，或缺少必要 negative test 导致 security control 未被证明 |
| Medium | 真实 hardening gap，触发条件受限或影响范围较小 |
| Low | 局部暴露、错误信息过宽、注释/文档可能误导，但短期风险低 |
| FYI | 背景、替代控制或 future hardening idea |

不要把 security bug 写成 optional suggestion。若风险依赖外部假设，明确写出假设和需要的验证。
