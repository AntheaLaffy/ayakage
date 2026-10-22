# 更新记录

## 0.1.1 · 2026-10-22

产品正式更名为 **绫影 · Ayakage**，标语为「让角色随你而动」。

- GitHub 仓库更名为 `AntheaLaffy/ayakage`，程序命令改为 `ayakage`，同步界面、帮助、文档和分发包。
- 输出窗口更名为 `Ayakage · Output`；OBS 窗口捕获目标与 niri 命令需更新。
- 默认配置改为 `ayakage/settings.json`，首次启动自动导入旧 `kage-studio` 配置并保留原件；已有新配置与显式 `--config` 不受导入影响。
- 更新内嵌字体以覆盖中文品牌名，帮助与界面版本跟随 Cargo 版本。

本版沿用 0.x 预发布制度。模型资源仍需自行准备，范围与环境限制见 [验证记录](https://github.com/AntheaLaffy/ayakage/blob/v0.1.1/docs/VERIFICATION.md)。

## 0.1.0 · 2026-10-22

首个 Linux / Wayland MVP，使用纯 Rust Mocari 运行时加载 Live2D 模型，接入 OpenSeeFace 与 OBS。

- 模型导入、参数映射、正面校准、平滑、物理、表情与动作。
- 独立透明 / 绿幕 / 深色输出窗口、舞台平移缩放、模型专属配置与快捷键。
- localhost UDP 面捕输入及带执行回执的命令接口，支持 niri 绑定。
- 模型检查、真实 GPU / UDP 验证及双窗口截图检查入口。
- Linux x86_64 便携发布包，包含源码、依赖许可、构建信息及 SHA-256 校验。
- PR / 主分支 / 每周 CI，最低 Rust 版本检查、软件 Vulkan 与 X11 窗口检查。

样例模型和 OpenSeeFace 不随包分发。摄像头识别质量、原生 PipeWire alpha 与长时间直播稳定性仍需在用户环境中确认，详见 [验证记录](https://github.com/AntheaLaffy/ayakage/blob/v0.1.0/docs/VERIFICATION.md)。
