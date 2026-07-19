**一句话核心：定义内置 JSON adapter 的真实产品语义，同时保持 core 静态注册、closed input 和 adapter-owned ref 边界不变；本文件是仅位于本 change 目录下的未审核临时新 capability，不影响现有主规范或其它 change。**

## ADDED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供
`docnav-json` MUST 以 adapter id `docnav-json`、format id `json` 和 content type `application/json` 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。它 MUST 实现固定的 probe、outline、read、find 和 info strategy interface，并 MUST NOT 声明 caller-configurable 参数或增加 core parameter catalog、`StandardInputBinding`、CLI、env、config 或 protocol input。

#### Scenario: Core 检查内置 JSON adapter
- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明 JSON format、`.json` extension 和 `application/json`
- **THEN** 发布包不包含独立 JSON adapter executable

#### Scenario: JSON 使用既有公共输入
- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 只接收对应的 closed standard operation input
- **THEN** `max_heading_level` 等 Markdown-scoped 值不适用于 JSON
- **THEN** 注册 JSON adapter 本身不扩大 core 接受的 public 参数面

### Requirement: JSON probe 必须同时验证格式提示和文档内容
JSON probe MUST 对大小写不敏感的 `.json` extension 执行 UTF-8 decode 和完整 JSON parse。只有 extension 匹配、文档包含一个完整 JSON value、其后仅有 whitespace 且对象内不存在重复 member name 时，probe 才能返回 `supported: true`、format `json` 和确定性 reasons。Extension 不匹配、非 UTF-8、parse failure、trailing non-whitespace 或重复 member name MUST 返回 `supported: false` 和对应 conflict/read reason，不得进入 JSON operation strategy。

#### Scenario: 自动选择有效 JSON
- **WHEN** 未声明 adapter 且文档名为 `settings.JSON`
- **AND** 文档是 UTF-8、包含一个完整 JSON value 且没有重复 object member
- **THEN** automatic discovery 选择 `docnav-json`
- **THEN** probe 返回 `supported: true` 和 format `json`

#### Scenario: 拒绝有歧义的 JSON 输入
- **WHEN** `.json` 文档包含 parse failure、trailing non-whitespace 或同一 object 中的重复 member name
- **THEN** JSON probe 返回 `supported: false`
- **THEN** JSON operation strategy 不接收该文档

### Requirement: JSON ref 必须使用 canonical JSON Pointer grammar
JSON adapter MUST 生成并解析非空 ref `json:<pointer>`，其中 `<pointer>` 是 RFC 6901 JSON Pointer；root ref MUST 为 `json:`，object token 中 `~` 和 `/` MUST 分别 canonical escape 为 `~0` 和 `~1`。数组 token MUST 为 `0` 或不带前导零的十进制索引，且 `-` 不可作为可读节点。缺少 `json:` prefix、pointer 缺少前导 `/`、非法 escape 或数组 token 非 canonical MUST 返回 `REF_INVALID`；grammar 合法但当前文档不存在的 object member 或 array index MUST 返回 `REF_NOT_FOUND`。

#### Scenario: 特殊 object key roundtrip
- **WHEN** JSON object 包含 key `a/b~c`
- **THEN** outline 为该 member 生成 ref `json:/a~1b~0c`
- **THEN** read 接收该 ref 后返回对应 value

#### Scenario: 区分非法 ref 与当前文档无匹配
- **WHEN** read 收到 `json:/items/01` 且 `items` 是 array
- **THEN** adapter 返回 `REF_INVALID`
- **WHEN** read 收到 canonical `json:/items/9` 但 array 没有该 index
- **THEN** adapter 返回 `REF_NOT_FOUND`

### Requirement: JSON outline 必须提供确定性扁平树导航
JSON outline MUST 对 object member 和 array element 进行 depth-first preorder 遍历，并为每个可导航 descendant 返回一个带完整 JSON ref、非空 label 和 value kind 的 flat entry。Object member MUST 按 key 的 Unicode code point 升序遍历，array element MUST 按 index 升序遍历。Root object/array 有 descendant 时 MUST 不额外加入 root entry；root scalar 或空 object/array MUST 返回唯一的 `json:` fallback entry。Outline MUST 使用现有 limit/page 契约分页，超长 item 截断时 MUST 保留完整 ref、最小非空 label 和分页前进。

