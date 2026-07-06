本 delta spec 定义 Markdown adapter 的 md-like `readable-view` 输出；当前文档只在 `openspec/changes/add-adapter-readable-view-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Markdown readable-view 必须提供 md-like 文本投影
`docnav-markdown` MUST provide an adapter readable-view renderer for successful `outline`, `read`, `find`, and `info` operations. This is a Markdown-specific renderer choice and MUST NOT be interpreted as a generic requirement for all adapter readable-view renderers. The renderer MUST produce Markdown-like UTF-8 text that preserves the operation's selected content, structure or matches in document order where applicable. The renderer MAY be lossy and MAY insert explicit omission markers; those markers MUST represent Markdown readable-view projection omissions rather than original Markdown content. The renderer MUST keep ref and continuation guidance discoverable when the operation result contains navigable refs or a next page.

#### Scenario: Outline renders a Markdown-like skeleton
- **WHEN** `docnav outline <markdown-path> --output readable-view` succeeds through the Markdown adapter
- **THEN** stdout is Markdown-like text rather than a protocol envelope
- **THEN** visible headings are rendered in document order with Markdown heading shape such as `#`, `##`, or equivalent readable indentation
- **THEN** each navigable visible heading keeps its complete ref discoverable near the heading text
- **THEN** omitted siblings, children, body content or deeper outline detail are represented by explicit omission markers

#### Scenario: Outline with no visible heading keeps full document ref
- **WHEN** Markdown outline returns the adapter-owned full document ref because current outline parameters expose no headings
- **THEN** Markdown readable-view output includes a readable full-document entry
- **THEN** the `doc:full` ref remains discoverable
- **THEN** any omission marker only describes readable-view projection omissions

#### Scenario: Read renders selected Markdown content
- **WHEN** `docnav read <markdown-path> --ref "<ref>" --output readable-view` succeeds through the Markdown adapter
- **THEN** stdout renders the selected Markdown content or current page in Markdown-like form
- **THEN** the selected ref remains discoverable
- **THEN** if page boundaries omit content before or after the current text, stdout contains explicit omission markers
- **THEN** if a next page is available, continuation guidance remains discoverable

#### Scenario: Find renders match context in Markdown-like order
- **WHEN** `docnav find <markdown-path> --query "<query>" --output readable-view` succeeds through the Markdown adapter
- **THEN** stdout renders match contexts in document order
- **THEN** each match keeps its complete ref discoverable near the matching context
- **THEN** non-contiguous regions between match contexts are separated by explicit omission markers
- **THEN** if more matches or a next page are available, continuation guidance remains discoverable

#### Scenario: Info renders Markdown-readable summary
- **WHEN** `docnav info <markdown-path> --output readable-view` succeeds through the Markdown adapter
- **THEN** stdout renders a concise Markdown-readable summary
- **THEN** the summary includes Markdown format identity or content type facts from the operation result
- **THEN** stdout is not required to preserve JSON header fields or block framing

#### Scenario: Markdown readable-view does not change machine outputs
- **WHEN** the same Markdown operation is rendered as `readable-view`, `readable-json`, and `protocol-json`
- **THEN** Markdown readable-view MAY contain md-like omissions or context reshaping
- **THEN** `readable-json` keeps the readable success schema fields
- **THEN** `protocol-json` keeps the protocol response envelope and result shape
- **THEN** ref and pagination semantics remain owned by the Markdown operation result

## MODIFIED Requirements

### Requirement: Markdown adapter 必须通过 core CLI 黑盒 smoke 测试
Core CLI smoke MUST cover linked Markdown adapter behavior through the `docnav` executable. 测试必须启动构建后的 core binary，并通过真实进程边界传入 argv、cwd 和环境；Markdown adapter implementation source MUST come from the current core release static registry。核心 fixtures 必须是提交到项目中的固定文件。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、core adapter inspection、CLI help、linked adapter dispatch 和 strict argv failure path。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Strict input 行为：unknown argv、多余 positional、operation-inapplicable 参数和 undeclared native options 返回 primary `DiagnosticRecord` 投影。
- Readable-view success：Markdown adapter-rendered md-like text for outline、read、find and info, including ref discoverability, omission marker visibility, continuation guidance where applicable, and absence of protocol envelope.
- Machine output stability：`readable-json` and `protocol-json` keep their owning success schema or protocol envelope shapes for the same fixtures.

#### Scenario: Strict argv failure 被覆盖
- **WHEN** smoke 测试执行 `docnav outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav outline --unknown <path> --output readable-view`
- **OR** 执行 `docnav outline <path> --unknown --output protocol-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令返回 strict input failure
- **THEN** linked Markdown handler 不执行
- **THEN** failure output 投影 primary `DiagnosticRecord`

#### Scenario: 成功 smoke 使用对应 output mode contract
- **WHEN** smoke suite 检查有效 `readable-json`、`readable-view` 和 `protocol-json` 输出
- **THEN** Markdown adapter behavior 通过 core-linked adapter dispatch 观察
- **THEN** `readable-json` successful output 使用 owning success schema fields
- **THEN** `readable-view` successful output uses Markdown md-like text with discoverable refs or continuation guidance where applicable
- **THEN** protocol JSON 输出符合 protocol response envelope 结构

#### Scenario: Markdown readable-view smoke covers operation-specific omissions
- **WHEN** smoke suite checks Markdown `outline`, `read`, and `find` in `readable-view`
- **THEN** outline output shows heading-shaped structure and omission markers for hidden detail when applicable
- **THEN** read output shows selected Markdown content and page omission markers when applicable
- **THEN** find output shows match contexts separated by omission markers when non-contiguous regions are omitted
