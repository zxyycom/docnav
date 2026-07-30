# json-adapter Specification

## Purpose
TBD - created by archiving change add-json-adapter. Update Purpose after archive.
## Requirements
### Requirement: JSON adapter 必须作为静态 linked adapter 提供
`docnav-json` MUST 以 adapter id `docnav-json`、format id `json` 和 content type `application/json` 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 probe、outline、read、find 和 info strategy interface。其 public input surface MUST 等于既有 closed standard operation input；core parameter catalog、`StandardInputBinding`、CLI、env、config 和 protocol input inventory MUST 保持注册前的契约。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter
- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明 JSON format、`.json` extension 和 `application/json`
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入
- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** core public input inventory 与注册前相同

### Requirement: JSON probe 必须同时验证格式提示和文档内容
JSON probe MUST 先匹配大小写不敏感的 `.json` extension。Extension mismatch MUST 返回单个 content-conflict reason，并在文件读取前结束 probe。

Extension match 后，probe MUST 去除一个可选开头 UTF-8 BOM，执行 UTF-8 decode 和完整 JSON parse。Root depth MUST 为 `0`，最大支持 `max_depth` MUST 为 `127`。文档包含一个完整 JSON value、其后仅有 whitespace、`max_depth <= 127`，且每个 object 的 decoded member name 唯一时，probe MUST 返回 `supported: true`、format `json`、confidence `1.0`，并依次给出 extension-match 和 content-match reason。

非 UTF-8、parse failure、trailing non-whitespace、`max_depth > 127` 或重复 decoded member MUST 返回 `supported: false`、confidence `0.0` 和对应 read/content-conflict reason，selection MUST 在 strategy dispatch 前结束。若同一路径在成功 probe 后、operation reload 前变为上述任一无效状态，adapter MUST 返回 `INTERNAL_ERROR` 和 error id `json-document-changed-after-probe`；文件消失或编码变化继续使用既有 document diagnostics。

#### Scenario: 自动选择有效 JSON
- **WHEN** 未声明 adapter 且文档名为 `settings.JSON`
- **AND** 文档是 UTF-8、包含一个完整 JSON value，且每个 object 的 decoded member name 唯一
- **THEN** automatic discovery 选择 `docnav-json`
- **THEN** probe 返回 `supported: true` 和 format `json`

#### Scenario: 有歧义的 JSON 输入标记为 unsupported
- **WHEN** `.json` 文档包含 parse failure、trailing non-whitespace、`max_depth > 127` 或同一 object 中的重复 member name
- **THEN** JSON probe 返回 `supported: false`
- **THEN** selection 在 JSON operation strategy dispatch 前结束

#### Scenario: 选择后文档发生竞争修改
- **WHEN** JSON probe 成功后，同一路径在 selected operation reload 前变为 syntactically invalid、超过 depth 上限或出现重复 member
- **THEN** operation 返回 `INTERNAL_ERROR`
- **THEN** error id 为 `json-document-changed-after-probe`

### Requirement: JSON ref 必须使用 canonical ASCII-safe JSON Pointer grammar
JSON adapter MUST 生成并解析非空 ref `json:#<fragment>`，其中 `#<fragment>` 是 RFC 6901 URI fragment identifier representation。Root ref MUST 为 `json:#`。Adapter MUST 先把 object token 中 `~` 和 `/` canonical escape 为 `~0` 和 `~1`，再以 UTF-8 和大写十六进制 percent escape 编码 URI fragment 不允许的 bytes；生成的 ref MUST 不包含原始 NUL 或其它控制字符。空 object key MUST 使用 `json:#/`。当当前节点是 array 时，token MUST 为 `0` 或不带前导零的十进制索引，且 `-` 不可作为可读节点；相同 token 在 object 上仍按 member name 解释。缺少 `json:#` prefix、non-root fragment 缺少前导 `/`、非法或非 canonical percent/`~` escape，以及 array token 非 canonical MUST 返回 `REF_INVALID`；grammar canonical 但当前文档不存在的 object member 或 array index MUST 返回 `REF_NOT_FOUND`。

