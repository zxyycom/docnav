本 design 说明 `interactive-outline-selection` 的实现方向：在核心 CLI 内为 `docnav outline <path>` 增加面向人类的交互式选择流程，并比较可用 Rust 终端交互库；当前 change 只在 `openspec/changes/interactive-outline-selection/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Docnav 的长期边界是 `outline -> ref -> read`。当前 CLI 已经适合 AI、脚本和可复制输出，但人类用户在终端中需要从 outline 输出中复制 ref，再手动调用 `read`。这个 change 的核心价值是把“看 outline、选条目、读内容”编排成一个 core CLI human-only workflow。

该能力位于 `docnav` 核心 CLI，因为它跨越 outline 和 read 两个 document operation，并且不改变任何格式 adapter 的解析、ref 生成或 read 语义。交互 UI 不属于 adapter protocol，也不属于 readable/protocol JSON 输出契约。

## Goals / Non-Goals

**Goals:**

- 为 `docnav outline <path> --interactive` 定义第一版人类交互流程。
- 复用现有 adapter selection、outline invoke、read invoke、错误映射和输出编排。
- 允许用户从 outline entries 中选择一个或多个 ref，并按选择顺序读取。
- 明确非 TTY、JSON output mode、用户取消和空选择行为。
- 给出 prompt-style 多选库和 full TUI 框架的候选示例，便于实现前做依赖决策。

**Non-Goals:**

- 不改变 adapter protocol、ref contract、outline entry shape 或 read result shape。
- 不给 protocol-json 或 readable-json 增加 batch selection 字段。
- 第一版不承诺树形折叠、实时 read preview pane、鼠标操作、持久布局或自定义快捷键系统。
- 第一版不把 interactive workflow 暴露给 adapter 直接 CLI。

## Decisions

### Decision: interactive outline 属于 core CLI 编排

`--interactive` SHALL 先通过现有 outline path 取得 entries，再将 entry display/ref 映射成可选项。用户确认选择后，core CLI 逐个调用现有 read 语义。这样可以保持 adapter 只拥有格式解析、outline/ref/read 业务语义，core CLI 拥有人类交互编排。

替代方案是让 adapter 提供 interactive outline，但这会复制交互实现、扩大 adapter 进程边界，并迫使每个 adapter 理解多选 UI，不符合当前制品职责。

### Decision: 第一版优先 prompt-style MultiSelect

第一版建议优先评估 `inquire::MultiSelect` 或 `dialoguer::MultiSelect`。两者都直接覆盖“从一组 options 中多选”的基础需求，能以较低实现成本证明人类 workflow 价值。

`inquire` 基础示例：

```rust
use inquire::MultiSelect;

#[derive(Clone)]
struct OutlineChoice {
    display: String,
    ref_id: String,
}

impl std::fmt::Display for OutlineChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

let choices: Vec<OutlineChoice> = outline_entries
    .into_iter()
    .map(|entry| OutlineChoice {
        display: entry.display,
        ref_id: entry.ref_id,
    })
    .collect();

let selected = MultiSelect::new("Select sections to read", choices)
    .with_page_size(12)
    .prompt()?;

for choice in selected {
    read_ref(&path, &choice.ref_id)?;
}
```

`dialoguer` 基础示例：

```rust
use dialoguer::{theme::ColorfulTheme, MultiSelect};

let labels: Vec<String> = outline_entries
    .iter()
    .map(|entry| entry.display.clone())
    .collect();

let selected_indexes = MultiSelect::with_theme(&ColorfulTheme::default())
    .with_prompt("Select sections to read")
    .items(&labels)
    .interact()?;

for index in selected_indexes {
    let ref_id = &outline_entries[index].ref_id;
    read_ref(&path, ref_id)?;
}
```

`inquire` 的优势是 MultiSelect 自带 filter 相关配置和 option formatter，适合大型 outline 初版筛选；`dialoguer` 的优势是 API 简单、返回 index 便于保持 entry/ref 映射。最终选型应在实现前检查 crate 版本、feature、Windows 终端行为和 transitive dependency。

### Decision: ratatui 作为第二阶段 full TUI 路线

如果产品目标升级为树形折叠、左右 pane、实时 preview、复杂快捷键或鼠标操作，应改用 `ratatui` 搭配 terminal backend/event layer，而不是继续堆叠 prompt API。

`ratatui` 方向的最小形态：

```rust
use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};

