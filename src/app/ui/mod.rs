use super::*;
use egui::{Color32, FontId, RichText, Vec2};
mod pages;

const BG: Color32 = Color32::from_rgb(16, 17, 24);
const PANEL: Color32 = Color32::from_rgb(22, 24, 33);
const CARD: Color32 = Color32::from_rgb(29, 31, 43);
const LINE: Color32 = Color32::from_rgb(43, 45, 60);
const MUTED: Color32 = Color32::from_rgb(137, 142, 163);
const TEXT: Color32 = Color32::from_rgb(233, 233, 243);
const ACCENT: Color32 = Color32::from_rgb(184, 164, 255);
const GREEN: Color32 = Color32::from_rgb(114, 219, 174);

impl Studio {
    fn header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .exact_size(76.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(24, 17))
                    .stroke(egui::Stroke::new(1.0, LINE)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(38.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 11, ACCENT);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "A",
                        FontId::proportional(26.0),
                        BG,
                    );
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(crate::PRODUCT_NAME)
                                .size(19.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.label(
                            RichText::new(format!(
                                "让角色随你而动。  /  {}",
                                env!("CARGO_PKG_VERSION")
                            ))
                            .size(11.0)
                            .color(MUTED),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if primary(
                            ui,
                            if self.output {
                                "●  输出窗口已开启"
                            } else {
                                "↗  打开输出窗口"
                            },
                        )
                        .clicked()
                        {
                            self.output = !self.output;
                        }
                        if ui
                            .add_sized([116.0, 36.0], egui::Button::new("＋  导入模型"))
                            .clicked()
                        {
                            self.open_dialog();
                        }
                        ui.add_space(8.0);
                        badge(ui, "LINUX / WAYLAND", MUTED);
                    });
                });
            });
    }
    fn navigation(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("navigation")
            .exact_size(206.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(16, 22)),
            )
            .show(ui, |ui| {
                eyebrow(ui, "工作空间");
                ui.add_space(12.0);
                for (page, label) in [
                    (Page::Stage, "预览舞台"),
                    (Page::Mapping, "参数映射"),
                    (Page::Expressions, "表情与动作"),
                    (Page::Output, "输出设置"),
                ] {
                    let selected = self.page == page;
                    let button = egui::Button::new(
                        RichText::new(format!("      {label}"))
                            .size(14.0)
                            .color(if selected { ACCENT } else { MUTED }),
                    )
                    .fill(if selected {
                        Color32::from_rgb(47, 41, 68)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(8);
                    let response = ui.add_sized([174.0, 43.0], button);
                    paint_nav_icon(
                        ui.painter(),
                        egui::pos2(response.rect.left() + 24.0, response.rect.center().y),
                        page,
                        if selected { ACCENT } else { MUTED },
                    );
                    if response.clicked() {
                        self.page = page;
                    }
                    ui.add_space(4.0);
                }
                ui.add_space(25.0);
                eyebrow(ui, "当前模型");
                ui.add_space(12.0);
                egui::Frame::new()
                    .fill(CARD)
                    .corner_radius(10)
                    .inner_margin(14)
                    .show(ui, |ui| {
                        ui.set_min_width(146.0);
                        if let Some(model) = &self.model {
                            ui.label(RichText::new(&model.name).strong().size(15.0));
                            ui.add_space(5.0);
                            muted(
                                ui,
                                &format!(
                                    "{} 个参数 · {} 个网格",
                                    model.runtime().parameter_ids().len(),
                                    model.runtime().meshes().len()
                                ),
                            );
                        } else {
                            ui.label("尚未导入模型");
                            muted(ui, "支持标准 .model3.json");
                        }
                    });
                if !self.config.recent.is_empty() {
                    ui.add_space(10.0);
                    let mut chosen = None;
                    egui::ComboBox::from_id_salt("recent")
                        .selected_text("最近使用…")
                        .width(168.0)
                        .show_ui(ui, |ui| {
                            for path in &self.config.recent {
                                let name = path.file_name().unwrap_or_default().to_string_lossy();
                                if ui
                                    .selectable_label(false, name.as_ref())
                                    .on_hover_text(path.display().to_string())
                                    .clicked()
                                {
                                    chosen = Some(path.clone());
                                }
                            }
                        });
                    if let Some(path) = chosen {
                        self.load(path);
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.label(
                        RichText::new("独立开源运行时 · 本机处理")
                            .size(11.0)
                            .color(MUTED),
                    );
                    ui.add_space(10.0);
                    let save_label = if self.save_error.is_some() {
                        "! 配置保存失败"
                    } else if !self.config_writable {
                        "! 自动保存已关闭"
                    } else {
                        "✓ 设置自动保存"
                    };
                    ui.label(
                        RichText::new(save_label)
                            .size(12.0)
                            .color(if self.config_writable { MUTED } else { ACCENT }),
                    );
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(14.0);
                    let (label, color) = self.tracking_status();
                    ui.label(RichText::new(format!("● {label}")).size(12.0).color(color));
                    eyebrow(ui, "OPENSEEFACE");
                });
            });
    }
    fn tracking_status(&self) -> (&'static str, Color32) {
        if self.tracking.error.is_some() {
            ("监听失败", Color32::LIGHT_RED)
        } else if self.tracking.demo {
            ("模拟演示数据", ACCENT)
        } else if !self.config.tracking.enabled {
            ("追踪已暂停", MUTED)
        } else if self.tracking.connected() {
            ("追踪已连接", GREEN)
        } else {
            ("等待追踪信号", MUTED)
        }
    }
    fn inspector(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("inspector")
            .exact_size(276.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(PANEL).inner_margin(20))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().slider_width = (ui.available_width() - 78.0).max(80.0);
                        ui.add_space(4.0);
                        ui.heading("追踪输入");
                        ui.add_space(5.0);
                        let (label, color) = self.tracking_status();
                        badge(ui, label, color);
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.label("面部追踪");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.checkbox(&mut self.config.tracking.enabled, "启用");
                                },
                            );
                        });
                        ui.add_space(12.0);
                        muted(ui, "接收地址  127.0.0.1");
                        ui.horizontal(|ui| {
                            ui.label("UDP 端口");
                            ui.add(egui::DragValue::new(&mut self.port_edit).range(1024..=65535));
                            if ui.small_button("应用").clicked() {
                                self.config.tracking.port = self.port_edit;
                                self.tracking.bind(self.port_edit);
                            }
                        });
                        let previous_face = self.config.tracking.face_id;
                        ui.horizontal(|ui| {
                            ui.label("人脸编号");
                            ui.add(
                                egui::DragValue::new(&mut self.config.tracking.face_id)
                                    .range(0..=99),
                            );
                        });
                        if previous_face != self.config.tracking.face_id {
                            self.tracking.latest = None;
                            self.tracking.last_seen = None;
                            self.config.tracking.center = None;
                        }
                        ui.add_space(16.0);
                        ui.label("平滑");
                        ui.add(
                            egui::Slider::new(&mut self.config.tracking.smoothing_ms, 0.0..=250.0)
                                .suffix(" ms"),
                        );
                        muted(ui, "数值越低越灵敏，越高越平稳。");
                        ui.add_space(14.0);
                        if ui
                            .add_enabled(
                                self.tracking.connected() && !self.tracking.demo,
                                egui::Button::new("◎  校准正面姿态  ·  F5")
                                    .min_size([ui.available_width(), 36.0].into()),
                            )
                            .clicked()
                        {
                            self.execute(Action::Calibrate);
                        }
                        muted(
                            ui,
                            if self.config.tracking.center.is_some() {
                                "已保存校准姿态"
                            } else {
                                "正视摄像头后点击校准"
                            },
                        );
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(16.0);
                        eyebrow(ui, "实时通道");
                        ui.add_space(12.0);
                        for channel in [
                            Channel::HeadYaw,
                            Channel::HeadPitch,
                            Channel::HeadRoll,
                            Channel::EyeLeft,
                            Channel::EyeRight,
                            Channel::MouthOpen,
                        ] {
                            let value = self.tracking.values[channel.index()];
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(channel.label()).size(12.0).color(MUTED));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!("{value:.2}"))
                                                .monospace()
                                                .size(11.0),
                                        );
                                    },
                                );
                            });
                            let [min, max] = channel.range();
                            ui.add(
                                egui::ProgressBar::new((value - min) / (max - min))
                                    .fill(ACCENT)
                                    .desired_height(3.0),
                            );
                            ui.add_space(7.0);
                        }
                        ui.add_space(10.0);
                        muted(
                            ui,
                            &format!(
                                "有效帧 {}  /  丢弃 {}",
                                self.tracking.packets, self.tracking.rejected
                            ),
                        );
                        if let Some(frame) = &self.tracking.latest {
                            muted(
                                ui,
                                &format!("Face {} · t {:.2}", frame.face_id, frame.timestamp),
                            );
                        }
                        if let Some(error) = &self.tracking.error {
                            ui.colored_label(Color32::LIGHT_RED, error);
                        }
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(12.0);
                        ui.checkbox(&mut self.tracking.demo, "使用模拟演示数据");
                        muted(ui, "用于检查模型绑定，无需摄像头。");
                        ui.add_space(10.0);
                        if ui.small_button("查看面捕接入说明 ↗").clicked() {
                            self.page = Page::Output;
                        }
                    });
            });
    }
    fn show_output(&mut self, ctx: &egui::Context) {
        if !self.output {
            return;
        }
        let background = self.config.background.color();
        let texture = self.texture;
        let has_model = self.model.is_some();
        let size = self.config.output_size;
        let closed = self.output_closed.clone();
        let screenshot_requested = self.output_screenshot_requested.clone();
        let screenshot = if !screenshot_requested.load(std::sync::atomic::Ordering::Relaxed)
            && self.loading.is_none()
            && self.started.elapsed() > Duration::from_secs(3)
            && let Some(original) = self.launch.screenshot.as_ref()
        {
            let stem = original.file_stem().unwrap_or_default().to_string_lossy();
            Some((
                original.with_file_name(format!("{stem}-output.png")),
                self.output_screenshot_done.clone(),
            ))
        } else {
            None
        };
        // A deferred viewport owns its repaint/capture lifecycle, allowing the
        // broadcast window to keep painting when the control window is obscured.
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("ayakage-output"),
            egui::ViewportBuilder::default()
                .with_title(crate::OUTPUT_TITLE)
                .with_app_id("ayaayakage-output")
                .with_inner_size([size[0] as f32, size[1] as f32])
                .with_min_inner_size([256.0, 144.0])
                .with_transparent(true)
                .with_decorations(self.config.output_decorated),
            move |ui, _class| {
                if ui.input(|i| i.viewport().close_requested()) {
                    closed.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(background))
                    .show(ui, |ui| {
                        let outer = ui.max_rect();
                        if has_model {
                            let rect = egui::Rect::from_center_size(
                                outer.center(),
                                fit_size(outer.size(), size[0] as f32 / size[1] as f32),
                            );
                            ui.painter().image(
                                texture,
                                rect,
                                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        }
                    });
                // The control UI can replace this deferred callback before it
                // runs. Claim the shared request only when the output runs it.
                if let Some((path, done)) = &screenshot
                    && !screenshot_requested.swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    let path = path.clone();
                    let done = done.clone();
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::ScreenshotCallback::new(move |image| {
                                let bytes: Vec<u8> = image
                                    .pixels
                                    .iter()
                                    .flat_map(|pixel| pixel.to_srgba_unmultiplied())
                                    .collect();
                                match image::save_buffer(
                                    &path,
                                    &bytes,
                                    image.size[0] as u32,
                                    image.size[1] as u32,
                                    image::ColorType::Rgba8,
                                ) {
                                    Ok(()) => {
                                        log::info!("output screenshot saved: {}", path.display());
                                        done.store(true, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    Err(error) => log::error!("output screenshot failed: {error}"),
                                }
                            }),
                        ));
                }
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            },
        );
    }
}