#### Scenario: 特殊 object key roundtrip
- **WHEN** JSON object 包含空 key、key `a/b~c` 和含控制字符或非 ASCII 字符的 key
- **THEN** outline 分别生成 root-distinct ref `json:#/`、ref `json:#/a~1b~0c` 和 canonical percent-encoded ASCII-safe ref
- **THEN** read 接收该 ref 后返回对应 value

#### Scenario: 区分非法 ref 与当前文档无匹配
- **WHEN** read 收到 `json:#/items/01` 且 `items` 是 array
- **THEN** adapter 返回 `REF_INVALID`
- **WHEN** read 收到 `json:#/object/01` 且 `object` 是包含 key `01` 的 object
- **THEN** read 返回该 object member
- **WHEN** read 收到 canonical `json:#/items/9` 但 array 没有该 index
- **THEN** adapter 返回 `REF_NOT_FOUND`

### Requirement: JSON outline 必须提供确定性扁平树导航
JSON outline MUST 对 object member 和 array element 进行 depth-first preorder 遍历，并为每个可导航 descendant 返回一个带完整 JSON ref、非空 label 和 `object|array|string|number|boolean|null` value kind 的 flat entry。Object member MUST 按源码顺序遍历，array element MUST 按 index 升序遍历。Object child label MUST 使用解码后的 member name，空 key 的正常 label MUST 为两个双引号字符 `""` 且 ref MUST 仍为 `json:#/`；array child label MUST 为 `[<index>]`。Root object/array 的 entry set MUST 由 descendants 组成，因此空 object/array MUST 返回空 entries 和 null page。Root scalar MUST 返回唯一 ref `json:#`、label `<root>` 且 kind 对应该 scalar 类型的 entry。首期 entry shape MUST 使用既有 common fields，JSON-specific metadata 和 source location 均为空。Outline MUST 使用现有 limit/page 契约分页，超长 item 截断时 MUST 保留完整 ref、最小非空 label 和分页前进；分页预算截断后没有可见的正常 label 内容可保留时，最小非空 label MUST 为 `.`，且该 fallback MUST NOT 替代空 key 的正常 label `""`。

#### Scenario: 遍历混合 JSON 树
- **WHEN** root object 含有 object、array 和 scalar descendants
- **THEN** outline 按 object member 源码顺序和 array index 顺序执行 depth-first preorder
- **THEN** container 与 scalar entry 的 kind 反映 JSON value kind
- **THEN** 每个 entry ref 都能原样传给 read

#### Scenario: 空容器没有可展开节点
- **WHEN** JSON 文档是 `{}` 或 `[]`
- **THEN** outline 返回空 entries 和 null page

#### Scenario: Root scalar 保持可导航
- **WHEN** JSON 文档的 root 是 string、number、boolean 或 null
- **THEN** outline 返回唯一 ref `json:#` 和对应 scalar kind
- **THEN** read 该 ref 返回对应 scalar

### Requirement: JSON read 必须返回指定节点的规范化 JSON
JSON read MUST 解析 JSON-owned ref，并将指定 JSON value 序列化为规范化 pretty JSON：object member MUST 保持源码顺序，container layout MUST 使用两空格缩进。每个 number MUST 使用原始 token；string escape、普通 scalar spelling 和尾随换行 MUST 使用 workspace-pinned parser/serializer 的自然结果。Read result MUST 保留输入 ref，使用 `application/json` content type，对分页前的完整 structured-read text 计算 cost，并按现有 Unicode-safe text pagination 返回 content 和下一页。当前 change 的 generic `readable-view` MUST 从这些 raw result facts 形成既有 read header 和 content block。

#### Scenario: 读取嵌套 object
- **WHEN** read 收到 outline 返回的 nested object ref
- **THEN** content 是该 object 的确定性 pretty-printed JSON
- **THEN** content type 为 `application/json`
- **THEN** cost 描述分页前的完整选中值

#### Scenario: 分页读取 Unicode string
- **WHEN** 指定 JSON value 的规范化输出超过 limit 且包含多字节 Unicode 字符
- **THEN** read 在字符边界分页
- **THEN** page 指向下一页直至内容结束

