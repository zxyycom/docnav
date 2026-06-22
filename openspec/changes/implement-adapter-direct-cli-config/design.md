本 design 说明 adapter direct CLI 如何通过 SDK 解析可覆盖的配置文件路径，并把配置内容合并为标准 direct CLI 调用参数；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 当前已经有 core `docnav` 配置实现，但 adapter direct CLI 仍只使用静态默认值和 argv。规范上每个 CLI 拥有独立配置域，且 `docnav-markdown` 的 `limit_chars` 和 `max_heading_level` 默认值属于 adapter 配置域；实现上这条链路缺失。

本 change 建立 adapter direct CLI 的通用配置读取基础设施，让具体 adapter 可以复用 SDK 提供的默认项目/用户配置路径，并允许调用方覆盖这些路径。SDK 读取配置文件后，把配置值与 argv 合并成标准 direct CLI operation 参数，再进入既有 request 构造和 operation handler。

## Goals / Non-Goals

**Goals:**

- 为 adapter direct CLI 支持项目级和用户级 JSON 配置，并把两者的配置文件路径作为可覆盖的 SDK-owned standard 参数。
- 统一配置优先级：显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值。
- 支持通用 direct CLI 默认值 `defaults.limit_chars`、`defaults.output`。
- 支持 adapter native options 配置，并通过 `NativeOptionSpec` 做 key、operation 和 value 校验。
- 在 SDK 内部把配置文件解析结果合并为标准 direct CLI operation 参数，避免每个 adapter 重建合并逻辑。
- 让 `docnav-markdown` 读取 `docnav-markdown.json`，支持 `options.max_heading_level`。
- 保持 adapter `invoke` stdin JSON 严格，不读取配置。

**Non-Goals:**

- 不让 core `docnav` 读取 adapter 专属配置。
- 不从 adapter manifest 合成默认参数或 native options。
- 不改变 protocol request/response 字段 shape。
- 不支持 JSON 之外的配置格式。
- 不实现 feature-specific 配置项，如 unstructured outline 或 frontmatter；这些由后续 feature change 使用本基础设施。

## Decisions

1. **配置读取放在 `docnav-adapter-sdk` direct CLI 层。**

   SDK 已经拥有 direct CLI argv、native option、输出模式和 operation request 构造。把配置读取放在这里，可以让所有 Rust adapter direct CLI 共享优先级、错误映射和测试。

   备选方案是每个 adapter 自行读取配置。该方案短期简单，但会导致 `docnav-markdown` 和后续 adapter 重复实现项目根发现、用户配置路径、JSON 校验和 native option 合并。

2. **配置路径是标准 direct CLI 参数，默认值可覆盖。**

   SDK 为每个 adapter direct CLI 计算两个默认配置路径：项目级默认是当前项目根下 `.docnav/<adapter-id>.json`，用户级默认是用户配置目录下 `<adapter-id>.json`。Direct CLI 使用 `--project-config-path <path>` 和 `--user-config-path <path>` 覆盖这两个路径；相对覆盖路径按启动 cwd 解析。路径参数只影响配置加载，不进入 protocol request、native options 或 operation handler。

   备选方案是把路径发现完全硬编码在 SDK 内部。该方案少暴露参数，但测试、临时配置和嵌入式调用更难控制，也会让后续工具难以指定隔离配置。

3. **只支持 JSON 配置文件。**

   本 change 读取两个解析后路径指向的 JSON 文件。用户级默认路径复用 core 当前的 `DOCNAV_CONFIG_DIR`、平台配置目录或等价 discovery 规则，但文件名替换为 adapter id；覆盖路径可指向任意可访问 JSON 文件。

   备选方案是立即支持 `.toml`、`.yaml` 或 glob 扫描。该方案扩大 parser 和错误面，且当前 core 实现已经使用 JSON。

4. **配置只影响 direct CLI document operations。**

   `docnav-markdown outline/read/find/info` 等 direct CLI 文档命令读取配置。`manifest`、`probe`、`help` 和 `invoke` 不读取 document operation 配置；`invoke` 只消费 stdin 中已显式给出的 protocol request。

   这样保持 process boundary：core `docnav` 通过 adapter `invoke` 调用 adapter 时，不会被 adapter direct CLI 配置隐式改变。

