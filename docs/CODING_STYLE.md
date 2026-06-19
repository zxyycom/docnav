# Docnav 编码规范

本文档定义 `Docnav` 实现代码的长期编码原则。它约束工程判断、边界处理、职责分层和验收标准，不替代产品规范，也不预设具体模块名、函数名或重构方案。

具体命令、协议字段、adapter 契约、ref 语义、schema 形状和测试矩阵，以 [文档导航](navigation.md) 的“如何阅读这些文档”指向的主规范为准。编码规范只回答一个问题：实现这些契约时，代码应该保持什么性质。

---

## 1. 契约优先

实现代码必须服从主规范定义的职责边界和稳定契约。

1. `docnav` 负责格式识别、adapter 路由、配置、项目初始化、adapter 管理、输出模式和错误映射。
2. 格式 adapter 负责本格式的识别、解析、导航策略、ref、分页结果和 adapter 直接输出。
3. `docnav-mcp` 的目标职责是 MCP 接入和 tool 映射，由 `implement-docnav-mcp-bridge` change 交付；不拥有文档解析、adapter 路由或 adapter 管理职责。
4. 原始协议层服务稳定校验、脚本和调试；阅读输出层服务人类和 AI 阅读。两层可以共享业务语义，但不能共享传输包装。

验收标准：新增代码可以明确指出它属于哪个制品、哪个语义层、哪个主规范所有权；不能说明所有权的代码，应先澄清边界再实现。

---

## 2. 边界显式

用户输入、CLI 参数、配置、路径、文件系统、进程、JSON、schema、MCP tool call 和 adapter 输出都属于边界。边界代码负责解析、校验、归一化和错误映射；内部逻辑只处理已经归一化的领域对象。

边界失败必须给调用方明确反馈：

1. 产品入口返回稳定错误、明确退出码或可行动诊断。
2. 验证脚本失败时说明文件、字段和原因。
3. 机器输出和诊断输出保持各自通道职责。

边界规则可以保留否定式，但必须配套正向做法：

边界：不通过空结果、默认 adapter、默认 ref、猜测 content type 或静默跳过掩盖失败。

做法：把失败转换为对应稳定错误、验证失败、候选证据或诊断信息，并保留足够的定位依据。实现可恢复兜底时，必须能追溯到主规范定义的输出层规则，不能在业务逻辑中静默丢弃失败。

验收标准：任何失败路径都能回答“调用方下一步该看什么、改什么或重试什么”。

---

## 3. 类型表达领域

跨模块传递领域含义时，代码应使用能表达业务含义和不变量的类型。裸字符串、通用 JSON 值和元组只适合协议镜像、opaque options、错误 details、脚本校验和局部 glue code。

类型设计遵循三条原则：

1. 协议镜像类型保持稳定序列化形状，不把格式私有语义塞进共享协议。
2. 业务逻辑入口接收已经校验过的领域值，不重复解析原始输入。
3. 公开 API 通过返回值表达普通失败，不因调用方可传入的普通非法值 panic。

字面量和配置遵循四条原则：

1. 稳定协议字段、operation、capability、错误码、退出码、schema 名、示例路径、固定诊断前缀和其它跨函数复用的字符串或数字，应使用枚举、新类型、常量模块或职责内配置中心表达。
2. 配置中心按职责归属拆分，例如协议常量归协议模块、SDK 输出标签归 SDK 模块、文档校验路径归验证脚本配置；不建立跨制品的全局杂物常量池。
3. 常量或配置项需要用中文注释说明来源、用途和边界，尤其要说明它来自主规范、schema、示例校验材料还是运行环境。
4. 测试 fixture、一次性局部标签和只在单个小作用域中用于构造示例的数据可以保留直写，但不能把稳定规则、跨边界字段或错误映射散落为魔法字符串。

验收标准：阅读函数签名时，可以区分“原始输入”“协议字段”“业务对象”和“格式私有参数”。

脚本代码同样适用本节。`scripts/` 和 `test/` 下跨模块传递的任务配置、schema 结果、进程执行结果、质量指标和 smoke state 应有显式 TypeScript 类型；脚本运行方式、文件扩展名和类型检查门禁由 [工程工具链](tooling.md) 拥有。

---

## 4. 输出分层

输出代码必须保持 protocol、readable-view、readable-json 和 MCP 的兼容目标不同。

