use crate::{
    config::{Action, Hotkey, Mapping, Profile},
    tracking::Channel,
};
use anyhow::{Context, Result, bail};
use mocari::{
    ExpressionManager, ModelRuntime, MotionPlayer,
    assets::{RuntimeModel, load_model_runtime},
    expression::load_expression,
    motion::load_motion,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub struct Model {
    pub path: PathBuf,
    pub key: String,
    pub name: String,
    pub asset: RuntimeModel,
    pub expressions: Vec<(String, PathBuf)>,
    pub motions: Vec<(String, PathBuf)>,
    pub active_expression: Option<String>,
    pub active_motion: Option<String>,
    pub manual: BTreeMap<String, f32>,
    expression_player: ExpressionManager,
    motion_player: Option<MotionPlayer>,
    idle_player: Option<MotionPlayer>,
    time: f32,
}

pub fn resolve_model(path: &Path) -> Result<PathBuf> {
    let path = path
        .canonicalize()
        .with_context(|| format!("路径不存在：{}", path.display()))?;
    if path.is_dir() {
        let mut candidates = vec![];
        // Model exports normally put the manifest in the root or one child.
        for entry in std::fs::read_dir(&path)? {
            let child = entry?.path();
            if child.is_dir() {
                for nested in std::fs::read_dir(child)? {
                    let nested = nested?.path();
                    if is_manifest(&nested) {
                        candidates.push(nested);
                    }
                }
            } else if is_manifest(&child) {
                candidates.push(child);
            }
        }
        candidates.sort();
        if candidates.len() == 1 {
            return Ok(candidates.remove(0));
        }
        if candidates.is_empty() {
            bail!("目录中没有 .model3.json；请导入完整的 Live2D 导出目录");
        }
        bail!(
            "目录包含 {} 个模型，请直接选择要使用的 .model3.json",
            candidates.len()
        );
    }
    if !is_manifest(&path) {
        bail!("请选择 .model3.json；.moc3 还需要模型清单和纹理文件");
    }
    Ok(path)
}
fn is_manifest(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".model3.json"))
}

