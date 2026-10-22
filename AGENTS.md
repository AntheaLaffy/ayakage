## CODEMAP 导航

模式：maintenance。从 [CODEMAP.md](CODEMAP.md) 的任务导航进入对应模块；Target 是初始阅读集，Also Check 仅在任务相关或接口变化时继续读取。

- 按领域进入子目录地图；先核对当前文件和符号，再依赖地图描述。
- 接口、返回值或语义变更时检查调用者；新增、移动或删除文件时同步权威清单和父路由。
- 验证产品实际行为：构建不能代替模型渲染、UDP 及原生窗口检查。运行入口见 CONTRIBUTING.md。
- 仅在替代实现及消费者迁移均已确认后删除旧路径；本项目不维护平行的旧版实现。

平台以 Linux / Wayland 为主。模型 runtime 使用 Mocari；不要引入官方闭源 Cubism Core。保留 `talk.md` 原始背景，产品范围以 `docs/MVP.md` 为准。