impl eframe::App for Studio {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dropped = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path().to_owned()));
        if let Some(path) = dropped {
            self.load(path);
        }
        self.header(ui);
        self.navigation(ui);
        self.inspector(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(26))
            .show(ui, |ui| {
                if let Some(error) = self.error.clone() {
                    egui::Frame::new()
                        .fill(Color32::from_rgb(63, 33, 44))
                        .corner_radius(8)
                        .inner_margin(12)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.colored_label(Color32::LIGHT_RED, error);
                                if ui.small_button("×").clicked() {
                                    self.error = None;
                                }
                            });
                        });
                    ui.add_space(12.0);
                }
                if self.loading.is_some() && self.model.is_some() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("正在加载 {}", self.loading_name));
                    });
                }
                if let Some((notice, time)) = &self.notice
                    && time.elapsed() < Duration::from_secs(3)
                {
                    ui.label(RichText::new(format!("✓ {notice}")).size(12.0).color(GREEN));
                    ui.add_space(5.0);
                }
                match self.page {
                    Page::Stage => self.stage(ui),
                    Page::Mapping => self.mappings(ui),
                    Page::Expressions => self.expressions(ui),
                    Page::Output => self.output_settings(ui),
                }
            });
        self.show_output(&ctx);
        if !self.screenshot_requested
            && self.launch.screenshot.is_some()
            && self.loading.is_none()
            && self.started.elapsed() > Duration::from_secs(3)
        {
            self.screenshot_requested = true;
            let path = self.launch.screenshot.clone().unwrap();
            let done = self.screenshot_done.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                egui::ScreenshotCallback::new(move |image| {
                    let bytes: Vec<u8> = image
                        .pixels
                        .iter()
                        .flat_map(|pixel| pixel.to_array())
                        .collect();
                    match image::save_buffer(
                        &path,
                        &bytes,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    ) {
                        Ok(()) => {
                            log::info!("screenshot saved: {}", path.display());
                            done.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        Err(error) => log::error!("screenshot failed: {error}"),
                    }
                }),
            ));
        }
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
    fn on_exit(&mut self) {
        self.save();
    }
}

