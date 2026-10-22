找到一个重要突破：现在已经有不依赖官方闭源 Cubism Core、同时能够读取 Live2D `.moc3` 模型的开源实现了。

这意味着我们不一定要放弃现有的 Live2D 模型格式，也不必自己从零编写渲染器。

我发现了三个值得认真考虑的底层项目：

- [Ayagami](https://github.com/AyagamiDev/ayagami?utm_source=chatgpt.com)：Rust 实现的 Live2D 兼容模型处理与渲染库，也是 OpenVT 的底层依赖。
- [Purism Core](https://github.com/SakuraMotion/PurismCore?utm_source=chatgpt.com)：MIT 许可、C99 实现，目标是完整替代官方 Cubism Core，支持 Linux。
- [Mocari](https://github.com/Eatgrapes/Mocari?utm_source=chatgpt.com)：纯 Rust 的 Cubism 兼容运行时，包含模型状态、动画和基于 wgpu 的渲染功能。

  [image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

  GitHub

  +2



下面重点比较能够拿来直播的完整软件，而不是把这些底层库误当成已经完成的 VTuber 应用。

## 一、目前最值得用的，是 OpenVT

[erodozer/open-vt](https://github.com/erodozer/open-vt?utm_source=chatgpt.com)

最符合你的需求

Godot + GDScript + Rust，MIT 许可。支持 Linux 原生运行、标准 Live2D 模型、OpenSeeFace 面捕、透明窗口，以及类似 VTube Studio 的参数和热键映射。

[下载 Linux Nightly](https://erodozer.itch.io/openvt?utm_source=chatgpt.com) · [查看源码](https://github.com/erodozer/open-vt?utm_source=chatgpt.com)

我检查了实际仓库，不只是 README：

| 源码位置                           | 实际作用                  |
| ------------------------------ | --------------------- |
| `thirdparty/ayagami`           | Rust 实现的 Live2D 兼容运行时 |
| `lib/tracking/`                | 摄像头、网络、麦克风、追踪数据处理     |
| `lib/tracking/net/`            | TCP、UDP 追踪输入          |
| `studio/`、`ui/`                | 主程序及界面                |
| `build_dependencies.sh`        | 编译 Rust/Godot 扩展      |
| `.github/workflows/build.yaml` | 构建 Linux 发布包          |

OpenVT 的 nightly 构建还会自动获取经过 Ayagami 补丁修改的 Godot 4.7.1，因此你下载预编译版就不必先自己编译修改过的 Godot 引擎。

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



### 目前发现的不足

OpenVT 仍在积极开发，不是已经达到 VTube Studio 完整功能水平的成熟替代品。

尤其值得关注两点：一是它使用的 Ayagami 渲染绑定目前需要定制 Godot 补丁；二是后台全局热键依赖 Linux 输入设备访问权限，在你的 niri 环境中需要单独测试，不建议为了热键直接给日常账户开放全部输入设备权限。

此外，OpenVT 和 Ayagami 上游均有明确的禁止 AI 辅助贡献政策。你可以按照开源许可证使用和维护自己的 Fork，但如果准备借助 Agent 修改源码，不能将这些修改作为符合其贡献规范的上游 PR 提交。

## 二、其他方案的实际价值

| 项目               | 完整开源 | 标准 `.moc3` | 适合直接直播             |
| ---------------- | ---- | ---------- | ------------------ |
| OpenVT + Ayagami | 是    | 支持         | 最推荐，开发版            |
| Inochi Session   | 是    | 不支持        | 可以，但需要 Inochi2D 皮套 |
| Ayagami Demo     | 是    | 支持         | 主要是模型测试工具          |
| Mocari           | 是    | 支持         | 需要开发主播前端           |
| Purism Core      | 是    | 支持         | 需要对接应用             |

其中 Inochi Session 本身没有闭源 Cubism Core 的依赖问题，但它是另一套模型生态。既然你已经选了标准 Live2D，除非愿意重新绑定模型，否则不建议转过去。

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



另外还发现了 [Umamo](https://github.com/umamoorg/umamo?utm_source=chatgpt.com)：GPL-3.0 开源、Linux 原生，支持导入导出 CMO3/MOC3。它不是直播软件，而是正在开发的 Cubism Editor 替代品。以后你要自己修改皮套的网格和参数，可以留意它，目前仍属于 Alpha 阶段。

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 三、你的最终直播组合

OpenSeeFace

开源面部追踪

OpenVT + Ayagami

标准 Live2D 模型加载、变形与渲染

OBS Studio

PipeWire 捕获 · 场景合成 · 推流编码

你的 bili-live-hime

开播、直播间管理、弹幕互动

哔哩哔哩

OpenVT 官方明确提供 Linux nightly 包，可以从 [itch.io 项目页面](https://erodozer.itch.io/openvt?utm_source=chatgpt.com) 下载 `openvt-linux-nightly.zip`。

在 Arch Linux 上解压运行即可尝试：

```
mkdir -p ~/.local/opt/openvt
unzip ~/Downloads/openvt-linux-nightly.zip \
  -d ~/.local/opt/openvt

chmod +x ~/.local/opt/openvt/openvt.x86_64
~/.local/opt/openvt/openvt.x86_64
```

该启动路径依据项目构建脚本中的产物名称；如果下载包内部增加了目录层级，需要相应调整路径。

然后连接 OpenSeeFace，导入完整 Live2D 模型文件夹，在 OBS 中使用 PipeWire 窗口捕获。透明背景是否能通过 niri 的 Portal 完整传递 Alpha 通道，需要实际录制验证，因为应用支持透明窗口不等于所有 OBS 捕获方式都保留透明度。

## 四、我最终更看好这一套生态

完全开源的 Live2D 技术生态

[Umamo](https://github.com/umamoorg/umamo?utm_source=chatgpt.com)

模型制作 / 绑定

[Ayagami](https://github.com/AyagamiDev/ayagami?utm_source=chatgpt.com)

Rust 模型运行时

[OpenVT](https://github.com/erodozer/open-vt?utm_source=chatgpt.com)

Linux 原生 VTuber 应用

[OpenSeeFace](https://github.com/emilianavt/OpenSeeFace?utm_source=chatgpt.com)

面部追踪

这一套更有吸引力的地方是，它不仅开源，而且很多核心逻辑确实由 Rust 实现。对你来说，缺少功能时可以在自己的分支中继续开发，而不是受制于一个无法修改的二进制 Core。

建议先试 OpenVT 的 nightly。如果你的 Live2D 皮套可以正常加载、面捕和被 OBS 捕获，我们就直接以它为基础；如果发现问题，再针对具体缺口改造。 不必提前另起一个 VTuber 软件项目。
