本 delta 的目标是让 release package 验收 protocol output 与默认 rendered output，而不改变运行时语义；当前文档只在 `openspec/changes/unify-output-with-injected-rendering/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: 统一制品目录不得改变运行时输出协议

发布制品目录结构 MUST preserve `docnav --output protocol-json` envelope semantics、默认 `readable-view` renderer behavior 和 core output-plan selection。Release packaging facts MUST remain outside protocol fields、operation results、render context and rendered text。

#### Scenario: 从 package 执行 protocol-json

- **WHEN** 调用方运行 package 中的 `docnav --output protocol-json`
- **THEN** stdout 输出完整原始协议 envelope
- **THEN** 制品目录信息不进入协议字段

#### Scenario: 从 package 执行默认 rendered output

- **WHEN** 调用方从 package 运行 document operation 且省略 output mode
- **THEN** core composition 使用 release 中的内置 `readable-view` renderer
- **THEN** 制品目录信息不进入 render context 或 rendered text
