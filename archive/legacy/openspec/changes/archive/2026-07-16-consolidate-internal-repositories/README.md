# consolidate-internal-repositories

将四个仅由 Docnav 使用的 Git 子仓库改为主仓库内源码，在保留现有 crate/module 职责与运行语义的同时移除独立 checkout、revision pin、nested workspace 和仅由子仓库边界产生的集成配置。
