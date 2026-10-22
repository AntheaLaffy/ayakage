use crate::{
    config::{self, Action, Background, Config, Hotkey, Mapping, Profile, View},
    control::Control,
    model::Model,
    render::{RenderTarget, Renderer},
    tracking::{Channel, Tracking},
};
use anyhow::{Context, Result};
use eframe::egui;

mod ui;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use ui::{key_from_name, setup_theme};

#[derive(Default, PartialEq, Clone, Copy)]
enum Page {
    #[default]
    Stage,
    Mapping,
    Expressions,
    Output,
}

#[derive(Default)]
pub struct LaunchOptions {
    pub model: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub screenshot: Option<PathBuf>,
    pub smoke: bool,
    pub demo: bool,
    pub output: bool,
    pub smoke_success: Arc<std::sync::atomic::AtomicBool>,
}

pub struct Studio {
    config: Config,
    config_path: PathBuf,
    config_writable: bool,
    last_saved: String,
    saved_at: Instant,
    save_error: Option<String>,
    gpu: eframe::egui_wgpu::RenderState,
    target: RenderTarget,
    texture: egui::TextureId,
    renderer: Option<Renderer>,
    model: Option<Model>,
    loading: Option<mpsc::Receiver<Result<Model>>>,
    dialog: Option<mpsc::Receiver<Option<PathBuf>>>,
    loading_name: String,
    tracking: Tracking,
    control: Control,
    page: Page,
    manual_tab: bool,
    filter: String,
    output: bool,
    output_closed: Arc<std::sync::atomic::AtomicBool>,
    error: Option<String>,
    notice: Option<(String, Instant)>,
    port_edit: u16,
    last_tick: Instant,
    fps_start: Instant,
    frame_count: u32,
    fps: f32,
    launch: LaunchOptions,
    started: Instant,
    screenshot_requested: bool,
    screenshot_done: Arc<std::sync::atomic::AtomicBool>,
    output_screenshot_requested: Arc<std::sync::atomic::AtomicBool>,
    output_screenshot_done: Arc<std::sync::atomic::AtomicBool>,
}

