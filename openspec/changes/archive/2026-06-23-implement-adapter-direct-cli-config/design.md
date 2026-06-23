本 design 是该 change 的正式设计，说明 adapter direct CLI 如何通过 SDK 解析可覆盖的配置文件路径，并把配置内容合并为标准 direct CLI 参数来源对象；实现阶段按 tasks 同步 owner 主规范、代码、schema/example 和测试。

## Context

Docnav 当前已经有 core `docnav` 配置实现，但 adapter direct CLI 仍只使用静态默认值和 argv。规范上每个 CLI 拥有独立配置域，且 `docnav-markdown` 的 `limit_chars` 和 `max_heading_level` 默认值属于 adapter 配置域；实现上这条链路缺失。

本 change 建立 adapter direct CLI 的通用配置读取基础设施，让具体 adapter 可以复用 SDK 提供的默认项目配置路径、默认用户配置目录参数和可覆盖配置路径。SDK 读取配置文件后，把配置值与 argv 合并成标准 direct CLI 参数来源对象，再交给既有参数处理、request 构造和 operation handler。argv、配置文件、`invoke` stdin JSON 和 MCP 映射到 core CLI 的参数都是入口侧传参方式；对应 CLI 内部的 document operation 线路仍是业务逻辑的唯一来源。

## Goals / Non-Goals

**Goals:**

- 为 adapter direct CLI 支持项目级和用户级 JSON 配置，并把两者的配置文件路径作为可覆盖的 SDK-owned standard 参数。
- 统一配置优先级：显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值。
- 支持通用 direct CLI 默认值 `defaults.limit_chars`、`defaults.output`。
- 支持 adapter native options 配置，并把配置中的 `options` 合并为标准 native option 参数来源。
- 在 SDK 内部把配置文件解析结果合并为标准 direct CLI 参数来源对象，避免每个 adapter 重建合并逻辑。
- 保持 adapter document operation 的内部执行线路唯一；argv/config 与 `invoke` 只决定进入该线路前提供哪些参数。
- 为配置路径参数补充 help 暴露，并为显式配置路径不可用、配置 JSON 无法解析等配置源跳过原因产生 direct CLI warning。
- 让 `docnav-markdown` 读取 `docnav-markdown.json`，支持 `options.max_heading_level`。
- 为 `docnav-markdown` 配置提供参考 schema 和示例，支持文档校验、编辑器提示和后续 adapter package 打包分发。
- 保持 adapter `invoke` stdin JSON 严格，不读取配置。

**Non-Goals:**

- 不让 core `docnav` 读取 adapter 专属配置。
- 不从 adapter manifest 合成默认参数或 native options。
- 不改变 protocol request/response 字段 shape。
- 不支持 JSON 之外的配置格式。
- 不把配置 schema 作为 direct CLI runtime 读取前置校验；runtime 仍按配置读取规则和标准 direct CLI 参数处理链路处理。
- 不实现 feature-specific 配置项，如 unstructured outline 或 frontmatter；这些由后续 feature change 使用本基础设施。
- 标准 direct CLI 参数处理链路保持现状；本 change 只为该链路提供配置来源。

## Decisions

1. **配置读取放在 `docnav-adapter-sdk` direct CLI 层。**

   SDK 已经拥有 direct CLI argv、native option、输出模式和 operation request 构造。把配置读取放在这里，可以让所有 Rust adapter direct CLI 共享路径发现、读取顺序和合并优先级。

   备选方案是每个 adapter 自行读取配置。该方案短期简单，但会导致 `docnav-markdown` 和后续 adapter 重复实现项目根发现、用户配置路径、JSON 读取和 native option 合并。

2. **配置路径是标准 direct CLI 参数，默认值可覆盖。**

   SDK 为每个 adapter direct CLI 计算两个默认配置路径：项目级默认是当前项目根下 `.docnav/<adapter-id>.json`，用户级默认是默认用户配置目录下 `<adapter-id>.json`。Adapter direct CLI 自行启动时，从启动 cwd 向上查找最近的 `.docnav/`；找到则以其父目录作为项目根，未找到则使用启动 cwd。Document path 不参与 adapter direct CLI 配置项目根发现。默认用户配置目录由 SDK config helper 的调用方提供；调用方未提供时使用当前调用位置（启动 cwd）。Direct CLI 使用 `--project-config-path <path>` 和 `--user-config-path <path>` 覆盖这两个路径；相对覆盖路径按启动 cwd 解析。路径参数只影响配置加载，不进入 protocol request、native options 或 operation handler。

   备选方案是把路径发现完全硬编码在 SDK 内部。该方案少暴露参数，但测试、临时配置和嵌入式调用更难控制，也会让后续工具难以指定隔离配置。

