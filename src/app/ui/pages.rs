use super::*;

impl Studio {
    pub(super) fn stage(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("预览舞台");
                muted(ui, "你的形象，即将登场。");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                badge(
                    ui,
                    &format!("{} × {}", self.target.size[0], self.target.size[1]),
                    MUTED,
                );
                if self.model.is_some() {
                    badge(ui, &format!("{:.0} FPS", self.fps), GREEN);
                }
            });
        });
        ui.add_space(22.0);
        let available = ui.available_size();
        let canvas_height = (available.y - 80.0).max(120.0);
        egui::Frame::new()
            .fill(PANEL)
            .corner_radius(12)
            .stroke(egui::Stroke::new(1.0, LINE))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new((available.x - 2.0).max(100.0), canvas_height));
                let outer = ui.max_rect();
                let ratio = self.target.size[0] as f32 / self.target.size[1] as f32;
                let size = fit_size(outer.size() - Vec2::splat(32.0), ratio);
                let rect = egui::Rect::from_center_size(outer.center(), size);
                if self.model.is_some() {
                    checkerboard(ui.painter(), rect);
                    ui.painter().image(
                        self.texture,
                        rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    let response = ui.interact(
                        rect,
                        ui.scope_id().with("canvas"),
                        egui::Sense::click_and_drag(),
                    );
                    if response.double_clicked() {
                        self.execute(Action::ResetView);
                    }
                    if let Some(key) = self.model.as_ref().map(|m| m.key.clone())
                        && let Some(profile) = self.config.profiles.get_mut(&key)
                    {
                        if response.dragged() {
                            let delta = ui.input(|i| i.pointer.delta());
                            profile.view.pan[0] = (profile.view.pan[0]
                                + delta.x / rect.width() * 2.0)
                                .clamp(-4.0, 4.0);
                            profile.view.pan[1] = (profile.view.pan[1]
                                - delta.y / rect.height() * 2.0)
                                .clamp(-4.0, 4.0);
                        }
                        if response.hovered() {
                            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                            profile.view.zoom =
                                (profile.view.zoom * (scroll * 0.002).exp()).clamp(0.1, 6.0);
                        }
                    }
                    ui.painter().rect_stroke(
                        rect,
                        0,
                        egui::Stroke::new(1.0, LINE),
                        egui::StrokeKind::Inside,
                    );
                } else {
                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space((size.y * 0.5 - 110.0).max(12.0));
                            ui.label(RichText::new("K").size(42.0).color(ACCENT));
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(if self.loading.is_some() {
                                    "正在准备你的形象…"
                                } else {
                                    "从你的第一个模型开始"
                                })
                                .size(22.0)
                                .strong(),
                            );
                            ui.add_space(10.0);
                            muted(ui, "拖入模型文件夹，或选择 .model3.json");
                            muted(ui, "请保留同目录中的 .moc3、纹理及动作文件。");
                            ui.add_space(24.0);
                            if self.loading.is_some() {
                                ui.spinner();
                                muted(ui, &self.loading_name);
                            } else if primary(ui, "＋  导入 Live2D 模型").clicked() {
                                self.open_dialog();
                            }
                        });
                    });
                }
            });
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            muted(ui, "拖动平移 · 滚轮缩放 · 双击复位");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("复位").clicked() {
                    self.execute(Action::ResetView);
                }
                if let Some(key) = self.model.as_ref().map(|m| m.key.clone())
                    && let Some(profile) = self.config.profiles.get_mut(&key)
                {
                    ui.add(
                        egui::Slider::new(&mut profile.view.zoom, 0.1..=4.0)
                            .suffix(" ×")
                            .max_decimals(2)
                            .show_value(true),
                    );
                }
            });
        });
    }
    pub(super) fn mappings(&mut self, ui: &mut egui::Ui) {
        ui.heading("参数映射");
        muted(
            ui,
            "把面部动作连接到模型参数。调整范围或交换输出值可以反转方向。",
        );
        ui.add_space(18.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.manual_tab, false, "追踪映射");
            ui.selectable_value(&mut self.manual_tab, true, "手动参数检查");
        });
        ui.add_space(18.0);
        let Some(model) = &mut self.model else {
            muted(ui, "导入模型后，即可编辑映射和检查参数。");
            return;
        };
        let Some(profile) = self.config.profiles.get_mut(&model.key) else {
            return;
        };
        let parameters: Vec<_> = model
            .runtime()
            .parameter_infos()
            .map(|p| (p.id().to_owned(), p.minimum(), p.maximum(), p.value()))
            .collect();
        if self.manual_tab {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("搜索参数…")
                        .desired_width(260.0),
                );
                if ui.button("释放全部手动值").clicked() {
                    model.manual.clear();
                }
            });
            ui.add_space(10.0);
            muted(
                ui,
                "移动滑块后锁定该参数；点击释放恢复追踪、动作与物理驱动。",
            );
            ui.add_space(14.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (id, min, max, value) in &parameters {
                    if !id.to_lowercase().contains(&self.filter.to_lowercase()) {
                        continue;
                    }
                    ui.push_id(id, |ui| {
                        let locked = model.manual.contains_key(id);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(id).monospace().size(12.0).color(if locked {
                                ACCENT
                            } else {
                                TEXT
                            }));
                            if locked && ui.small_button("释放").clicked() {
                                model.manual.remove(id);
                            }
                        });
                        let mut edited = *value;
                        if parameter_slider(ui, &mut edited, *min..=*max).changed() {
                            model.manual.insert(id.clone(), edited);
                        }
                        ui.add_space(12.0);
                    });
                }
            });
        } else {
            let mut remove = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, mapping) in profile.mappings.iter_mut().enumerate() {
                    ui.push_id(i, |ui| {
                        egui::Frame::new()
                            .fill(CARD)
                            .corner_radius(9)
                            .inner_margin(14)
                            .show(ui, |ui| {
                                ui.set_min_width((ui.available_width() - 1.0).max(100.0));
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut mapping.enabled, "");
                                    egui::ComboBox::from_id_salt("source")
                                        .selected_text(mapping.source.label())
                                        .width(108.0)
                                        .show_ui(ui, |ui| {
                                            for source in Channel::ALL {
                                                ui.selectable_value(
                                                    &mut mapping.source,
                                                    source,
                                                    source.label(),
                                                );
                                            }
                                        });
                                    ui.label(RichText::new("→").color(ACCENT));
                                    egui::ComboBox::from_id_salt("target")
                                        .selected_text(&mapping.target)
                                        .width(180.0)
                                        .show_ui(ui, |ui| {
                                            for (id, _, _, _) in &parameters {
                                                ui.selectable_value(
                                                    &mut mapping.target,
                                                    id.clone(),
                                                    id,
                                                );
                                            }
                                        });
                                    if ui.small_button("×").clicked() {
                                        remove = Some(i);
                                    }
                                });
                                ui.add_space(10.0);
                                ui.horizontal_wrapped(|ui| {
                                    muted(ui, "输入");
                                    range_edit(ui, &mut mapping.input);
                                    ui.add_space(8.0);
                                    muted(ui, "输出");
                                    range_edit(ui, &mut mapping.output);
                                    if ui.small_button("⇄").on_hover_text("反转输出范围").clicked()
                                    {
                                        mapping.output.swap(0, 1);
                                    }
                                    ui.label(
                                        RichText::new(format!(
                                            "当前 {:.2} → {:.2}",
                                            self.tracking.values[mapping.source.index()],
                                            mapping.value(
                                                self.tracking.values[mapping.source.index()]
                                            )
                                        ))
                                        .size(11.0)
                                        .color(ACCENT),
                                    );
                                });
                                if model.runtime().parameter_index(&mapping.target).is_none() {
                                    ui.colored_label(
                                        Color32::LIGHT_RED,
                                        "目标参数不存在，请重新选择。",
                                    );
                                }
                            });
                        ui.add_space(9.0);
                    });
                }
                if let Some(i) = remove {
                    profile.mappings.remove(i);
                }
                ui.horizontal(|ui| {
                    if ui.button("＋ 添加映射").clicked() {
                        let target = parameters.first().map(|p| p.0.clone()).unwrap_or_default();
                        profile.mappings.push(Mapping {
                            target,
                            ..Default::default()
                        });
                    }
                    if ui.button("恢复自动映射").clicked() {
                        profile.mappings = model.default_mappings();
                    }
                });
            });
        }
    }
    pub(super) fn expressions(&mut self, ui: &mut egui::Ui) {
        ui.heading("表情与动作");
        muted(
            ui,
            "点击触发，再次点击同一表情可取消。快捷键在应用聚焦时生效。",
        );
        ui.add_space(22.0);
        let Some(model) = &self.model else {
            muted(ui, "导入模型后，这里会显示模型自带的表情与动作。");
            return;
        };
        let expressions = model.expressions.clone();
        let motions = model.motions.clone();
        let active_expression = model.active_expression.clone();
        let active_motion = model.active_motion.clone();
        let key = model.key.clone();
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            eyebrow(ui, "EXPRESSIONS / 表情");
            ui.add_space(12.0);
            if expressions.is_empty() {
                muted(ui, "该模型的清单未声明表情文件。");
            }
            for (name, _) in &expressions {
                let a = Action::Expression(name.clone());
                self.asset_row(
                    ui,
                    &key,
                    name,
                    a.clone(),
                    active_expression.as_deref() == Some(name),
                    &mut action,
                );
            }
            if ui.button("清除表情  ·  F8").clicked() {
                action = Some(Action::ClearExpression);
            }
            ui.add_space(26.0);
            eyebrow(ui, "MOTIONS / 动作");
            ui.add_space(12.0);
            if motions.is_empty() {
                muted(ui, "该模型的清单未声明动作文件。");
            }
            for (name, _) in &motions {
                self.asset_row(
                    ui,
                    &key,
                    name,
                    Action::Motion(name.clone()),
                    active_motion.as_deref() == Some(name),
                    &mut action,
                );
            }
            ui.add_space(22.0);
            ui.separator();
            ui.add_space(12.0);
            if let Some(profile) = self.config.profiles.get_mut(&key) {
                ui.checkbox(&mut profile.idle, "播放待机动作与呼吸");
                ui.checkbox(&mut profile.physics, "启用模型物理（头发、衣物等）");
            }
            ui.add_space(10.0);
            muted(
                ui,
                "面捕映射优先于动作的同名参数。要完整预览动作，可暂时关闭相应映射。",
            );
        });
        if let Some(action) = action {
            self.execute(action);
        }
    }
    fn asset_row(
        &mut self,
        ui: &mut egui::Ui,
        profile_key: &str,
        name: &str,
        action: Action,
        active: bool,
        triggered: &mut Option<Action>,
    ) {
        egui::Frame::new()
            .fill(CARD)
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 1.0).max(100.0));
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [50.0, 30.0],
                            egui::Button::new(if active { "●" } else { "▷" }).fill(if active {
                                Color32::from_rgb(61, 49, 90)
                            } else {
                                PANEL
                            }),
                        )
                        .clicked()
                    {
                        *triggered = Some(action.clone());
                    }
                    ui.label(RichText::new(name).size(14.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(profile) = self.config.profiles.get_mut(profile_key) {
                            hotkey_picker(ui, profile, action.clone());
                        }
                    });
                });
            });
        ui.add_space(8.0);
    }
    pub(super) fn output_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("输出与连接");
        muted(ui, "把干净的模型画面交给 OBS，控制界面留在这里。");
        ui.add_space(22.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            card(ui, |ui| {
                eyebrow(ui, "01 / 直播画面"); ui.add_space(12.0);
                ui.horizontal(|ui| { ui.label("画布");
                    egui::ComboBox::from_id_salt("resolution").selected_text(format!("{} × {}", self.config.output_size[0], self.config.output_size[1])).show_ui(ui, |ui| {
                        for size in [[1280, 720], [1920, 1080], [1080, 1080], [720, 1280]] {
                            ui.selectable_value(&mut self.config.output_size, size, format!("{} × {}", size[0], size[1]));
                        }
                    });
                });
                ui.add_space(10.0); ui.horizontal(|ui| {
                    ui.label("背景");
                    for background in [Background::Transparent, Background::Green, Background::Dark] {
                        ui.selectable_value(&mut self.config.background, background, background.label());
                    }
                }); ui.add_space(10.0);
                ui.checkbox(&mut self.config.output_decorated, "保留输出窗口标题栏（便于在 niri 中选择）");
                ui.add_space(10.0);
                if primary(ui, if self.output { "关闭输出窗口" } else { "↗  打开输出窗口" }).clicked() { self.output = !self.output; }
                ui.add_space(12.0);
                ui.label("在 OBS 中添加「屏幕捕获 (PipeWire)」，选择 Ayakage · Output 窗口。");
                muted(ui, "透明是否保留取决于 Portal / OBS 捕获链路。若变成黑底，选绿幕背景，在 OBS 添加色度键滤镜，关键颜色选绿色。");
            }); ui.add_space(14.0);
            card(ui, |ui| {
                eyebrow(ui, "02 / OPENSEEFACE 面捕"); ui.add_space(12.0);
                ui.label("在独立终端运行 OpenSeeFace，数据仅发送到本机："); ui.add_space(8.0);
                ui.label(RichText::new(format!("uv run facetracker.py -c 0 --ip 127.0.0.1 --port {}", self.config.tracking.port)).monospace().size(12.0));
                ui.add_space(10.0); muted(ui, "接收到人脸数据后，正视摄像头并点击右侧校准。接入步骤和环境依赖见 README.md。");
                ui.hyperlink_to("OpenSeeFace 项目 ↗", "https://github.com/emilianavt/OpenSeeFace");
            }); ui.add_space(14.0);
            card(ui, |ui| {
                eyebrow(ui, "03 / WAYLAND 快捷键"); ui.add_space(12.0);
                ui.label("全局快捷键可交给 niri，通过命令触发本机应用："); ui.add_space(8.0);
                let options = if self.control.port() == crate::control::PORT { String::new() } else { format!(" --control-port {}", self.control.port()) };
                ui.label(RichText::new(format!("ayakage{options} --command calibrate\nayakage{options} --command toggle-tracking\nayakage{options} --command expression:表情名")).monospace().size(12.0));
                ui.add_space(10.0); muted(ui, "无需授予 /dev/input 权限。F5 校准 · F6 暂停追踪 · F7 复位舞台 · F8 清除表情。");
            }); ui.add_space(14.0);
            card(ui, |ui| {
                eyebrow(ui, "04 / 配置"); ui.add_space(12.0);
                ui.label(self.config_path.display().to_string());
                muted(ui, "追踪与输出设置全局保存；映射、热键和视图按模型保存。"); ui.add_space(10.0);
                if self.config_writable {
                    if ui.button("立即保存").clicked() { self.save(); self.notify("设置已保存"); }
                } else if ui.button("备份旧配置并启用保存").clicked() {
                    let suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                    let backup = self.config_path.with_extension(format!("json.backup-{suffix}"));
                    match std::fs::copy(&self.config_path, &backup) {
                        Ok(_) => { self.config_writable = true; self.save(); self.notify("旧配置已备份，自动保存已恢复"); },
                        Err(error) => self.error = Some(format!("备份失败：{error}")),
                    }
                }
                if let Some(error) = &self.save_error { ui.colored_label(Color32::LIGHT_RED, error); }
            });
        });
    }
}

fn parameter_slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> egui::Response {
    // Rounding a moving runtime value during painting would report `changed`
    // and silently lock that parameter even though the user never edited it.
    ui.add(
        egui::Slider::new(value, range)
            .clamping(egui::SliderClamping::Edits)
            .max_decimals(2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn viewing_live_values_does_not_lock_or_round_parameters() {
        let ctx = egui::Context::default();
        for original in [13.081234, -3.364543, 0.000032, 29.99998] {
            let mut value = original;
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                assert!(!parameter_slider(ui, &mut value, -30.0..=30.0).changed());
            });
            // This test consumes UI state without a GPU renderer.
            output.textures_delta.clear();
            assert_eq!(value, original);
        }
    }
}
