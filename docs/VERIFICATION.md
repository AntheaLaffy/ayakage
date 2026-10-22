# MVP 验证记录

以下本机与 OBS 记录来自 0.1.0（旧名 Kage Studio），保留当时的窗口名称。0.1.1 更名为「绫影 · Ayakage」，当前输出窗口为 `Ayakage · Output`；更名后的发布包由完整 CI 再次检查，结果见 [GitHub Actions](https://github.com/AntheaLaffy/ayakage/actions)。

验证日期：2026-10-22。环境为 Arch Linux / niri，Intel RPL-P 核显。首版范围见 [MVP.md](MVP.md)；本记录只列实际执行结果。

## 构建与状态逻辑

`cargo test --locked`：14 项通过；`cargo clippy --locked --all-targets -- -D warnings` 和 `cargo fmt --check` 通过。覆盖实际 UDP 收发、多脸选择、左右眼字段、异常 / 截断数据、角度跨 ±180° 校准、平滑与失联回中、映射反转、模型档案 / 输出设置保存、命令执行确认，以及浏览实时参数不应自动锁定参数的 UI 回归。

应用已生成 Linux x86_64 release 可执行文件。分发目录包含二进制、可重建源码、依赖锁文件、文档与第三方许可证；`build-info.json` 记录二进制和锁文件 SHA-256。

## 真实模型与 GPU

使用 Mocari 上游样例模型（源码 checkout `13c7ed3c1e6471bcc1aacf5e7d3fdb1d038ed759`），通过应用的 `--render-test` 读取真实 `.model3.json`、`.moc3`、纹理和 sidecar 文件。模型资产不随本软件分发。

| 模型 | 参数 | 网格 | UDP 驱动改变的像素 | 物理更新的参数 | 表情数 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Haru | 42 | 84 | 27,198 | 4 | 8 |
| Hiyori | 70 | 134 | 43,319 | 33 | 0 |
| Mao | 132 | 262 | 65,045 | 14 | 8 |
| Mark | 21 | 30 | 29,838 | 3 | 0 |
| Natori | 96 | 176 | 24,642 | 8 | 11 |
| Ren | 73 | 198 | 44,135 | 11 | 5 |
| Rice | 96 | 178 | 21,344 | 41 | 0 |
| Wanko | 25 | 34 | 23,985 | 2 | 0 |

8 个模型均有可见像素、全透明区域及半透明边缘，实际 localhost UDP 输入均改变画面。逐个读取了声明的表情，有表情的模型均至少有一个表情改变画面；中立表情允许保持原画面。动作执行也产生画面变化。报告及 PNG 位于 `.local/acceptance/models.json`、`.local/acceptance/models/`。

这些结果证明上述样例的导入和驱动链路，不代表所有 Cubism 版本、特殊混合模式或用户模型都已兼容。

## 原生窗口与交互

已在实际 niri / Wayland 会话运行主窗口和独立 `Kage Studio · Output` 窗口并截图。独立输出截图保留全透明区域和半透明边缘，不包含控制界面。参考产物为 `.local/studio.png`、`.local/studio-output.png`。

本机候选分发包再次在原生 Wayland 下通过双窗口检查并正常退出：透明输出包含超过 92 万个全透明像素和超过 2 千个半透明边缘像素；深色输出所有像素不透明，背景为 `(17, 19, 26)`。第二次启动不提供 `--model`，成功恢复上次模型以及非默认 view、反向映射和 F12 表情快捷键。报告、截图和日志在 `.local/acceptance/final-native/`，其中 SHA-256 与该次本机分发包及模型验证报告对应。GitHub Release 使用独立的干净 runner 构建。

已检查中文界面、参数映射页、手动参数页、实际 F6 切换，以及通过 CLI 校准、触发表情和动作。独立验收配置恢复了非默认舞台视图、模型映射、快捷键、绿幕背景、分辨率与标题栏设置，未修改默认用户配置。

在独立 Xvfb 显示中，实际鼠标键盘流程也已通过：禁用 / 反转映射并自动保存，编辑 / 释放手动参数，拖动和滚轮缩放舞台，F7 复位、F6 暂停 / 恢复，表情按钮触发及 F1 取消。拖动产生 pan `[0.22858776, -0.20318912]`，滚轮将 zoom 从 `1.0` 改为 `1.62`。报告和各步骤截图在 `.local/acceptance/ui/`。Xvfb 的软件渲染结果用于确认交互逻辑，不用于报告硬件帧率。

## OBS 实际录制

OBS Studio 32.2.2，独立测试配置；精确捕获 `Kage Studio · Output`，使用 XComposite 窗口捕获与 `chroma_key_filter_v2` 绿色色度键。实际录制 H.264 / 960×720，时长 6.166 秒，可解码 182 帧。两次 OBS 场景截图有 28,652 个像素变化，未残留纯绿色背景，画面只有模型与测试背景，没有控制界面。

面部输入使用明确的合成 OpenSeeFace UDP 数据，并实际执行了 `calibrate`、`expression:F07`、`motion:TapBody/0`。测试 OBS 实例在录制前移除了麦克风和桌面音频来源，没有进行推流。报告、场景截图和录像在 `.local/acceptance/obs-report.json`、`obs-frame-a.png`、`obs-frame-b.png` 及报告指定的 MKV 路径。

这证明 XWayland 输出窗口 → OBS 窗口捕获 → 绿幕去除 → 实际编码的链路。原生 Wayland 的 PipeWire / Portal alpha 是否完整传递仍取决于捕获环境；本次没有把 XComposite 结果当作 PipeWire alpha 的证明。

## 使用者还需确认

- 真实摄像头、光照和 OpenSeeFace 识别质量，以及自己的模型参数范围。
- 具体 Portal / OBS 捕获方式的透明通道；遇到黑底时使用已验证的绿幕路径。
- 长时间直播稳定性与目标机器帧率。当前结果不构成长时间压力测试或固定 FPS 保证。

首次本机验收时尚未初始化 Git，地图按实际目录清单核对。GitHub 发布阶段补充了锁定工具链、最低 Rust 版本、解压后发布包及软件 Vulkan / X11 检查，具体门禁见 [CI.md](CI.md)，每次实际结果以对应 GitHub Actions 运行记录为准。运行和复验入口见 [CONTRIBUTING.md](../CONTRIBUTING.md)。
