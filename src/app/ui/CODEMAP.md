# UI 展示与交互

使用 egui 原生控件绘制界面；舞台拖动、参数编辑和表情按钮直接更新同一个 Studio 状态。

## 任务导航

| 任务 | 领域 | Target | Also Check |
| --- | --- | --- | --- |
| 主题 / 字体 / 导航 / 追踪面板 | 窗口 | `mod.rs` | 新汉字需更新 `assets/fonts/AyakageUI.otf` |
| 独立输出与透明背景 | 输出 | `mod.rs` → `show_output` | `pages.rs` 输出设置，`../../render.rs` |
| 舞台拖动 / 缩放 / 复位 | 舞台 | `pages.rs` → `stage` | `../../render.rs` 的变换 |
| 映射或手动参数 | 参数 | `pages.rs` → `mappings` | `../../config.rs`, `../../model.rs` |
| 表情 / 动作 / 热键 | 操作 | `pages.rs` → `expressions`，`mod.rs` → `hotkey_picker` | `../../model.rs`, `../../config.rs` |

## 文件

| 文件 | 领域 | 职责 |
| --- | --- | --- |
| `mod.rs` | 窗口 | App UI 生命周期、框架布局、主题字体、追踪面板、输出窗口及延迟回调共享的截图请求 |
| `pages.rs` | 页面 | 舞台、映射 / 参数检查、表情 / 动作和输出接入设置 |
