本清单按 owner contract、validation material、core projection、navigation composition、output/logging 和 verification 的依赖顺序执行。每项完成时应同时留下对应文档、代码或测试证据；本次 artifact 优化不勾选应用阶段任务。

## 0. 用户批准门

- [x] 0.1 用户审计并明确批准更新后的 proposal、design、四个 spec deltas 和本 tasks 清单后，才进入 owner 文档或应用代码修改。

## 1. Owner contract

- [x] 1.1 更新 `docs/cli.md`：记录 `outline/find --auto-read disabled|unique-ref`、built-in default `unique-ref`、project/user config support、strict flag applicability、config inspect projection、两种 output mode、退出码和 help。
- [x] 1.2 更新 `docs/navigation-input-resolution.md`：记录 canonical identity `docnav.defaults.auto_read`、CLI locator、`defaults.auto_read` config locator、`Replace`、`explicit > project > user > built_in`、无 env locator、outline/find bindings、core-only projection、当前返回 ref 去重判定、existing read typed dispatch 和静默保留 base response。
- [x] 1.3 更新 `docs/protocol.md`：定义 success-only optional `auto_read` object、exact `reason`、nested `ReadResult`、outer operation 和 existing page ownership。
- [x] 1.4 更新 `docs/output.md`：定义 base/composed response 的 `ProtocolJson` path、成功 auto-read readable header mapping 和 `/auto_read/read/content` block。
- [x] 1.5 更新 `docs/architecture.md`：把 canonical CLI/config declaration、source resolution、navigation composition、adapter single-operation boundary、protocol owner 和 output owner 放入现有调用链。
- [x] 1.6 更新 `docs/testing.md`、coverage/case owner 和 invocation logging 说明：记录 source precedence、default-on dispatch、CLI/config disable compatibility、silent non-success、content capture 和 failure proof targets。

## 2. Validation material

- [x] 2.1 扩展 `docs/schemas/docnav-markdown-config.schema.json`：在 `defaults` 中加入 closed `auto_read` enum `disabled|unique-ref`，并保持 unknown-field rejection。
- [x] 2.2 增加或更新 config examples、`docs/examples/README.md` 和 config inspect fixture，证明 project/user `defaults.auto_read` shape、canonical identity、source candidate 和只读 inspection。
- [x] 2.3 扩展 `docs/schemas/protocol-response.schema.json`：只允许 outline/find result 的 optional closed `auto_read` success object，并校验 exact `reason`、existing `ReadResult` 和 forbidden extra fields。
- [x] 2.4 增加 protocol examples：至少覆盖 outline auto-read success、find 多 match/单 distinct ref success，以及无 unique ref 或 nested read 未成功时保持 base result；所有示例由 schema validator 消费。
- [x] 2.5 更新 protocol example index 与 schema 索引，说明 default-on success-only field、current-result unique ref、base field preservation 和 existing page continuation。
- [x] 2.6 增加 built-in renderer conformance vectors：覆盖 nested content block、无 `auto_read` 的 base projection 和 unstructured base content block。
- [x] 2.7 更新 invocation content-capture fixture；若既有 event schema 无法表达成功追加的 content，先同步 owner 文档和 schema，再实现代码。

## 3. CLI catalog 和 core projection

- [x] 3.1 在 core 定义 `disabled` / `unique-ref` typed mode，并在 `DocumentParameterCatalog` 增加 `docnav.defaults.auto_read`、CLI `--auto-read`、config `defaults.auto_read`、`Replace`、built-in `unique-ref`、无 env locator 和 outline/find bindings。
- [x] 3.2 增加 core/navigation binding，使 resolved mode 进入 core orchestration projection，同时保持 `OperationArguments`、`Options` 和 `StandardOperationInput` 无 auto-read field。
- [x] 3.3 更新 generated CLI parser/help/preflight tests：证明 exact tokens、omitted default `unique-ref`、explicit disable、duplicate/missing/invalid value 和 read/info/non-document rejection。
- [x] 3.4 增加 source resolution tests：证明 CLI > project > user > built-in、project/user config 均可选择两种 mode、selected/overridden provenance 和 invalid enum 的 source-attributed diagnostic。
- [x] 3.5 增加 operation/config boundary tests：证明 valid `defaults.auto_read` 在 read/info 中是已知但不适用的 field、config inspect 可见且只读，以及未声明 env locator 不产生 candidate。