impl Studio {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        launch: LaunchOptions,
        control: Control,
    ) -> Result<Self> {
        setup_theme(&cc.egui_ctx);
        let path = launch
            .config_path
            .clone()
            .unwrap_or_else(config::config_path);
        let (config, error, config_writable) =
            match Config::load_startup(&path, launch.config_path.is_none()) {
                Ok(config) => (config, None, true),
                Err(error) => (
                    Config::default(),
                    Some(format!(
                        "{error:#}。为保护原文件，自动保存已关闭；可在输出设置中备份并恢复。"
                    )),
                    false,
                ),
            };
        let gpu = cc.wgpu_render_state.clone().context("需要 wgpu 渲染后端")?;
        let target = RenderTarget::new(&gpu.device, config.output_size);
        let texture = gpu.renderer.write().register_native_texture(
            &gpu.device,
            &target.view,
            wgpu::FilterMode::Linear,
        );
        let tracking = Tracking::new(config.tracking.port);
        let last_saved = serde_json::to_string(&config)?;
        let model_path = launch.model.clone().or_else(|| config.last_model.clone());
        let mut studio = Self {
            port_edit: config.tracking.port,
            output: launch.output,
            output_closed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            config,
            config_path: path,
            config_writable,
            last_saved,
            saved_at: Instant::now(),
            save_error: None,
            gpu,
            target,
            texture,
            renderer: None,
            model: None,
            loading: None,
            dialog: None,
            loading_name: String::new(),
            tracking,
            control,
            page: Page::default(),
            manual_tab: false,
            filter: String::new(),
            error,
            notice: None,
            last_tick: Instant::now(),
            fps_start: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            started: Instant::now(),
            screenshot_requested: false,
            screenshot_done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            output_screenshot_requested: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            output_screenshot_done: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            launch,
        };
        studio.tracking.demo = studio.launch.demo;
        if let Some(path) = model_path {
            studio.load(path);
        }
        cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        Ok(studio)
    }
    fn load(&mut self, path: PathBuf) {
        if self.loading.is_some() {
            return;
        }
        self.loading_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let (tx, rx) = mpsc::channel();
        self.loading = Some(rx);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| Model::load(&path)).unwrap_or_else(|_| {
                Err(anyhow::anyhow!(
                    "模型解析器异常；请检查模型文件或尝试另一份导出"
                ))
            });
            let _ = tx.send(result);
        });
    }
    fn open_dialog(&mut self) {
        if self.dialog.is_some() || self.loading.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.dialog = Some(rx);
        std::thread::spawn(move || {
            let file = rfd::FileDialog::new()
                .set_title("导入 Live2D 模型 · 选择 .model3.json")
                .add_filter("Live2D 模型清单", &["json"])
                .pick_file();
            let _ = tx.send(file);
        });
    }
    fn profile(&self) -> Option<&Profile> {
        self.model
            .as_ref()
            .and_then(|m| self.config.profiles.get(&m.key))
    }
    fn execute(&mut self, action: Action) -> bool {
        let result = match action {
            Action::Calibrate => self
                .tracking
                .calibrate(&mut self.config.tracking)
                .map(|_| self.notify("已校准正面姿态")),
            Action::ToggleTracking => {
                self.config.tracking.enabled = !self.config.tracking.enabled;
                Ok(())
            }
            Action::ResetView => {
                if let Some(model) = &self.model
                    && let Some(profile) = self.config.profiles.get_mut(&model.key)
                {
                    profile.view = View::default();
                }
                Ok(())
            }
            Action::ClearExpression => {
                if let Some(model) = &mut self.model {
                    model.clear_expression();
                }
                Ok(())
            }
            Action::Expression(name) => self
                .model
                .as_mut()
                .context("请先导入模型")
                .and_then(|m| m.expression(&name)),
            Action::Motion(name) => self
                .model
                .as_mut()
                .context("请先导入模型")
                .and_then(|m| m.motion(&name)),
        };
        if let Err(error) = result {
            self.error = Some(format!("{error:#}"));
            false
        } else {
            true
        }
    }
    fn notify(&mut self, message: &str) {
        self.notice = Some((message.into(), Instant::now()));
    }
    fn save(&mut self) {
        if !self.config_writable {
            return;
        }
        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.last_saved = serde_json::to_string(&self.config).unwrap_or_default();
                self.saved_at = Instant::now();
                self.save_error = None;
            }
            Err(error) => self.save_error = Some(format!("配置保存失败：{error:#}")),
        }
        self.saved_at = Instant::now();
    }
    fn tick(&mut self, ctx: &egui::Context) {
        if self
            .output_closed
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            self.output = false;
        }
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32().clamp(0.0001, 0.1);
        self.last_tick = now;
        self.frame_count += 1;
        if self.fps_start.elapsed().as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32 / self.fps_start.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.fps_start = now;
        }
        if let Some(dialog) = &self.dialog
            && let Ok(path) = dialog.try_recv()
        {
            self.dialog = None;
            if let Some(path) = path {
                self.load(path);
            }
        }
        if let Some(receiver) = &self.loading
            && let Ok(result) = receiver.try_recv()
        {
            self.loading = None;
            match result.and_then(|model| {
                Renderer::new(&self.gpu.device, &self.gpu.queue, &model)
                    .map(|renderer| (model, renderer))
            }) {
                Ok((model, renderer)) => {
                    self.config
                        .profiles
                        .entry(model.key.clone())
                        .or_insert_with(|| model.default_profile());
                    self.config.last_model = Some(model.path.clone());
                    self.config.recent.retain(|p| p != &model.path);
                    self.config.recent.insert(0, model.path.clone());
                    self.config.recent.truncate(8);
                    self.renderer = Some(renderer);
                    self.model = Some(model);
                    self.error = None;
                    self.notify("模型已加载");
                }
                Err(error) => self.error = Some(format!("{error:#}")),
            }
        }
        self.tracking.update(&self.config.tracking, dt);
        for (action, sender) in self.control.poll() {
            let success = self.execute(action);
            self.control.reply(sender, success);
        }
        let mut actions = vec![];
        if !ctx.egui_wants_keyboard_input()
            && let Some(profile) = self.profile()
        {
            for hotkey in &profile.hotkeys {
                if let Some(key) = key_from_name(&hotkey.key)
                    && ctx.input(|i| i.key_pressed(key))
                {
                    actions.push(hotkey.action.clone());
                }
            }
        }
        for action in actions {
            self.execute(action);
        }
        if self.target.size != self.config.output_size {
            self.target = RenderTarget::new(&self.gpu.device, self.config.output_size);
            self.gpu
                .renderer
                .write()
                .update_egui_texture_from_wgpu_texture(
                    &self.gpu.device,
                    &self.target.view,
                    wgpu::FilterMode::Linear,
                    self.texture,
                );
        }
        if let (Some(model), Some(renderer)) = (&mut self.model, &mut self.renderer)
            && let Some(profile) = self.config.profiles.get(&model.key)
        {
            let result = model
                .update(profile, &self.tracking.values, dt)
                .and_then(|_| {
                    renderer.render(
                        &self.gpu.device,
                        &self.gpu.queue,
                        model,
                        &self.target,
                        &profile.view,
                    )
                });
            if let Err(error) = result {
                self.error = Some(format!("渲染失败：{error:#}"));
            }
        }
        if self.config_writable
            && self.saved_at.elapsed() > Duration::from_secs(2)
            && serde_json::to_string(&self.config)
                .ok()
                .is_some_and(|data| data != self.last_saved)
        {
            self.save();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        ctx.request_repaint_after(Duration::from_millis(16));
        if self.launch.smoke
            && self
                .screenshot_done
                .load(std::sync::atomic::Ordering::Relaxed)
            && (!self.launch.output
                || self
                    .output_screenshot_done
                    .load(std::sync::atomic::Ordering::Relaxed))
        {
            self.launch.smoke_success.store(
                self.error.is_none() && (self.launch.model.is_none() || self.model.is_some()),
                std::sync::atomic::Ordering::Relaxed,
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.launch.smoke && self.started.elapsed() > Duration::from_secs(45) {
            log::error!("smoke test timed out: {:?}", self.error);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
