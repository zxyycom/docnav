本 change 的目标是新增一个直接链接 ast-grep Rust crates 的多语言代码 adapter，通过 `outline -> ref -> read` 提供有限、可继续的代码结构化阅读。

本文是仅位于 `openspec/changes/add-ast-grep-code-adapter/` 的未审核临时 tasks，不修改或替代现有主规范、其它文档或其它 change。

## 1. 阻塞级实现前审计

- [x] 1.1 完成阻塞审计；审计完成前不得执行任何 2.x 及后续实现任务。结论（2026-07-20）：proposal、design、两份 delta specs 和 tasks 均围绕核心句；`code-adapter` 是稳定新 owner，`release-artifacts` 复用准确；本 change 只包含自身目录下的未审核临时 artifacts，未修改其它 docs/specs/changes；`## Open Questions` 无未回答问题或歧义。门禁解除。

## 2. 实现基线与依赖接入

- [ ] 2.1 在门禁完成后读取 `docs/coding-style.md`、`docs/testing.md`、`docs/testing/case-maintenance.md` 以及 architecture/adapter/ref/release owner 文档，记录本 change 的证明目标、最小验证命令和与 active `add-json-adapter` 的文件重叠。
- [ ] 2.2 在 workspace 中精确锁定互相兼容的 `ast-grep-core`、`ast-grep-language`、`ast-grep-outline` 版本，关闭全语言默认 features，只启用 Rust、JavaScript、TypeScript/TSX 和 Python parser，并用 `cargo tree -e features` 验证 feature closure。
- [ ] 2.3 新增 `crates/adapters/code` workspace crate 和最小 public definition factory，接入既有 adapter contracts、protocol、diagnostics、text-cost 与 SHA-256 helper，不增加通用 engine trait 或新的 shared crate。
- [ ] 2.4 完成新增依赖的 license、来源、Rust 1.96.0 compatibility 和 release binary size 变化检查，并把证据放入既有质量/发布 owner。

## 3. 格式、parser 与私有模型

- [ ] 3.1 实现 extension、format id、content type 与 `SupportLang` 的 closed mapping，以及 `docnav-code` manifest/probe tests；证明一个 definition 暴露五个 formats、未知 extension 被拒绝且无 caller-configurable 参数。
- [ ] 3.2 实现按语言过滤 `DEFAULT_OUTLINE_RULES`、构造/复用 `CombinedExtractors` 和进程内 `SupportLang::ast_grep` 的私有解析路径，并用 tests 证明初始化失败映射为稳定 adapter diagnostic。
- [ ] 3.3 实现 owned `CodeSymbol` 转换及五个 language fixtures；覆盖 imports、functions、types/classes、direct members、Unicode 标识符和 recoverable incomplete syntax，并证明 ast-grep 类型与原始 error 不越过 adapter 边界。

## 4. Ref 与确定性 outline

- [ ] 4.1 实现 `code:v1:<format>:<start-byte>:<end-byte>:<sha256>` canonical formatter/parser，并覆盖 `REF_INVALID`、`REF_NOT_FOUND`、Unicode boundary 和 stale digest。
- [ ] 4.2 实现 kind/import、label、exclusive byte range 到 inclusive one-based location、240-scalar summary、item/member metadata 和 parent ref 映射，并覆盖所有 public fields。
- [ ] 4.3 实现 top-level/member 排序、相同 ref 去重、symbol-free/empty fallback 和现有 limit/page 分页；用重复运行、多页和 outline-to-read tests 证明确定性与可继续性。

## 5. Read、find 与 info operations

- [ ] 5.1 实现 read 的 format/range/boundary/digest 校验、原始 source slice、content type、完整区域 cost 和 Unicode-safe pagination；覆盖保真、多页、fallback、malformed/mismatched/stale ref。
- [ ] 5.2 实现 find 的 name/signature 大小写敏感 literal search、空 query diagnostic、outline-order result 和 entry/ref 复用；覆盖去重、分页、Unicode、pattern-like literal 与 match-to-read。
- [ ] 5.3 实现 info 的 UTF-8、byte size、adapter/format、fallback 前 `symbol_count` 和分页前 `outline_entry_count`；覆盖 normal、partial syntax、symbol-free 与 empty source。

## 6. Core、输出与可观察契约集成

- [ ] 6.1 把 `docnav-code` definition 加入 core static registry 和已知 adapter catalog，保留当前 Markdown、JSON（若已合并）及其它 definitions，不使用固定 adapter 总数或覆盖并行 change。
- [ ] 6.2 增加 adapter list、automatic/explicit selection、invocation logging 和每种 format 的 core CLI tests；证明 closed input、现有 error mapping、linked dispatch，且日志不泄露源码或 parser internals。
- [ ] 6.3 增加 protocol-json/readable-view integration tests 与 code outline/read/find/info examples，证明现有 envelope/wrapper/schema 可直接承载新增行为；若出现 schema mismatch，先更新本 change 的 contract 决策再修改 schema。

## 7. Owner 文档与测试账本

- [ ] 7.1 新增 code adapter 主文档，记录读取时机、formats、outline mapping、ref grammar、read/find/info、partial parse、错误、非目标和验证入口，并从 `docs/navigation.md`、adapter/ref owner 的适当位置建立单一所有权链接。
- [ ] 7.2 按 `docs/testing/case-maintenance.md` 为新增/修改测试分配 case ids、源码 `@case` 标记、case ledger 条目和 coverage mapping，逐项写明所证明的 requirement/scenario。
- [ ] 7.3 更新用户可见的 adapter/format 支持说明和示例索引，并运行文档导航、链接及 schema/example 局部验证，确认 consumer 文档不暴露 ast-grep API 或内部 rule shape。

## 8. Release artifact 验证

- [ ] 8.1 更新 canonical package smoke，从 `package/manifest.json` 运行同一个 `docnav`，覆盖 Rust、TypeScript、TSX、JavaScript 和 Python 的 outline-to-read，并保留当前其它 adapters 的代表性 roundtrip。
- [ ] 8.2 增加 package file-set 与隔离环境断言，证明制品不包含 `ast-grep`/`docnav-code`/language parser executable，且在无法发现或执行外部 ast-grep 时 code operations 仍成功。
- [ ] 8.3 在 Linux 与 Windows 支持 target 上构建/验证 canonical package，记录 feature closure、构建结果和变更前后 core binary size，不以 Cargo target binary 替代 package 验收对象。

## 9. 最终验证与交付审计

- [ ] 9.1 对变更范围运行 Rust format、targeted unit/integration tests、Clippy 和 protocol/schema/example checks，修复全部失败并保留命令证据。
- [ ] 9.2 运行 `bun run verify:docnav-workspace` 及 required release/package checks，证明跨 adapter、protocol、docs、OpenSpec 和 release 边界一致。
- [ ] 9.3 运行 `openspec validate add-ast-grep-code-adapter --type change --json --strict --no-interactive`，确认 artifacts 与实现证据一致。
- [ ] 9.4 审查局部 diff、依赖与 public surface，确认没有外部 executable path、用户 rule/pattern input、AST internals、无关重构、重复 abstraction 或其它 active change 回归，再准备实现验收。