5. **配置内容先合并为标准 direct CLI 参数。**

   SDK 解析 argv 后先得到配置路径、显式业务参数和显式 native options；随后读取用户级与项目级配置，把配置字段合并进标准 direct CLI 参数模型。配置只贡献 `limit_chars`、`output` 和 adapter native options；`path`、`ref`、`query` 来自 argv，`page` 来自 argv 或入口固定默认 `1`。后续 operation request 构造只消费这个标准模型。

   备选方案是在每个 operation request 构造点分别读取配置。该方案会让 outline/read/find/info 的优先级实现分散，且容易漏掉 native option 适用性检查。

6. **native options 使用 `options.<key>` 命名。**

   通用 direct CLI config shape 为：

   ```json
   {
     "defaults": {
       "limit_chars": 6000,
       "output": "readable-view"
     },
     "options": {
       "max_heading_level": 3
     }
   }
   ```

   SDK 使用 adapter 注册的 `NativeOptionSpec` 校验 `options` key 和 value。只有当前 operation 适用的 native option 会写入最终 `arguments.options` 或等价语义输入。

7. **配置错误按 document output mode 渲染。**

   Direct CLI 在 document operation handler 执行前发现配置文件 JSON 无法解析、未知 key、非法 value、配置路径不可读或不支持的 output 时，返回输入/config 错误，并且不执行 operation handler。输出形态由已确定的 document output mode 决定：显式 `--output protocol-json` 时 stdout 是 protocol failure envelope，使用 `INVALID_REQUEST`，`details.field` 指向配置路径或配置 key，`details.reason` 给出失败原因；显式 `--output readable-json` 或配置已确定 `readable-json` 时 stdout 是 readable error JSON；其它 document operation 配置错误使用默认 `readable-view` error framing。配置错误不得退化为只写 stderr 的协议外错误，除非 stdout 写入本身失败。

## Risks / Trade-offs

- [Risk] Direct CLI 配置和 core `docnav` 配置规则相似但不完全相同，容易混淆。  
  Mitigation: docs 明确 core 只读 `docnav.json`，adapter direct CLI 只读 `<adapter-id>.json`；MCP 不读 adapter 配置。

- [Risk] 配置文件中 native option 对部分 operation 不适用。  
  Mitigation: SDK 使用 `NativeOptionSpec.operations` 决定是否注入当前 operation；不适用时不写入 options。

- [Risk] 暴露配置路径覆盖参数后，调用方可能把路径参数误认为业务参数。  
  Mitigation: docs 和 help 使用 `--project-config-path` / `--user-config-path` 命名，并明确配置路径参数只影响配置加载；它们不进入 protocol request、readable output 或 adapter native options。

- [Risk] JSON-only 与文档中 `.docnav/<adapter-id>.*` 的长期表述存在范围差。  
  Mitigation: 本 change 把首期实现限定为 `.json`，后续格式扩展必须另开 change。

- [Risk] 用户以为 core `docnav outline` 会读取 `docnav-markdown.json`。  
  Mitigation: 明确非目标；core 仍只传递自身显式参数，adapter-specific defaults 只适用于 adapter direct CLI。

## Migration Plan

1. 更新 docs 和 OpenSpec 主规范，明确 adapter direct CLI JSON 配置路径默认值、`--project-config-path` / `--user-config-path`、配置范围、输出错误形态和优先级。
2. 在 `docnav-adapter-sdk` 中实现 config path defaults/override、read、merge、validation 和 resolved standard direct CLI parameters。
3. 接入 `docnav-markdown`，把静态默认值替换为内置默认值 fallback。
4. 增加 direct CLI smoke 和 SDK 单元测试。
5. 运行 Markdown adapter 范围测试和 workspace verifier。

## Open Questions

- 无。

## 实现边界

- Adapter direct CLI 配置只属于 adapter direct CLI；实现阶段不得把它扩展成 core `docnav` 的 adapter-specific options 来源。