### Requirement: JSON find 必须返回可继续读取的节点
JSON find MUST 接受长度至少为一的 query，并 MUST 拒绝长度为零的 query。它 MUST 对去除一个可选开头 UTF-8 BOM 后的原文执行大小写敏感、从左到右、非重叠的 literal search；canonical pointer 和 structured-read serialization 只作为其它 operation 的结果，不扩充 find 语料。

Adapter-private source regions MUST 让每个命中确定性归属到可读取 JSON value：root region 覆盖完整 BOM-stripped source；object child region 从 member name token 开始并覆盖其 value；array child region 覆盖其 value。命中 MUST 归属完整覆盖其范围的最深 region，因此 member name 命中映射到对应 value，child region 之外的 container 结构或空白映射到最近 container，root value 外围空白映射到 root。跨越多个 child region 的命中 MUST 映射到同时覆盖它们的最近 container。

Find MUST 为每个 source occurrence 返回一个 match，并按 source offset 排序；多个 occurrence 映射到同一 ref 时 MUST 保留多个 match。每个 match MUST 带完整 ref、`kind: "match"`、从原文派生的非空 bounded excerpt label 和 source line location。现有 entry pagination MAY 截断 display facts，并在完整 ref 已耗尽预算时只保留最小非空 label；分页 MUST 保持 occurrence order 并持续前进。返回 ref 必须能原样传给 read。当前 change 的 generic `readable-view` MUST 使用这些 raw entry facts。

#### Scenario: Member name 命中映射到 member value
- **WHEN** 原文 object member name `a/b` 命中 query `a/b`
- **THEN** find 返回该 member value 的 canonical ref `json:#/a~1b`
- **THEN** read 该 ref 返回对应 JSON value

#### Scenario: 搜索保留原文 spelling
- **WHEN** string value 的原文 token 包含 `\u0061`
- **THEN** query `\u0061` 命中该 source occurrence
- **THEN** structured read 的自然 serializer spelling 不改变 find 语料

#### Scenario: 同一节点的多个源码命中分别返回
- **WHEN** 同一 scalar 或 container region 中存在两个 non-overlapping query occurrence
- **THEN** find 按 source offset 返回两个 match
- **THEN** 两个 match MAY 携带相同 ref，且该 ref 均可原样传给 read

### Requirement: JSON info 和 full-read 必须暴露稳定事实
JSON info MUST 返回 `application/json`、UTF-8、包含可选 BOM 的原文件 byte size、adapter id 和 format id。JSON-specific metadata key set MUST 精确等于 `{root_kind, node_count, max_depth}`；`root_kind` MUST 使用 `object|array|string|number|boolean|null`，`node_count` MUST 包含 root，root depth MUST 为 `0`。JSON adapter MUST 声明 unstructured full-read content 与 cost capability；full-read MUST 返回去除一个可选 UTF-8 BOM 后的原始 JSON text、`application/json` 和针对实际返回 text 的 lines/bytes/tokens cost measurements。

#### Scenario: 查看 JSON 摘要
- **WHEN** info 针对有效 JSON 文档执行
- **THEN** result 标识 `docnav-json`、format `json` 和 `application/json`
- **THEN** metadata 给出 root kind、包含 root 的 node count 和以 root 为 `0` 的 max depth

#### Scenario: 小 JSON 进入 unstructured full-read
- **WHEN** navigation policy 对 JSON 文档选择 unstructured full-read
- **THEN** adapter 返回原始 JSON text 和 `application/json`
- **THEN** result 使用 unstructured full-read content/cost shape

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界
JSON adapter 主文档、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖 probe、确定性 outline、empty-container 空 entries、root scalar entry、ASCII-safe ref grammar、空/特殊/control key roundtrip、context-sensitive array index、无损 JSON number、原文 occurrence 与 source-region-to-ref 映射、outline/find-to-read roundtrip、parse/ref errors、Unicode pagination、info/full-read、generic `readable-view`、自动与显式 adapter selection，以及同一个 release binary 中的 linked JSON behavior。

#### Scenario: 验证 JSON 产品与集成语义
- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned parse、navigation、ref、content 和 error behavior
- **THEN** core/release 验证覆盖 static selection、closed input 和单一 binary linked behavior
- **THEN** 测试从 package core executable 运行，并证明 public input inventory 与注册前相同
