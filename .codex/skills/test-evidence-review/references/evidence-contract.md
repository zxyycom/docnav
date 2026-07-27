# 测试证据目录契约

## 内容

- [责任与权威源](#责任与权威源)
- [固定布局](#固定布局)
- [NativeTestEntry 与 inventory](#nativetestentry-与-inventory)
- [Evidence Claim](#evidence-claim)
- [派生索引](#派生索引)
- [校验与诊断](#校验与诊断)
- [查询与机器接口](#查询与机器接口)

## 责任与权威源

通用模型只定义 Entry、Claim、topic 和索引契约。项目 wrapper 定义 runner profile、
静态扫描、运行时枚举、`entryKey` 归一和当前树 `sourceRevision`。

权威顺序固定为：

1. 当前源码与 runner 报告拥有入口存在性和 runner 身份。
2. Claim Markdown 拥有长期语义。
3. 受控 topic 表拥有 Claim topic。
4. inventory 和 query index 是可删除重建的派生制品。

inventory 便于离线查询和 Git 审计，但不能创建源码入口。索引不能修改 inventory
或 Claim。查询回退不得隐式写文件。

## 固定布局

工作区只通过 `workspaceRoot` 定位，目录固定为：

```text
docs/test-evidence/
├── claim-topics.json
├── claims/
│   └── <topic>/<slug>.md
├── native-test-inventory.json
└── test-evidence-index.json
```

`claims` 可在没有 Claim 时不存在。存在时只允许受控 topic 直属目录和每文件一个
Claim 的 Markdown；不接受符号链接、嵌套目录或其他成员。

## NativeTestEntry 与 inventory

每个 Entry 固定包含：

- `entryKey`：项目 wrapper 由 runner、target、selector 构造的确定性当前身份。
- `runner`、`target`、`selector`：runner 可稳定独立报告或选择的身份。
- `sourcePath`：工作区相对 POSIX 路径。
- `sourceRange`：1-based、闭开区间的 start/end line 与 column。
- `sourceFingerprint`：规范化入口 AST 或稳定 task declaration 的
  `sha256:<64 lowercase hex>`。

inventory 使用 `schemaVersion: 1`，保存 profile ID/version、当前树
`sourceRevision` 和按 `entryKey` 二进制词法排序的唯一 Entry。精确结构见
`schemas/native-test-inventory.schema.json`。

Machine case 与 Entry 一一对应，身份就是 `entryKey`；没有额外手写 case 文件或
源码 marker。Entry 重命名允许产生新 `entryKey`，长期连续性由 Claim ID 承接。

## Evidence Claim

Claim 路径是 `claims/<topic>/<slug>.md`，格式固定为：

```markdown
# Claim CLAIM-AUTH-GUEST-001: Guest mutation remains forbidden

Topic: `access-control`
Owner ref: `docs/access.md#guest-mutations`

Statement:
- Guest callers cannot mutate protected resources.

Observations:
- The mutation returns the documented forbidden error.
- The protected resource remains unchanged.

Supported by:
- `bun|tests/access.test.ts|access > rejects guest mutation`
```

规则：

1. 标题、Topic、Owner ref、Statement、Observations、Supported by 各恰好一处，
   顺序固定，不接受额外段落。
2. Claim ID 符合 `^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+){2,}-\\d{3}$`，按稳定语义
   命名且全局唯一；不把旧目录、旧 case ID 或一次性迁移来源编码进 ID。
3. topic 符合 slug 规则，存在于受控表，并与目录名一致。受控表只保留至少被一个
   当前 Claim 使用的 topic。
4. `ownerRef` 是工作区相对 Markdown 路径加非空 heading fragment；文件和 heading
   必须存在。
5. Statement、Observations、Supported by 都是非空、去重列表。`supportedBy`
   必须引用当前 inventory。
6. 已知通用模板、仅复述测试名/实现、空观察信号或未知 owner 都是阻断错误。

受控 topic 表使用 `schemaVersion: 1`，topic 按 ID 排序且 ID 唯一。精确结构见
`schemas/claim-topic-catalog.schema.json`。解析后的 Claim 结构见
`schemas/evidence-claim.schema.json`。

## 派生索引

`test-evidence-index.json` 使用 `schemaVersion: 1`，保存：

1. 当前 projection 的 `sourceRevision` 与 inventory revision。
2. 受控 topics。
3. 全部 Entry 及其反向 `claimIds`。
4. 全部 Claim、owner/source fingerprint 及其 `supportedBy`。

索引 revision 由规范化 inventory、topic 结构、Claim 正文和 owner section 内容
共同计算。任一来源变化都会使索引陈旧。owner 内容或 Claim 关联变化应另外报告
`claim-stale`，不能只归为普通 index mismatch。

## 校验与诊断

严格检查至少区分以下来源：

- `inventory`：inventory schema、排序、重复 Entry 或 revision 问题。
- `claim`：布局、topic、owner、模板、未知 Entry 和空证据问题。
- `index`：缺失、结构错误、陈旧和 `claim-stale`。
- `query`：参数或目标不存在。

项目 wrapper 另外拥有 `discovery` 和 `runner` 来源，以及 `missing-case`、
`orphan-case`、`duplicate-entry`、`static-only`、`runtime-only` 和
`unsupported-entry-shape`。通用模块不得把这些失败改写成 Claim 或 index 错误。

所有诊断提供稳定 `code`、`origin`、`severity`、`blocking`、`message`，并尽可能
附带 `path`、`entryKey` 或 `claimId`。

## 查询与机器接口

模块导出：

1. `validateTestEvidence(options)`
2. `syncTestEvidenceIndex(options)`
3. `queryTestEvidence(options)`
4. `showTestEvidence(options)`
5. `listTestEvidenceTopics(options)`
6. `runTestEvidenceCatalogCli(argv)`

CLI：

```text
node scripts/test-evidence-catalog.mjs topics --root <workspace-root> [--json]
node scripts/test-evidence-catalog.mjs check --root <workspace-root> [--json]
node scripts/test-evidence-catalog.mjs sync-index [--write] --root <workspace-root> [--json]
node scripts/test-evidence-catalog.mjs list [--kind entry|claim|all] [filters] --root <workspace-root> [--json]
node scripts/test-evidence-catalog.mjs show <entry-key-or-claim-id> --root <workspace-root> [--json]
```

`list` 支持精确 `--entry-key`、`--runner`、`--target`、`--source-path`、
`--claim-id`、`--topic`、`--owner-ref`，以及 `--query`、`--limit`、
`--offset`。默认 limit 为 20，最大 200。

`list` / `show` 在索引缺失或陈旧时构造只读内存投影并返回 warning；`check` 和
`sync-index --check` 把同一状态视为阻断。CLI 退出状态：成功 `0`，阻断或目标缺失
`1`，参数错误 `2`。`--json` 的预期失败写 stdout，stderr 保留给非结构化执行故障。
