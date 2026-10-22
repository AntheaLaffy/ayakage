# 开发

先读 [CODEMAP.md](CODEMAP.md) 的任务导航，再进入对应模块。应用层与模型运行时分开：追踪模块只产生通道值，映射与动作修改模型参数，渲染模块统一产出透明纹理，主预览和直播窗口消费同一纹理。

开发分支通过 PR 合入 main，说明问题、动机、改动后的行为及实际验证。所有必需 CI 检查成功后再合并，不能以编译成功代替模型、UDP 或窗口检查。完整门禁、维护频率及版本发布步骤见 [docs/CI.md](docs/CI.md)。

`rust-toolchain.toml` 固定默认开发 / 发布工具链；最低支持 Rust 为 1.95，CI 同时测试两者。修改依赖需提交 `Cargo.lock`，修改界面字符需更新内嵌字体，新增 / 删除文件需同步 CODEMAP。产品变化需更新使用说明及更新记录。

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

无摄像头 / 无桌面也可以验证实际模型的 GPU 渲染和 UDP 驱动：

```sh
cargo run --locked -- --check-model /path/to/model.model3.json
cargo run --locked -- --render-test /path/to/model.model3.json --output-dir .local/verification/model
```

`--render-test` 使用真实模型、真实 localhost UDP 和实际 GPU 读回，检查非空像素、透明区域和追踪前后画面变化，输出 PNG 与 JSON 报告。模型表情可能包含中立表情，因此逐个检查，并要求至少一个表情产生画面变化。样例资产不随软件分发；请使用有权使用的模型。

在桌面会话内运行窗口检查；截图完成后自动退出，避免长期留下测试窗口：

```sh
cargo run --locked -- --model /path/to/model.model3.json --demo --output \
  --config .local/smoke/settings.json --screenshot .local/studio.png --smoke-test
```

输出窗口也会保存 `.local/studio-output.png`。`--demo` 仅用于验证绑定，不能据此宣称摄像头识别质量已验收。最终录制仍需在 OBS 中检查。

已有应用实例运行时，可给检查实例增加 `--control-port 16574` 并使用独立配置中的追踪端口，避免与默认 `11574` / `11573` 冲突。给该实例发送命令时也必须加上相同的 `--control-port 16574`。

调试接收链路而不运行摄像头识别，可用 `python3 tools/send_tracking.py` 发送明确标注的合成 OpenSeeFace 数据。格式来源是上游 [`facetracker.py`](https://github.com/emilianavt/OpenSeeFace/blob/master/facetracker.py) 和 [`Unity/OpenSee.cs`](https://github.com/emilianavt/OpenSeeFace/blob/master/Unity/OpenSee.cs)，每张脸 1785 字节，小端、逐字段打包，成功标记后无填充字节。

字体子集包含代码中的界面汉字。增加汉字后用 `python3 tools/subset_font.py /path/to/NotoSansCJK-Regular.ttc` 更新，开发环境需要 fontTools；运行应用不需要。