## 4. Navigation composition

- [x] 4.1 提取可单测的 current-result unique-ref helper：对 validated `entries[].ref` 或 `matches[].ref` 做 string-exact 去重，只返回零个或一个 candidate ref，不生成 public skip reason。
- [x] 4.2 证明多个 find matches 共享同一 ref 时只产生一次 nested read，且 ref 不被解析或改写。
- [x] 4.3 证明 request page 和 response continuation 不参与 current-result ref uniqueness；later page 或 non-null continuation 上的一个 distinct ref 仍触发 read。
- [x] 4.4 在 validated base response 后复用 selected adapter typed read dispatch；使用同一 normalized path/adapter、原样 ref、read page `1` 和 existing read input semantics，不递归 CLI、不重新 selection、不执行中间 output。
- [x] 4.5 扩展 protocol result types，只构造 closed `{ reason: "unique_ref", read: ReadResult }` success object，并保持 outer operation 与 base fields。
- [x] 4.6 当 nested read 未返回 validated success 或 composed result 无法通过校验时，返回原 validated base response，不增加 public status、reason 或 error。

## 5. Output 和 invocation logging

- [x] 5.1 扩展 `docnav-output` protocol-to-readable mapping：保留 base payload，仅从 protocol `auto_read` success facts 构造 readable `auto_read`。
- [x] 5.2 扩展 renderer config/view selection，使成功 auto-read 使用 `/auto_read/read/content`；没有 `auto_read` 时不要求该 pointer，unstructured outline 继续使用 `/content`。
- [x] 5.3 保持 `ProtocolJson` 直接序列化 navigation 选择的同一个 response，并增加 stdout purity 与 no-second-dispatch 证明。
- [x] 5.4 让 invocation logging 保持 root outline/find event；可记录 bounded attempt outcome，显式 content capture 对成功追加的 content 复用 existing hash/capture event，不 inline 正文。

## 6. Behavioral tests

- [x] 6.1 增加 default/disable tests：所有来源省略时 unique ref 自动 read；CLI、project config 或 user config 解析为 `disabled` 时，protocol、readable output、dispatch count、stdout/stderr 和 exit code 保持原 base command。
- [x] 6.2 增加 outline eligibility tests：single ref、empty result、multiple refs、later request page、base continuation 和 unstructured content。
- [x] 6.3 增加 find eligibility tests：single match、multiple matches/same ref、multiple distinct refs、empty result、later request page 和 base continuation。
- [x] 6.4 增加 nested read tests：same adapter/path/ref、read page `1`、existing read input semantics、read continuation，以及 adapter diagnostic/invalid success 静默保留 base response。
- [x] 6.5 增加 composed protocol validation tests，覆盖 exact success object、forbidden status/error/extra fields、base field preservation、outer operation 和 absent-object outcomes。
- [x] 6.6 增加 readable integration tests，覆盖 nested block byte length/framing、无 auto-read 时的原 base projection、render failure 和两种 output mode 的同源 facts。
- [x] 6.7 增加 invocation logging/content capture tests，证明单 root event、bounded attempt outcome、显式 capture 和默认不记录正文。

## 7. Verification 和交付审计

- [x] 7.1 运行范围匹配的 Rust format、clippy、unit/integration、schema/example、renderer conformance 和 CLI smoke checks。
- [x] 7.2 运行 `bun run verify:docnav-workspace`，保存无 failure 的验证证据；已有 warning 必须确认无 changed regression。
- [x] 7.3 运行 `openspec validate "add-obvious-result-auto-read" --type change --strict --no-interactive`。
- [x] 7.4 用局部 diff 审计只修改本 change 及实现所需 owner surfaces；确认 CLI/config 由同一个 canonical field 拥有，且没有新增 adapter operation、protocol request argument、auto-read 专用 read 参数、public skip/failure branch、env locator 或未声明 output mode。