1. protocol 输出保持完整 envelope、稳定字段、稳定错误 code 和必需 details。
2. `readable-view` 和 `readable-json` 必须从同一个完整的 typed readable payload 派生：`readable-json` 直接序列化该 payload；`readable-view` 在同一 JSON value 上应用仓库内 renderer config 指定的 block 替换和 framing。两种输出保持相同业务字段和值。
3. renderer config 只声明 block 字段（JSON Pointer 列表），不定义展示模板、排序或样式。稳定语义通过字段名和值、block pointer 和 UTF-8 byte length 表达；header JSON object key 顺序和多个 block section 的输出顺序不作为稳定契约。
4. 每个 operation 或 adapter 的 document output 通过共享 readable payload 进入 renderer path。renderer 按 config 声明的 JSON Pointer 只把字符串字段外置为 block；未声明字段保持 header JSON 值。
5. document output mode 只包含 `readable-view`、`readable-json` 和 `protocol-json`；help、version、config 等非文档纯文本通道使用 `PlainText` 或等价明确名称，并与 document output mode 类型分离。
6. MCP structuredContent 使用 readable shape；TextContent 使用精简阅读文本，其渲染任务消费本仓库的 readable-view contract、renderer config 和 conformance vectors。
7. 用户可配置文案只能影响阅读文本、guidance、usage 和包装文案，不能改变稳定机器字段。renderer config（block 字段声明和 framing 规则）是仓库内代码契约，不受用户配置、项目配置、环境变量或 CLI flag 控制。

边界：protocol envelope 不进入 readable-view header、readable JSON、MCP structuredContent 或 TextContent。

做法：在入口转换层显式完成 protocol result 到 typed readable payload 的映射；readable-view 和 readable-json 在同一个 payload JSON value 上分流。用 schema、conformance vectors 或测试验证字段集合一致和 block payload 还原。

验收标准：同一业务结果在 readable-view 和 readable-json 中语义一致（除 block 表示形式外）；不同输出层包装字段、稳定性承诺和消费对象清晰不同。

---

## 5. Ref 与分页保持格式所有权

ref 是 adapter 生成和解释的非空 opaque string。共享协议、`docnav`、MCP 和其它接入层只校验 ref 是非空字符串、原样传递且不解析 ref 内部结构。ref 的 grammar、定位语义、唯一性、稳定性和错误分类由 adapter 专属契约定义。

分页使用有限结果和可继续位置表达，不通过无限输出、隐式截断或隐藏状态完成。

验收标准：

1. `outline -> ref -> read` 链路中，ref 可以原样从 outline 进入 read。
2. ref 非法 grammar、无匹配等失败由 adapter 按其专属契约转换为稳定错误。
3. page 和 limit 的处理可以通过输出字段和测试观察，不依赖调用方猜测内部状态。

---

## 6. 职责聚合受控

文件、模块、函数和脚本应围绕单一变化原因组织。编码规范不规定固定目录方案；具体拆分应服从当前代码形状和相邻模块惯例。

需要拆分的信号：

1. 同一文件同时承载多个制品职责、多个语义层或多个边界类型。
2. 新增功能需要反复修改无关代码。
3. 测试、fixture、解析、输出和错误映射互相遮蔽，难以局部阅读。
4. 代码评审无法用一个简短句子说明该单元的主职责。

验收标准：新增或修改的代码有清晰所有权，局部改动不会迫使读者理解无关制品或无关语义层。

---

## 7. 规则来源单一

协议字段、错误码、必需 details、schema 字段形状、capability、输出层语义和 adapter 管理规则必须能追溯到主规范或 schema。示例和验证脚本用于校验，不成为新的规则来源。

当契约变化时，同步检查：

1. 主规范是否更新。
2. schema 是否更新。
3. 示例是否覆盖新语义。
4. Rust/JS 类型和语义校验是否一致。
5. 验证脚本是否仍只是在校验规则，而不是重新定义规则。

验收标准：任何稳定字段或稳定错误的变化，都能在规范、schema、示例和实现之间找到一致映射。

---

## 8. 测试按风险选层级

测试范围随风险和影响面扩展。窄改动用局部测试验证不变量；跨边界改动必须覆盖调用链、输出层和错误映射；契约改动必须覆盖 schema、示例和语义校验。

最低要求：

1. 内部语义有单元测试。
2. 入口行为有集成测试或等价验证。
3. schema 和示例保持可编译、可解析、可互相映射。
4. 输出层边界和 stderr/stdout 边界有测试或脚本检查。
5. 无法运行的验证必须在交付说明中明确原因和风险。

验收标准：测试能证明本次改动影响的边界仍成立，而不是只证明代码可以编译。

---

## 9. 变更前后自检

提交或交付前检查：

1. 职责边界是否仍符合文档导航指向的主规范。
2. 边界失败是否显式反馈，没有静默兜底。
3. 公开 API 是否通过类型或返回值表达不变量和失败。
4. protocol、readable-view、readable-json 和 MCP 输出是否保持分层；readable-view 和 readable-json 是否从同一 typed payload 派生。
5. ref 是否仍由 adapter 拥有，接入层只原样传递。
6. 稳定规则是否只有一个来源，并同步到 schema、示例、代码和验证。
7. 用户可见文案是否与稳定机器字段分离。
8. 模块职责是否清晰，未把新功能堆进无关入口。
9. 验证命令是否按改动范围运行并记录结果。