pub(super) fn setup_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "ayakage-ui".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/AyakageUI.otf"
        ))),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("ayakage-ui".into());
    }
    for path in [
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
    ] {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(egui::FontData::from_owned(data)));
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push("cjk".into());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = PANEL;
    style.visuals.window_fill = CARD;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.selection.bg_fill = Color32::from_rgb(73, 59, 111);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    style.visuals.widgets.inactive.bg_fill = CARD;
    style.visuals.widgets.inactive.weak_bg_fill = CARD;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, LINE);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(57, 51, 78);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(69, 58, 100);
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    style.spacing.slider_width = 170.0;
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(24.0));
    ctx.set_style_of(egui::Theme::Dark, style);
}
fn primary(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).strong().color(BG))
            .fill(ACCENT)
            .corner_radius(8)
            .min_size([120.0, 36.0].into()),
    )
}
fn muted(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).size(12.0).color(MUTED));
}
fn eyebrow(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).size(11.0).strong().color(MUTED));
}
fn badge(ui: &mut egui::Ui, label: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.12))
        .corner_radius(5)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(label).monospace().size(10.0).color(color));
        });
}
fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(CARD)
        .corner_radius(10)
        .inner_margin(18)
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 1.0).max(100.0));
            add(ui);
        });
}
fn range_edit(ui: &mut egui::Ui, range: &mut [f32; 2]) {
    ui.add(
        egui::DragValue::new(&mut range[0])
            .speed(0.1)
            .max_decimals(2)
            .range(-1000.0..=1000.0),
    );
    ui.label(RichText::new("~").color(MUTED));
    ui.add(
        egui::DragValue::new(&mut range[1])
            .speed(0.1)
            .max_decimals(2)
            .range(-1000.0..=1000.0),
    );
}
fn fit_size(available: Vec2, ratio: f32) -> Vec2 {
    let width = available.x.max(1.0).min(available.y.max(1.0) * ratio);
    Vec2::new(width, width / ratio)
}
fn checkerboard(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0, Color32::from_rgb(28, 30, 39));
    let cell = 18.0;
    for row in 0..(rect.height() / cell).ceil() as usize {
        for col in 0..(rect.width() / cell).ceil() as usize {
            if (row + col) % 2 == 0 {
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + Vec2::new(col as f32 * cell, row as f32 * cell),
                    Vec2::splat(cell),
                )
                .intersect(rect);
                painter.rect_filled(cell_rect, 0, Color32::from_rgb(33, 35, 45));
            }
        }
    }
}
pub(super) fn key_from_name(name: &str) -> Option<egui::Key> {
    Some(match name {
        "F1" => egui::Key::F1,
        "F2" => egui::Key::F2,
        "F3" => egui::Key::F3,
        "F4" => egui::Key::F4,
        "F5" => egui::Key::F5,
        "F6" => egui::Key::F6,
        "F7" => egui::Key::F7,
        "F8" => egui::Key::F8,
        "F9" => egui::Key::F9,
        "F10" => egui::Key::F10,
        "F11" => egui::Key::F11,
        "F12" => egui::Key::F12,
        _ => return None,
    })
}
fn hotkey_picker(ui: &mut egui::Ui, profile: &mut Profile, action: Action) {
    let current = profile
        .hotkeys
        .iter()
        .find(|h| h.action == action)
        .map(|h| h.key.clone())
        .unwrap_or_else(|| "—".into());
    let mut selected = current.clone();
    egui::ComboBox::from_id_salt(action.command())
        .selected_text(&selected)
        .width(65.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut selected, "—".into(), "无快捷键");
            for n in 1..=12 {
                let key = format!("F{n}");
                let reserved = profile
                    .hotkeys
                    .iter()
                    .any(|h| h.key == key && h.action != action);
                ui.add_enabled_ui(!reserved, |ui| {
                    ui.selectable_value(
                        &mut selected,
                        key.clone(),
                        if reserved {
                            format!("{key} · 已占用")
                        } else {
                            key
                        },
                    );
                });
            }
        });
    if selected != current {
        profile.hotkeys.retain(|h| h.action != action);
        if selected != "—" {
            profile.hotkeys.push(Hotkey {
                key: selected,
                action,
            });
        }
    }
}

