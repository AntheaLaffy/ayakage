# Kage Studio

[![CI](https://github.com/AntheaLaffy/vtb-in-linux/actions/workflows/ci.yml/badge.svg)](https://github.com/AntheaLaffy/vtb-in-linux/actions/workflows/ci.yml)

面向 Linux / Wayland 的开源 Live2D 虚拟主播软件。首个 MVP 把现有模型、OpenSeeFace 面捕和 OBS 串起来；用纯 Rust 的 Mocari 运行时加载 `.moc3`，不依赖官方闭源 Cubism Core，也不需要定制 Godot。

## 开始使用

从 [GitHub Releases](https://github.com/AntheaLaffy/vtb-in-linux/releases) 下载 Linux x86_64 压缩包与 `SHA256SUMS`，放在同一目录后执行：

```sh
sha256sum --check SHA256SUMS
tar -xzf kage-studio-0.1.0-linux-x86_64.tar.gz
./kage-studio-linux-x86_64/kage-studio
```

预编译包以 Ubuntu 22.04 / glibc 2.35 为基线，需要系统 Vulkan / OpenGL 驱动与桌面 Portal。模型和 OpenSeeFace 需自行准备。`0.x` 是实验性 MVP。

Ubuntu / Debian 的常用运行库：

```sh
sudo apt install libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 \
  libx11-6 libxcursor1 libxi6 libxrandr2 libegl1 libgl1 libvulkan1
```

显卡驱动与 XDG Desktop Portal 使用桌面环境提供的版本；Mesa 软件 / 开源驱动可安装 `mesa-vulkan-drivers`。

从源码运行：

```sh
cargo run --locked --release
```

1. 点击「导入模型」选择 `.model3.json`，或把完整模型文件夹拖进窗口。`.moc3`、纹理、物理、表情和动作文件要保留在导出目录中。
2. 启动下方的 OpenSeeFace。右侧显示「追踪已连接」后，正视摄像头并点击「校准正面姿态」。
3. 在「参数映射」调整输入 / 输出范围；交换输出两端可反转方向。非标准参数可自行添加映射。
4. 点击「打开输出窗口」，在 OBS 中捕获 `Kage Studio · Output`。舞台内拖动平移、滚轮缩放、双击复位。

不接摄像头时，可勾选「使用模拟演示数据」检查模型绑定。界面会明确标记模拟状态。可直接打开模型并同时创建输出窗口：

```sh
cargo run --locked --release -- --model /path/to/Haru.model3.json --output
```

表情和动作读取模型清单的 `Expressions` / `Motions`。在「表情与动作」点击触发或分配 F1–F12；重复点击当前表情可取消。F5 校准、F6 暂停 / 恢复追踪、F7 复位舞台、F8 清除表情。应用内快捷键在窗口聚焦时生效。

设置保存到 `$XDG_CONFIG_HOME/kage-studio/settings.json`，默认 `~/.config/kage-studio/settings.json`；映射、舞台视图和热键按模型的绝对路径分别保存。可用 `--config /path/settings.json` 隔离测试配置。模型原文件不会被修改。

## 构建与运行依赖

需要 **Rust 1.95+**、C 编译器、pkg-config、Wayland / X11 和可用的 Vulkan 或 OpenGL 驱动。中文界面字体已内嵌；建议安装 Noto CJK，以显示模型自定义名称中的其他汉字。

Arch Linux 可安装以下构建依赖（已有 Rust 环境可保留）：

```sh
sudo pacman -S --needed base-devel rustup pkgconf wayland libxkbcommon libxkbcommon-x11 libx11 libxcursor libxi libxrandr mesa vulkan-intel noto-fonts-cjk
cargo build --locked --release
./target/release/kage-studio
```

`vulkan-intel` 适用于 Intel 核显；AMD 使用 `vulkan-radeon`，NVIDIA 使用对应驱动。文件选择器依赖桌面的 XDG Desktop Portal；niri 通常还需要 `xdg-desktop-portal-gnome` 提供文件选择器、`xdg-desktop-portal-gtk` 或其他可用后端。导入也可以通过拖放或 `--model` 完成。

打包当前机器可用的便携目录：

```sh
python3 tools/package.py --archive
./dist/kage-studio-linux-x86_64/kage-studio
```

构建产物沿用当前系统的 glibc 和图形库要求，不是通用 AppImage。

贡献和发布流程见 [CONTRIBUTING.md](CONTRIBUTING.md) 与 [CI 制度](docs/CI.md)，版本变化见 [CHANGELOG.md](CHANGELOG.md)。

纯文档 PR 只运行轻量文件分类与门禁汇总，跳过编译、模型渲染和打包；代码 / 依赖 / 工作流变化运行完整检查。定时任务及版本发布始终运行完整 CI。

## 接入摄像头

[OpenSeeFace](https://github.com/emilianavt/OpenSeeFace) 独立负责摄像头识别，Kage Studio 接收它的标准二进制 UDP 数据；图像不进入 Kage Studio，也不上传服务器。按照上游目前提供的锁文件安装兼容的 Python 环境：

```sh
git clone https://github.com/emilianavt/OpenSeeFace.git
cd OpenSeeFace
uv sync --locked
uv run facetracker.py -c 0 --ip 127.0.0.1 --port 11573
```

`uv` 负责选用上游要求的 Python 版本，避免系统 Python 3.14 与旧推理依赖冲突。用 `uv run facetracker.py --help` 查看摄像头、帧率等选项。Kage Studio 默认只监听 `127.0.0.1:11573`；修改端口后，需在两边保持一致。多人追踪时，在右侧选择人脸编号。失联约 650 ms 后通道平滑回到中立值。

## OBS / niri

在 OBS 添加「屏幕捕获 (PipeWire)」，从 Portal 选择 `Kage Studio · Output`。输出窗口只有模型和背景；控制界面不进入该窗口。可在「输出设置」选择横屏、方形或竖屏画布。

透明窗口的 alpha 是否穿过 Portal，取决于具体桌面与捕获方式；若录制显示黑底，切换为「绿幕」，在 OBS 给捕获源添加「色度键」滤镜，关键颜色选绿色。正式直播前录制一小段检查边缘效果。

也可使用已验证的 XWayland 捕获路径：用 `WAYLAND_DISPLAY= ./target/release/kage-studio --output` 启动，在 OBS 选择「窗口捕获 (XComposite)」并选中 Output 窗口，再使用绿色色度键。分发包中可把命令的可执行文件路径改为 `./kage-studio`。

Wayland 下全局热键由桌面负责，无需读取 `/dev/input`。例如把以下绑定加入 niri 的 `binds` 块，并把路径改成实际可执行文件的绝对路径：

```kdl
Mod+F5 { spawn "/path/to/kage-studio" "--command" "calibrate"; }
Mod+F6 { spawn "/path/to/kage-studio" "--command" "toggle-tracking"; }
Mod+F7 { spawn "/path/to/kage-studio" "--command" "reset-view"; }
Mod+F8 { spawn "/path/to/kage-studio" "--command" "clear-expression"; }
Mod+F1 { spawn "/path/to/kage-studio" "--command" "expression:F01"; }
```

命令接口为本机 UDP `11574`，应用实际执行后返回确认；应用未启动或动作失败时 CLI 返回非零状态。`motion:TapBody/0` 触发动作组的第 0 个动作，名称可在界面查看。默认命令端口同时只允许一个实例。开发验证可通过 `--control-port 16574 --config /path/test.json` 使用独立实例；对应 CLI 命令也必须指定相同的 `--control-port`。

## 当前边界

这是第一版 MVP，长期方向是向 VTube Studio 靠拢。已实现模型加载、面捕映射 / 校准 / 平滑、模型物理、待机动作、表情 / 动作热键、参数检查、独立直播输出和配置恢复。Mocari 仍为实验性运行时，首版不保证所有 Cubism 版本与特殊混合效果兼容。

暂未包含模型制作、手机追踪、VTube Studio 插件 API、道具系统、虚拟摄像头、Spout 和 VNet。原生窗口 / UDP / 模型渲染的验证结果见 [验证记录](docs/VERIFICATION.md)；摄像头实机和 OBS 最终录制需要结合你的设备确认。

开发入口见 [CONTRIBUTING.md](CONTRIBUTING.md)，范围与后续方向见 [MVP](docs/MVP.md) 和 [路线图](docs/ROADMAP.md)。本项目采用 MIT 许可，依赖与字体说明见 [THIRD_PARTY.md](THIRD_PARTY.md)。
