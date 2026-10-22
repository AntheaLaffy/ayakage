`AyakageUI.otf` 是 Noto Sans CJK SC Regular 的界面字符子集，用于离线显示中文界面；许可见 `OFL.txt`。完整字体的上游为 <https://github.com/notofonts/noto-cjk>，字体内容未重新绘制，仅做 glyph 子集裁剪。

应用还会寻找系统 Noto CJK / 文泉驿字体，补全用户模型名称中的其他汉字。构建后修改界面字符时，可使用 `tools/subset_font.py` 更新子集；运行应用无需安装 Python 或 fontTools。