3. **只支持 JSON 配置文件。**

   本 change 读取两个解析后路径指向的 JSON 文件。用户级默认路径不在 SDK 内复用 core 用户配置目录发现逻辑；SDK 只使用调用方提供的默认用户配置目录参数，并在未提供时回退到当前调用位置（启动 cwd）。覆盖路径可指向任意可访问 JSON 文件。

   备选方案是立即支持 `.toml`、`.yaml` 或 glob 扫描。该方案扩大 parser 和实现面，且当前 core 实现已经使用 JSON。

4. **配置只影响 direct CLI document operations。**

   `docnav-markdown outline/read/find/info` 等 direct CLI 文档命令读取配置。`manifest`、`probe`、`help` 和 `invoke` 不读取 document operation 配置；`help` 只展示可传入的 SDK-owned 配置路径参数，`invoke` 只消费 stdin 中已显式给出的 protocol request。

   这样保持 process boundary：core `docnav` 通过 adapter `invoke` 调用 adapter 时，只有 core 显式写入 request 的参数会进入 adapter document operation 线路，不会被 adapter direct CLI 配置隐式改变。

5. **配置内容先合并为标准 direct CLI 参数来源对象。**

   SDK 解析 argv 后先得到配置路径、显式业务参数和显式 native options；随后读取用户级与项目级配置，把 argv、配置和内置默认值合并进标准 direct CLI 参数来源对象。实现中该对象由 `docnav-adapter-sdk` direct CLI 层拥有，可落为 `DirectCliParameterSources` 或等价内部类型，至少承载 `limit_chars`、`output`、`native_options`、`path`、`ref`、`query`、`page` 和 warning 列表。配置读取层将 `defaults.limit_chars` 投影为 `limit_chars` 参数来源，将 `defaults.output` 投影为 `output` 参数来源，将 `options` object 整体投影为 native options 参数来源；`path`、`ref`、`query` 来自 argv，`page` 来自 argv 或入口固定默认 `1`。后续 direct CLI 参数处理链路对该对象执行标准化、类型校验、native option 注册校验和 operation 适用性处理。

   备选方案是在每个 operation request 构造点分别读取配置。该方案会让 outline/read/find/info 的优先级实现分散，且容易把配置读取、参数投影和后续 native option 处理边界混在一起。

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

   SDK 将配置中的 `options` object 保留为标准 native option 参数来源，后续与 argv native options 使用同一条处理链路。配置层不判断 `options.<key>` 是否已注册，也不判断它是否适用于当前 operation。

7. **配置读取只产出标准参数来源对象和 warning。**

   本 change 的责任边界到“产出标准 direct CLI 参数来源对象”为止：

   1. 解析配置路径。未覆盖的默认配置路径不存在，表示该层没有配置源；显式 `--project-config-path` / `--user-config-path` 替换对应默认路径。
   2. 读取 JSON 配置源。未覆盖默认路径不存在不产生 warning；显式覆盖路径不可用、已发现配置路径不可读、JSON 无法解析或顶层不是 JSON object 时，该配置源不参与本次合并，并产生 `adapter_config_source_skipped` direct CLI warning。该 warning family 由 `docnav-diagnostics` 和 readable warning schema 承接，details 使用稳定字段：`source_level` 为 `project` 或 `user`，`path_origin` 为 `default` 或 `override`，`path` 为本次尝试读取的解析后路径，`reason_code` 为 `missing_override`、`not_file`、`unreadable`、`invalid_json` 或 `non_object`。
   3. 合并标准参数来源对象。SDK 按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”把 `defaults.limit_chars`、`defaults.output` 和 `options` 投影为标准 direct CLI 参数来源对象。合并阶段只处理来源优先级和固定字段投影；未知顶层字段和未知 `defaults` 字段不产生配置读取 warning，`options` object 中的 key/value 原样进入 native options 参数来源。

   合并完成后的参数来源对象和 warning 交给既有 direct CLI 参数处理与输出通道。