#### Scenario: 遍历混合 JSON 树
- **WHEN** root object 含有 object、array 和 scalar descendants
- **THEN** outline 按 object key 排序和 array index 顺序执行 depth-first preorder
- **THEN** container 与 scalar entry 的 kind 反映 JSON value kind
- **THEN** 每个 entry ref 都能原样传给 read

#### Scenario: 空容器使用 root fallback
- **WHEN** JSON 文档是 `{}` 或 `[]`
- **THEN** outline 返回唯一 ref `json:`
- **THEN** read 该 ref 返回对应空容器

### Requirement: JSON read 必须返回指定节点的规范化 JSON
JSON read MUST 解析 JSON-owned ref，并将当前文档中指定 JSON value 序列化为确定性的 pretty-printed JSON；object key 顺序 MUST 与 outline 的确定性顺序一致。Read result MUST 保留输入 ref，使用 `application/json` content type，对完整选中值计算 cost，并按现有 Unicode-safe text pagination 返回 content 和下一页。Read 不承诺保留原文 whitespace、数字拼写或 object member 源码顺序。

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
JSON find MUST 拒绝空 query，并按 outline 的确定性遍历顺序对每个节点执行大小写敏感的 literal search。Search corpus MUST 包含节点的 canonical JSON Pointer；scalar 节点还 MUST 包含其规范化 scalar text。一个节点无论 pointer 与 scalar text 命中多少次都 MUST 最多返回一个 match；match MUST 带该节点完整 ref、`kind: "match"` 和包含 pointer 及有界 scalar preview 的非空 label。Find MUST 使用现有 limit/page 契约分页，返回 ref 必须能原样传给 read。

#### Scenario: 按 key path 查找节点
- **WHEN** query 命中某节点 canonical pointer 中的 object key
- **THEN** find 返回该节点 ref
- **THEN** read 该 ref 返回对应 JSON value

#### Scenario: 按 scalar 内容查找节点
- **WHEN** query 命中 string、number、boolean 或 null 的规范化 scalar text
- **THEN** find 最多为该节点返回一个 match
- **THEN** 多个匹配节点按确定性树顺序返回

### Requirement: JSON info 和 full-read 必须暴露稳定事实
JSON info MUST 返回 `application/json`、UTF-8、原文件 byte size、adapter id、format id，以及稳定 metadata `root_kind`、`node_count` 和 `max_depth`；`node_count` MUST 包含 root，root depth MUST 为 `0`，且 metadata 不得暴露 parser-private structures。JSON adapter MUST 声明 unstructured full-read content 与 cost capability；full-read MUST 返回去除可选 UTF-8 BOM 后的原始 JSON text、`application/json` 和支持的 lines/bytes/tokens cost measurements，而不得把规范化 read 表示替代原文。

#### Scenario: 查看 JSON 摘要
- **WHEN** info 针对有效 JSON 文档执行
- **THEN** result 标识 `docnav-json`、format `json` 和 `application/json`
- **THEN** metadata 给出 root kind、包含 root 的 node count 和以 root 为 `0` 的 max depth

#### Scenario: 小 JSON 进入 unstructured full-read
- **WHEN** navigation policy 对 JSON 文档选择 unstructured full-read
- **THEN** adapter 返回原始 JSON text 和 `application/json`
- **THEN** result 不包含 structured outline entries 或 JSON ref

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界
JSON adapter 主文档、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖 probe、确定性 outline、root fallback、ref grammar、outline/find-to-read roundtrip、parse/ref errors、Unicode pagination、info/full-read、自动与显式 adapter selection，以及同一个 release binary 中的 linked JSON behavior。

#### Scenario: 验证 JSON 产品与集成语义
- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned parse、navigation、ref、content 和 error behavior
- **THEN** core/release 验证覆盖 static selection、closed input 和单一 binary linked behavior
- **THEN** 测试不依赖独立 JSON adapter executable 或新增 public 参数
