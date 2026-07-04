本 delta 定义非结构化 outline 全文结果在共享协议类型中的形态，并定义 adapter 为非结构化全文读取和 cost threshold 提供的专用可选 hook set。

## ADDED Requirements

### Requirement: OutlineResult 支持带 kind 的结构化和非结构化形态
`docnav-protocol` MUST 将 outline success result 表达为可判别 union：普通结构化形态包含 `kind: "structured"`、`entries` 和 `page`；生效策略触发的非结构化全文形态包含 `kind: "unstructured"`、全文 `content`、`content_type`、稳定 `cost` 字段和稳定原因字段。`cost.measurements[]` MAY be empty. 非结构化全文形态 MUST NOT 包含 `entries`、`ref`、`page` 或 continuation 字段。

Stable reason values for the unstructured branch MUST include `path_rule` and `cost_threshold`.

#### Scenario: 构造非结构化 outline 成功响应
- **WHEN** 调用方使用共享协议类型构造生效策略触发的非结构化 `outline` 成功响应
- **THEN** 响应包含 `protocol_version`、`request_id`、`operation: "outline"`、`ok: true` 和 outline result
- **THEN** result 包含 `kind: "unstructured"` 并可被识别为非结构化全文形态
- **THEN** result 包含全文 `content`、`content_type`、`cost` 和稳定 reason
- **THEN** reason is `path_rule` or `cost_threshold`
- **THEN** result 不包含 `entries`、`ref`、`page` 或 continuation 字段

#### Scenario: 普通 outline 使用结构化 entries 分支
- **WHEN** outline 使用默认结构化策略
- **THEN** 成功 result 使用结构化 entries 形态
- **THEN** result 包含 `kind: "structured"`
- **THEN** 每条 entry 仍包含 adapter 生成的 `ref` 和 `display`
- **THEN** `page` 仍按普通 outline 分页规则表达是否可继续

### Requirement: Adapter 可以声明非结构化全文支持 hook set
Adapter contract MAY 支持只服务非结构化全文 outline 的可选 hook set。该 hook set MAY 包含 `unstructured_full_read` content hook、full-read cost measurement hook/declaration 和非结构化结果事实补充 hook。Selected adapter 声明 `unstructured_full_read` hook 时，navigation 在 `outline_mode = "unstructured_full"` 且跳过正常 outline handler 后 MUST 调用该 hook。Hook set MUST 只返回非结构化全文结果需要的 `content`、`content_type`、`Cost.measurements[]` 或其它稳定 result facts，MUST NOT 返回 entries、ref、page、continuation 或 readable-only wrapper。

未声明 `unstructured_full_read` hook 时，navigation MAY 使用默认 UTF-8 原文读取方案；默认方案不解析 adapter 私有 ref 或格式结构。

#### Scenario: Adapter hook 提供格式专属全文内容
- **WHEN** selected adapter 声明 `unstructured_full_read` hook
- **AND** navigation 解析出的标准 `outline_mode` 为 `unstructured_full`
- **THEN** navigation 调用该 hook 而不是正常 outline handler
- **THEN** hook result 映射为 `kind: "unstructured"` outline success result
- **THEN** mapped result 不包含 entries、ref、page 或 continuation

#### Scenario: Adapter 未声明 full-read hook 时使用默认读取方案
- **WHEN** selected adapter 未声明 `unstructured_full_read` hook
- **AND** navigation 解析出的标准 `outline_mode` 为 `unstructured_full`
- **THEN** navigation 使用默认 UTF-8 原文读取方案
- **THEN** 默认方案不解析 adapter 私有 ref 或格式结构

#### Scenario: Adapter hook set 补充非结构化结果事实
- **WHEN** selected adapter 声明非结构化结果事实补充 hook
- **AND** navigation 需要为 `kind: "unstructured"` result 补足 adapter-owned facts
- **THEN** navigation MAY call that hook as part of the unstructured full-read path
- **THEN** hook result only contributes stable result facts such as `Cost.measurements[]`
- **THEN** hook result does not introduce entries、ref、page、continuation or readable-only fields

### Requirement: Adapter 可以声明 full-read cost measurements 供 threshold 比较
Adapter contract MAY 支持 optional full-read cost measurement hook/declaration。该声明 SHOULD 列出 adapter 可以为非结构化全文路径产出的标准 cost units；hook MUST 接收 navigation 传入的 requested units，并返回对应的 `Cost.measurements[]`。这些 measurements MUST 对应非结构化全文路径实际会返回的内容。未声明 hook/declaration 时，adapter 的 full-read measurement set is empty.

Navigation owns threshold parsing, selected adapter candidate filtering, unit-level threshold merge and numeric comparison. Adapter owns format-specific cost measurement production. Navigation MUST NOT compute format-private cost to satisfy a threshold unit that is absent from selected adapter measurements.

#### Scenario: Adapter cost hook 为 threshold 提供标准 measurements
- **WHEN** selected adapter 声明 full-read cost measurement hook/declaration
- **AND** navigation has selected-adapter candidate thresholds
- **AND** navigation passes the effective requested units to the hook
- **THEN** navigation MAY call the cost measurement hook before normal outline dispatch
- **THEN** hook result contains standard `Cost.measurements[]`
- **THEN** navigation compares returned measurement values to the effective threshold values for the same units

#### Scenario: Adapter 未产出 threshold unit 时 threshold 不命中
- **WHEN** config threshold 的 `adapter` matches selected adapter id
- **AND** selected adapter full-read measurements do not contain config threshold 的 `unit`
- **THEN** navigation treats the threshold as not matched
- **THEN** navigation 不用默认 text helper 推导该 adapter 的格式私有成本