8. **配置 schema/example 是参考材料，不是 runtime gate。**

   `docs/schemas/docnav-markdown-config.schema.json` 记录当前 `docnav-markdown` 配置文件的推荐 JSON shape，`docs/examples/json/docnav-markdown-config.json` 提供可校验示例。它们用于文档验证、编辑器提示、安装或打包时随 adapter 分发，不改变 direct CLI runtime 行为：direct CLI 不需要先用该 schema 校验配置文件，也不因为 schema 文件缺失而拒绝读取配置。

   配置读取层处理的是配置源是否可读、JSON 是否可解析、顶层是否为 object、固定字段投影和来源优先级。它将 `defaults.limit_chars`、`defaults.output` 和 `options` 合并为标准 direct CLI 参数来源对象，并把该对象交给既有 direct CLI 参数处理链路完成类型、范围、枚举、native option 注册和 operation 适用性处理。未知字段是否可用不是配置读取层责任。

## Risks / Trade-offs

- [Risk] Direct CLI 配置和 core `docnav` 配置规则相似但不完全相同，容易混淆。  
  Mitigation: docs 明确 core 只读 `docnav.json`，adapter direct CLI 只读 `<adapter-id>.json`；MCP 不读 adapter 配置。

- [Risk] 配置文件中 native option 对部分 operation 不适用。  
  Mitigation: 配置读取层只把 `options` 合并为标准 native option 参数来源；operation 适用性由既有 native option 处理链路负责。

- [Risk] 用户级默认路径如果复用 core 环境变量查找，会把 core binary 的内部路径逻辑泄漏到 adapter SDK。
  Mitigation: adapter SDK 使用显式默认用户配置目录参数；调用方未传入时使用当前调用位置（启动 cwd），`--user-config-path` 仍可覆盖最终读取路径。

- [Risk] 暴露配置路径覆盖参数后，调用方可能把路径参数误认为业务参数。  
  Mitigation: docs 和 help 使用 `--project-config-path` / `--user-config-path` 命名，并明确配置路径参数只影响配置加载；它们不进入 protocol request、readable output 或 adapter native options。

- [Risk] 配置文件读取失败如果静默跳过，会让显式传错路径或 JSON 写坏的用户难以定位问题。
  Mitigation: 默认配置文件不存在时不报警；显式覆盖路径不可用、已发现配置路径不可读或 JSON 无法解析时产生 direct CLI warning，document operation 仍按其余来源继续执行。

- [Risk] JSON-only 与文档中 `.docnav/<adapter-id>.*` 的长期表述存在范围差。  
  Mitigation: 本 change 把首期实现限定为 `.json`，后续格式扩展必须另开 change。

- [Risk] 用户以为 core `docnav outline` 会读取 `docnav-markdown.json`。  
  Mitigation: 明确非目标；core 仍只传递自身显式参数，adapter-specific defaults 只适用于 adapter direct CLI。

- [Risk] 配置 schema 被误解为 runtime 强制校验。
  Mitigation: schema/example 文档和 design 明确它们是参考与打包材料；runtime 边界仍由 direct CLI 配置读取和标准参数处理链路负责。

## Migration Plan

1. 更新 docs 和 OpenSpec 主规范，明确 adapter direct CLI JSON 配置路径默认值、默认用户配置目录参数、`--project-config-path` / `--user-config-path`、配置范围、优先级、项目根发现和 config schema/example 的参考边界。
2. 在 `docnav-adapter-sdk` 中实现 config path defaults/override、read、merge 和 standard direct CLI parameter sources。
3. 为配置源跳过原因接入 `adapter_config_source_skipped` direct CLI warning，同步 `docnav-diagnostics`、readable warning schema 和 warning 测试，并确认 help 展示配置路径参数但不读取配置。
4. 接入 `docnav-markdown`，把静态默认值替换为内置默认值 fallback。
5. 增加 `docnav-markdown` config schema/example，并让 docs validator 校验示例符合 schema。
6. 增加 direct CLI smoke 和 SDK 单元测试。
7. 运行 Markdown adapter 范围测试、docs validation 和 workspace verifier。

## Open Questions

- 无。

## 实现边界

- Adapter direct CLI 配置只属于 adapter direct CLI；实现阶段不得把它扩展成 core `docnav` 的 adapter-specific options 来源。
