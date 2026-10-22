# 原生界面

该目录是 `app.rs` 的子模块，可以访问父模块状态；后台加载、保存和命令调度仍由 `app.rs` 负责。

## 任务导航

| 任务 | 领域 | Target | Also Check |
| --- | --- | --- | --- |
| 窗口布局、主题、导航、实时追踪面板 | UI | `ui/CODEMAP.md` → `mod.rs` | 状态变化时读 `../app.rs` |
| 舞台 / 参数 / 表情 / 输出设置交互 | UI | `ui/CODEMAP.md` → `pages.rs` | 对应 `../model.rs`, `../config.rs` |

## 子目录

| 目录 | 领域 | 依赖 | 职责 |
| --- | --- | --- | --- |
| [ui/](ui/CODEMAP.md) | UI | `../../config.rs`, `../../tracking.rs`, 父级 Studio | 布局、交互与输出窗口 |