let mut terminal = ratatui::init();
let mut selected = std::collections::BTreeSet::<usize>::new();
let mut cursor = 0usize;

loop {
    terminal.draw(|frame| {
        let items = outline_entries.iter().enumerate().map(|(index, entry)| {
            let mark = if selected.contains(&index) { "[x]" } else { "[ ]" };
            ListItem::new(format!("{mark} {}", entry.display))
        });
        frame.render_widget(List::new(items), frame.area());
    })?;

    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char(' ') => {
                if !selected.insert(cursor) {
                    selected.remove(&cursor);
                }
            }
            KeyCode::Enter => break,
            KeyCode::Esc => return Ok(()),
            KeyCode::Down => cursor = (cursor + 1).min(outline_entries.len() - 1),
            KeyCode::Up => cursor = cursor.saturating_sub(1),
            _ => {}
        }
    }
}

ratatui::restore();
```

这一路线更强，但会引入 event loop、terminal raw mode 生命周期、panic/early-return restore、snapshot 或 pseudo-terminal 测试等额外成本。它不适合作为第一版除非明确需要 full-screen TUI。

### Decision: JSON output mode 与 interactive 互斥

`--interactive` 是 human-only UI。调用方显式选择 `--output readable-json` 或 `--output protocol-json` 时，core CLI MUST 返回 `INVALID_REQUEST`，避免 stdout 同时承载 prompt 控制序列和机器可读 JSON。未显式选择 output 时，interactive 模式可以使用人类终端 UI，并在选择后以 human-readable read rendering 展示结果。

### Decision: 不新增 batch protocol

第一版不新增“多个 read result 的 protocol envelope”。交互确认后，core CLI 可以在进程内顺序执行 read，并按人类 UI 需要展示每个 read result。若未来需要脚本化批量读取，应另行设计非交互 batch/read composition change。

## Risks / Trade-offs

- [Risk] prompt-style UI 不能表达真实树形折叠。→ Mitigation: 第一版只承诺多选 entries；树形 TUI 作为后续 ratatui 路线。
- [Risk] 终端交互在 CI、管道或重定向环境中失败。→ Mitigation: 显式检测 TTY，非 TTY 返回 `INVALID_REQUEST`，测试覆盖。
- [Risk] 选中多个 ref 后输出边界不清。→ Mitigation: 不定义机器可读 batch 输出；第一版作为 human-only workflow。
- [Risk] 用户取消和空选择容易被误判为错误。→ Mitigation: 用户取消返回成功且不执行 read，空选择返回成功且不执行 read，除非后续产品要求强制至少选择一项。
- [Risk] 新依赖增加 Windows 终端兼容性风险。→ Mitigation: 实现前用候选库官方 API 和 Windows 本地 smoke 验证，再固定 workspace dependency。

## Migration Plan

1. 在 core CLI 参数层增加 `outline --interactive`，普通 `outline` 行为保持不变。
2. 将 interactive workflow 放在 `docnav` CLI 交互层，复用现有 core handoff、navigation input resolution、adapter selection 和 read execution。
3. 先实现 prompt-style 多选 MVP；若后续需求要求 full TUI，再单独提 change 或扩展 design。
4. 失败或回滚时移除 `--interactive` 参数和交互依赖，不影响 adapter protocol 和已有 document operation。

## Open Questions

- 第一版选择 `inquire` 还是 `dialoguer`，需要在实现前结合当前 crate 版本、features、Windows 行为和依赖体积确认。
- 选中多个 ref 后，在终端中是直接顺序打印 read readable-view，还是进入一个可滚动的临时查看界面，需要在实现前定最终 UX。
- 用户取消时 exit code 是否必须为 0；本 design 倾向 0，因为这是用户主动退出，不是文档操作失败。
