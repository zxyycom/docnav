本 change 的目标是新增一个直接链接 ast-grep Rust crates 的多语言代码 adapter，通过 `outline -> ref -> read` 提供有限、可继续的代码结构化阅读。

本文是仅位于 `openspec/changes/add-ast-grep-code-adapter/` 的未审核临时 design，不修改或替代现有主规范、其它文档或其它 change。

## Context

当前 core release 通过 static registry 注册 linked `AdapterDefinition`，一个 manifest 已可声明多个 `formats[]`；adapter 接收 closed operation input，并拥有 parser、导航算法、ref 和结果事实。现有 protocol `Entry` 已能表达 ref、label、kind、行范围、summary 和 metadata，因此本 change 不需要扩展共享协议。

ast-grep 0.44.1 已发布可直接使用的 `ast-grep-core`、`ast-grep-language` 和 `ast-grep-outline` crates。`ast-grep-outline` 提供 `DEFAULT_OUTLINE_RULES`、`parse_outline_rules`、`CombinedExtractors` 以及带 byte range、source position、symbol type、signature 和 direct members 的 `OutlineItem`。仓库固定 Rust 1.96.0，高于该依赖当前声明的 Rust 版本下限。

参考：

- <https://docs.rs/ast-grep-outline/0.44.1/ast_grep_outline/>
- <https://docs.rs/ast-grep-outline/0.44.1/ast_grep_outline/combined_extractor/struct.CombinedExtractors.html>
- <https://docs.rs/ast-grep-outline/0.44.1/ast_grep_outline/model/struct.OutlineItem.html>
- <https://docs.rs/ast-grep-language/0.44.1/ast_grep_language/enum.SupportLang.html>

当前 active change `add-json-adapter` 也会触及 workspace、registry 和 release smoke。本 change 的语义不依赖 JSON adapter，但实现合并时必须保留双方的 linked definitions 和验证覆盖。

## Goals / Non-Goals

**Goals:**

- 在同一个 `docnav` 进程内解析 Rust、TypeScript/TSX、JavaScript/JSX 和 Python 源码。
- 将成熟的 ast-grep outline 结果转换成 Docnav 拥有的、确定性的 `Entry`、opaque ref 和 operation result。
- 保持 `outline -> ref -> read`、static registry、closed input、protocol/readable 分层和稳定错误映射不变。
- 对空文件、无可识别符号文件和存在可恢复语法错误的文件仍提供有界读取路径。
- 用 adapter、core、protocol example 和 release package 验证证明行为。

**Non-Goals:**

- parser 边界固定为 linked Rust crates，不提供 `ast-grep` CLI discovery 或 subprocess 路径。
- rules 与 parser 集合由 adapter 随 release 固定，不接受调用方提供的 YAML、pattern、language injection 或 parser plugin。
- 不提供跨文件索引、定义/引用、调用图、继承图、类型推断或增量更新。
- 不新增 CLI flag、config key、env var、protocol field、operation 或 output mode。
- 不保证 ref 在文件修改、格式映射变化或 adapter ref version 变化后仍可读取。
- 不为 ast-grep 额外设计通用 engine trait、运行时插件层或第二套 adapter contract。

## Decisions

### Decision 1: 直接链接 ast-grep Rust crates

`docnav-code` 直接依赖并精确锁定同一版本线的 `ast-grep-core = "=0.44.1"`、`ast-grep-language = "=0.44.1"` 和 `ast-grep-outline = "=0.44.1"`。实现使用 `SupportLang::ast_grep`、`DEFAULT_OUTLINE_RULES`、`parse_outline_rules` 和 `CombinedExtractors`；outline 在当前进程内完成。

影响：依赖 API 变化由 `docnav-code` 内部转换代码和 contract tests 吸收。CLI subprocess 会增加第二套运行边界，直接使用 tree-sitter 会把多语言 query 维护转移到 Docnav；两者均不作为本 change 的实现路径。

### Decision 2: 一个 linked adapter 承载多个代码格式

新增一个 `docnav-code` definition。manifest 声明 `rust`、`typescript`、`tsx`、`javascript` 和 `python` 五个 format id；`.jsx` 由 `javascript` format 和 `SupportLang::JavaScript` 处理。probe 只依据受支持扩展名返回具体 format，未知扩展名不猜测内容。

影响：core registry 只新增一个 adapter id，显式 `--adapter docnav-code` 与自动选择走现有路径；不需要修改 adapter contract 或增加每语言 adapter crate。

### Decision 3: 只编译首批语言 parser

`ast-grep-language` 关闭默认的全语言 parser feature 集，只启用 Rust、JavaScript、TypeScript/TSX 和 Python 所需 features。Cargo.lock、`cargo tree -e features`、license 检查和 release binary size 变化作为依赖接入证据。

影响：首批之外的 ast-grep built-in languages 即使有默认 outline rules 也不构成支持承诺；新增语言必须更新 code adapter spec、manifest、fixtures、features 和 release smoke。

### Decision 4: ast-grep 模型只存在于 adapter 私有解析边界

adapter 按 format 过滤内置 rules、构造 extractor、解析源码并立即把借用型 `OutlineItem` 转成 owned `CodeSymbol`。`CodeSymbol` 只保留 Docnav 需要的 name、稳定 symbol kind、role、visibility、source byte range、line range、signature 和 parent relation。转换使用私有 module 和普通函数；当前不增加 engine trait。

影响：`ast_kind`、ast-grep error 类型、rule id、YAML shape 和 crate 生命周期不会进入 protocol、ref 或共享 crate。规则解析/编译错误映射为 adapter-owned internal diagnostic，原始依赖错误只作为内部 cause，不进入稳定字段。

### Decision 5: outline 使用确定性扁平映射

