本 change 的核心目标是让 Clap 与 `cli-config-resolution-clap` 成为 Docnav CLI 结构解析和动态字段解码的唯一业务实现路径；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 tasks，不影响现有主规范或其它文档。

## 1. 阻塞级审计（未完成前禁止实现）

- [ ] 1.1 审计 proposal、design、specs 和 tasks：确认它们围绕同一核心目标；capability id 仅复用 `core-cli`、`cli-config-resolution`、`navigation-input-resolution`；change 只包含当前目录下的未审核临时 artifacts；未修改其它主规范；`Open Questions` 无未回答项。审计未通过前，不得执行 2.1 及后续任务。
- [ ] 1.2 将 Clap 4.6.1 focused proof 固化为可执行测试：覆盖 typed parsers、help/version outcomes、structured error context、hyphen-leading inline value、known-next-flag consumption，并证明 `ignore_errors`/passthrough 不能替代 bounded presentation probe。
- [ ] 1.3 建立变更前证据清单：记录 public command/help/error/output 行为、legacy scanner/string/JSON 路径、相关 test cases、companion API，以及 8 条 accepted CLI warnings。

## 2. 子仓库：typed Clap projection

- [ ] 2.1 按 registration、typed parser、extraction、conflict/error 和 tests 拆分 `cli-config-resolution-clap`，保持 Docnav-free dependency boundary。
- [ ] 2.2 按 design Decision 2 为全部受支持 `ValueKind` 注册 Clap action 和 typed parser，并保持 short/long 与 static/dynamic conflict checks。
- [ ] 2.3 让 extraction 只读取 typed `ArgMatches` 并生成 canonical candidates；删除 post-match numeric/object parsing 和 arbitrary JSON guessing。
- [ ] 2.4 建立 companion contract matrix，覆盖每种支持类型、string-looking-JSON、malformed value、unsupported JSON、unknown argument 和 argument conflicts。
- [ ] 2.5 更新子仓库 README/example，并通过 metadata、fmt、build、clippy `-D warnings`、all-target tests、doc tests 和 example run。

## 3. Navigation：projection 与 selected source

- [ ] 3.1 提供 operation-scoped native CLI `FieldDefSet` projection，直接复用 static registry adapter declarations。
- [ ] 3.2 在 projection validation 中拒绝 duplicate native CLI locator 和 core argument conflict，并返回可定位的 internal declaration failure。
- [ ] 3.3 将 navigation native CLI handoff 改为 canonical typed `Source`，保持现有 explicit/project/user/built-in priority。
- [ ] 3.4 Adapter 选择后执行 candidate owner/applicability filtering，并落实 design Decision 4 的错误优先级和 typed handoff。
- [ ] 3.5 增加 navigation tests，证明 registry conflict、selected/unselected candidates、source priority、config isolation、pre-dispatch failure 和 handler typed input。

## 4. Core：authoritative Clap boundary

- [ ] 4.1 接入 `cli-config-resolution-clap` 和 Clap `error-context` feature，确认 navigation crate 不依赖 Clap。
- [ ] 4.2 构造完整 root/subcommand command tree，并为每个 document subcommand 加入 navigation native projection；运行时只执行一次 authoritative parse。
- [ ] 4.3 为 core-owned arguments 注册 typed parsers，并落实 `--flag=<hyphen-leading-value>` 与 negative-number 规则。
- [ ] 4.4 从 nested typed matches 构造 command model 和 native CLI `Source`，不执行二次业务解析。
- [ ] 4.5 将 help/version outcomes 和 structured Clap/projection errors 映射到既有 PlainText、diagnostic、output channel 和 exit behavior。
- [ ] 4.6 将 preflight 收窄为 design Decision 5 定义的 operation/output-only presentation probe，并覆盖位置、inline、invalid 和 duplicate cases。
- [ ] 4.7 删除 native option catalog、`cli_arg_id()`、string command model、business scanners、JSON guess、compatibility fallback，以及整个 `docnav-cli-args` crate 和引用。
- [ ] 4.8 更新 core Rust tests，覆盖 command families、strict argv、typed native values、conflicts、错误优先级、help/version 和三种 failure presentation contexts。

## 5. 契约、测试账本与质量

- [ ] 5.1 按 owner 更新 architecture、CLI、navigation input、adapter contract 和 testing 主规范；非 owner 文档只保留摘要或引用。
- [ ] 5.2 按 testing/case-maintenance 维护证明目标、case 账本和源码 `@case` 标记；不为无自定义分支的 Clap 内建行为建立重复矩阵。
- [ ] 5.3 审计 protocol/readable schemas、examples、fixtures 和 release artifact shape；若出现可观察变化，先更新本 change 和对应 owner materials。
- [ ] 5.4 修复并移除 8 条 deferred CLI warning records，运行 full quality check，并确认没有新增抽象或 accepted warning。

## 6. 验收与交付

- [ ] 6.1 重新运行子仓库完整独立验证，要求 clean checkout、0 warning 且不依赖 Docnav crates。
- [ ] 6.2 运行主仓库 fmt、Rust tests、case consistency、core CLI smoke 和 clippy `-D warnings`，记录 changed surface 证据。
- [ ] 6.3 运行 `bun run verify:docnav-workspace:full` 和 strict OpenSpec validation，要求 0 warning、0 failed。
- [ ] 6.4 搜索并审查 legacy scanner/string/JSON/fallback/warning 残留，确认 protocol/schema/example 与 Markdown 性能未改变；随后按“子仓库提交先行、主仓库提交随后”的顺序整理并推送。