fn paint_nav_icon(painter: &egui::Painter, center: egui::Pos2, page: Page, color: Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    let point = |x, y| center + Vec2::new(x, y);
    match page {
        Page::Stage => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(0.0, -8.0),
                    point(8.0, 0.0),
                    point(0.0, 8.0),
                    point(-8.0, 0.0),
                ],
                stroke,
            ));
            painter.circle_filled(center, 2.0, color);
        }
        Page::Mapping => {
            for (y, x) in [(-6.0, -3.0), (0.0, 4.0), (6.0, -1.0)] {
                painter.line_segment([point(-8.0, y), point(8.0, y)], stroke);
                painter.circle_filled(point(x, y), 2.5, color);
            }
        }
        Page::Expressions => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(0.0, -9.0),
                    point(2.5, -2.5),
                    point(9.0, 0.0),
                    point(2.5, 2.5),
                    point(0.0, 9.0),
                    point(-2.5, 2.5),
                    point(-9.0, 0.0),
                    point(-2.5, -2.5),
                ],
                stroke,
            ));
        }
        Page::Output => {
            painter.line_segment([point(-7.0, 7.0), point(7.0, -7.0)], stroke);
            painter.line_segment([point(-2.0, -7.0), point(7.0, -7.0)], stroke);
            painter.line_segment([point(7.0, -7.0), point(7.0, 2.0)], stroke);
        }
    }
}