每个 top-level item 后紧跟其按 source range 排序的 direct members，top-level items 按 source start、source end、kind、name 排序。相同 ref 的重复提取只保留排序后的第一项。输出映射为：

- `label`：trim 后的 symbol name；为空时使用 `<kind>@<1-based-line>`。
- `kind`：adapter-owned snake_case symbol category；import item 覆盖为 `import`，未知 future category 映射为 `other`。
- `location`：`line_start` 是 start byte 所在的 one-based 行；`line_end` 是 exclusive end byte 之前最后一个 byte 所在的 one-based 行，empty range 时等于 `line_start`。
- `summary`：signature 折叠连续 whitespace 后成为单行文本；输出最多 240 个 Unicode scalar（包含结尾 `…`），空值或与 label 相同则省略。
- `metadata.role`：`item` 或 `member`；item 另带 `exported`，member 另带 `public` 和 `parent_ref`。

如果转换后没有 symbol，outline 返回一个覆盖完整文件的 `kind: "file"` fallback entry；空文件允许 `start == end == 0`。所有分页在完整确定性 entry sequence 上使用现有 limit/page 语义。

影响：protocol shape 不变，但 code adapter 对 kind、summary 和 metadata 的映射成为自身公开 contract。依赖升级不得绕过 fixtures 静默改变这些字段。

### Decision 6: ref 使用 byte range 与内容摘要

code ref grammar 为：

```text
code:v1:<format>:<start-byte>:<end-byte>:<sha256>
```

`format` 必须是当前 adapter 支持的 format id；offset 是无多余前导零的十进制 UTF-8 byte offset，满足 `start <= end`；`sha256` 是所选原始 source bytes 的 64 位小写十六进制 SHA-256。outline 与 find 生成 ref，core 原样传递。

read 对 malformed prefix/version/field/number/hash 返回 `REF_INVALID`。grammar 合法但 format 与当前文档不符、range 越界、不是 UTF-8 boundary 或摘要不匹配时返回 `REF_NOT_FOUND`。该 grammar 不产生 `REF_AMBIGUOUS`。它有意只保证同一未修改 source 的 roundtrip，不提供 edit-stable identity。

影响：无需保存解析 session 或在 read 时重新匹配不稳定 AST identity；digest 阻止旧 range 静默读取文件修改后的错误区域。

### Decision 7: read 返回原始源码，find 只搜索符号语义

read 返回 ref range 对应的原始 UTF-8 source slice，不格式化、不补上下文，并按 format 返回稳定 content type。cost 针对分页前完整 slice 计算，content 使用现有 Unicode-safe text pagination。

find 复用 outline 的完整 normalized symbol sequence，对原始 name 与 signature 做大小写敏感 literal search；一个 symbol 最多返回一次，顺序与 outline 一致，结果 ref 和 entry facts 与 outline 相同。空 query 返回既有 invalid-request diagnostic。任何 ast-grep pattern-like 字符都按普通文本处理。

影响：结构查找保持有限且可预测；语法模式查询留给未来独立 change。

### Decision 8: info 只返回稳定文档事实

info 返回 format-specific content type、UTF-8、原文件 byte size、adapter id、format id，以及去重后、fallback 前的 `symbol_count` 和分页前实际 outline sequence 的 `outline_entry_count`。它不返回 ast-grep version、AST kind、rule id 或 parser tree。跨层验证范围由 `code-adapter` 与 `release-artifacts` specs 拥有，具体执行顺序由 tasks 拥有。

### Decision 9: additive rollout and cross-change coordination

本 change 不迁移现有数据或 ref。registry 中 code adapter 放在已有 definitions 之后，避免改变现有格式的优先选择；由于扩展名集合不重叠，正常 automatic discovery 仍由具体 probe 决定。实现若与 `add-json-adapter` 并行，必须合并 registry、Cargo workspace 和 release smoke，而不能用固定 adapter 数量覆盖另一项 change。

回滚时删除 code adapter registration、crate/dependencies、code fixtures/docs 和对应 smoke 即可；现有 Markdown、JSON（若已合并）、protocol 和用户配置不需要迁移。

## Risks / Trade-offs

- [ast-grep Rust API 与内置 rules 可变化] → 精确锁版本、私有 owned mapping、每语言 golden/contract fixtures；升级作为显式依赖变更审查。
- [静态语言 parsers 增加编译时间和 binary size] → 关闭默认全语言 features，只启用首批 parser，并记录两 target release binary delta。
- [tree-sitter 对不完整源码产生 partial tree] → 接受可恢复 symbol；无 symbol 时返回 file fallback，避免把 parser recovery 细节写入公共结果。
- [range ref 在文件编辑后失效] → digest 检测并返回 `REF_NOT_FOUND`；明确不承诺 edit stability。
- [内置 rules 对不同语言覆盖不均] → 为每语言维护代表性 fixtures；覆盖缺口作为显式限制或后续 adapter-owned rule 变更处理，不能用未记录的输出漂移绕过 spec。
- [多个 active changes 修改 registry/release smoke] → 实现时保留当前 static set，并以 adapter identity 断言代替固定总数或顺序。

## Migration Plan

1. 新增依赖与 `docnav-code` crate，先完成五个 language mappings 的 adapter tests。
2. 注册 definition，同步 catalog/inspection、owner docs、examples 和 core smoke。
3. 更新 canonical package smoke，在 Linux/Windows release target 上验证同一 binary。
4. workspace 与 package verification 通过后随正常 release 交付。
5. 回滚时移除 registry entry、code adapter crate 及其验证材料；现有 protocol、配置和其它 adapter 不需要迁移。

## Open Questions

无未回答开放问题，可以进入实现前审计。