impl Model {
    pub fn load(path: &Path) -> Result<Self> {
        let path = resolve_model(path)?;
        let asset = load_model_runtime(&path)
            .with_context(|| format!("无法加载模型 {}", path.display()))?;
        let dir = path.parent().context("模型缺少父目录")?;
        let expressions = asset
            .runtime()
            .model()
            .expressions()
            .iter()
            .map(|r| (r.name().to_owned(), dir.join(r.file())))
            .collect();
        let motions: Vec<_> = asset
            .runtime()
            .model()
            .motions()
            .iter()
            .flat_map(|(group, references)| {
                references
                    .iter()
                    .enumerate()
                    .map(move |(i, r)| (format!("{group}/{i}"), dir.join(r.file())))
            })
            .collect();
        let idle_player = motions
            .iter()
            .find(|(name, _)| {
                name.split('/')
                    .next()
                    .is_some_and(|g| g.eq_ignore_ascii_case("idle"))
            })
            .and_then(|(_, path)| load_motion(path).ok())
            .map(|motion| MotionPlayer::with_looping(motion, true));
        if asset.runtime().meshes().is_empty() {
            bail!("模型没有可渲染的网格");
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .trim_end_matches(".model3.json")
            .to_owned();
        let key = path.to_string_lossy().into_owned();
        Ok(Self {
            path,
            key,
            name,
            asset,
            expressions,
            motions,
            active_expression: None,
            active_motion: None,
            manual: BTreeMap::new(),
            expression_player: ExpressionManager::new(),
            motion_player: None,
            idle_player,
            time: 0.0,
        })
    }
    pub fn runtime(&self) -> &ModelRuntime {
        self.asset.runtime()
    }
    pub fn default_mappings(&self) -> Vec<Mapping> {
        let templates = [
            (Channel::HeadYaw, "ParamAngleX", "PARAM_ANGLE_X"),
            (Channel::HeadPitch, "ParamAngleY", "PARAM_ANGLE_Y"),
            (Channel::HeadRoll, "ParamAngleZ", "PARAM_ANGLE_Z"),
            (Channel::EyeLeft, "ParamEyeLOpen", "PARAM_EYE_L_OPEN"),
            (Channel::EyeRight, "ParamEyeROpen", "PARAM_EYE_R_OPEN"),
            (Channel::BrowLeft, "ParamBrowLY", "PARAM_BROW_L_Y"),
            (Channel::BrowRight, "ParamBrowRY", "PARAM_BROW_R_Y"),
            (Channel::MouthOpen, "ParamMouthOpenY", "PARAM_MOUTH_OPEN_Y"),
            (Channel::MouthSmile, "ParamMouthForm", "PARAM_MOUTH_FORM"),
            (Channel::EyeX, "ParamEyeBallX", "PARAM_EYE_BALL_X"),
            (Channel::EyeY, "ParamEyeBallY", "PARAM_EYE_BALL_Y"),
            (Channel::BodyYaw, "ParamBodyAngleX", "PARAM_BODY_ANGLE_X"),
        ];
        templates
            .into_iter()
            .filter_map(|(source, modern, legacy)| {
                let parameter = self
                    .runtime()
                    .parameter_info(modern)
                    .or_else(|| self.runtime().parameter_info(legacy))?;
                let input = source.range();
                let output = if matches!(
                    source,
                    Channel::EyeLeft | Channel::EyeRight | Channel::MouthOpen
                ) {
                    [parameter.minimum().max(0.0), parameter.maximum().min(1.0)]
                } else {
                    [parameter.minimum(), parameter.maximum()]
                };
                Some(Mapping {
                    enabled: true,
                    source,
                    target: parameter.id().into(),
                    input,
                    output,
                })
            })
            .collect()
    }
    pub fn default_profile(&self) -> Profile {
        let mut profile = Profile {
            mappings: self.default_mappings(),
            ..Default::default()
        };
        for (i, (name, _)) in self.expressions.iter().take(4).enumerate() {
            profile.hotkeys.push(Hotkey {
                key: format!("F{}", i + 1),
                action: Action::Expression(name.clone()),
            });
        }
        for (i, (name, _)) in self
            .motions
            .iter()
            .filter(|(name, _)| !name.to_lowercase().starts_with("idle/"))
            .take(4)
            .enumerate()
        {
            profile.hotkeys.push(Hotkey {
                key: format!("F{}", i + 9),
                action: Action::Motion(name.clone()),
            });
        }
        profile
    }
    pub fn expression(&mut self, name: &str) -> Result<()> {
        if self.active_expression.as_deref() == Some(name) {
            self.clear_expression();
            return Ok(());
        }
        let (_, path) = self
            .expressions
            .iter()
            .find(|(n, _)| n == name)
            .with_context(|| format!("模型没有表情：{name}"))?;
        let expression =
            load_expression(path).with_context(|| format!("无法读取表情 {}", path.display()))?;
        self.expression_player.play(expression);
        self.active_expression = Some(name.into());
        Ok(())
    }
    pub fn clear_expression(&mut self) {
        self.expression_player.stop_all();
        self.active_expression = None;
    }
    pub fn motion(&mut self, name: &str) -> Result<()> {
        let (_, path) = self
            .motions
            .iter()
            .find(|(n, _)| n == name)
            .with_context(|| format!("模型没有动作：{name}"))?;
        let motion =
            load_motion(path).with_context(|| format!("无法读取动作 {}", path.display()))?;
        self.motion_player = Some(MotionPlayer::new_once(motion));
        self.active_motion = Some(name.into());
        Ok(())
    }
    pub fn update(&mut self, profile: &Profile, values: &[f32; 12], dt: f32) -> Result<()> {
        self.time += dt;
        let runtime = self.asset.runtime_mut();
        runtime.reset_parameters();
        runtime.reset_part_opacities();
        if let Some(player) = &mut self.motion_player {
            player.tick(dt);
            player.apply(runtime);
            if player.is_finished() {
                self.motion_player = None;
                self.active_motion = None;
            }
        } else if profile.idle
            && let Some(idle) = &mut self.idle_player
        {
            idle.tick(dt);
            idle.apply(runtime);
        }
        if profile.idle {
            runtime.set_parameter("ParamBreath", (self.time * 1.5).sin() * 0.5 + 0.5);
        }
        for mapping in &profile.mappings {
            if mapping.enabled {
                runtime.set_parameter(
                    &mapping.target,
                    mapping.value(values[mapping.source.index()]),
                );
            }
        }
        self.expression_player.tick(dt);
        self.expression_player.apply(runtime);
        if profile.physics {
            runtime.apply_physics(dt);
        }
        for (id, value) in &self.manual {
            runtime.set_parameter(id, *value);
        }
        runtime.apply_pose(dt);
        runtime.update_meshes().context("运行时无法更新模型网格")?;
        Ok(())
    }
}
