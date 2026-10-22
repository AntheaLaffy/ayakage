use crate::tracking::{Channel, TrackingSettings};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Mapping {
    pub enabled: bool,
    pub source: Channel,
    pub target: String,
    pub input: [f32; 2],
    pub output: [f32; 2],
}
impl Default for Mapping {
    fn default() -> Self {
        Self {
            enabled: true,
            source: Channel::HeadYaw,
            target: "ParamAngleX".into(),
            input: [-30.0, 30.0],
            output: [-30.0, 30.0],
        }
    }
}
impl Mapping {
    pub fn value(&self, input: f32) -> f32 {
        let span = self.input[1] - self.input[0];
        if span.abs() < 0.0001 || !input.is_finite() {
            return self.output[0];
        }
        let t = ((input - self.input[0]) / span).clamp(0.0, 1.0);
        self.output[0] + t * (self.output[1] - self.output[0])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct View {
    pub zoom: f32,
    pub pan: [f32; 2],
}
impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: [0.0; 2],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Calibrate,
    ToggleTracking,
    ResetView,
    ClearExpression,
    Expression(String),
    Motion(String),
}
impl Action {
    pub fn command(&self) -> String {
        match self {
            Self::Calibrate => "calibrate".into(),
            Self::ToggleTracking => "toggle-tracking".into(),
            Self::ResetView => "reset-view".into(),
            Self::ClearExpression => "clear-expression".into(),
            Self::Expression(name) => format!("expression:{name}"),
            Self::Motion(name) => format!("motion:{name}"),
        }
    }
    pub fn from_command(command: &str) -> Result<Self> {
        match command.trim() {
            "calibrate" => Ok(Self::Calibrate),
            "toggle-tracking" => Ok(Self::ToggleTracking),
            "reset-view" => Ok(Self::ResetView),
            "clear-expression" => Ok(Self::ClearExpression),
            command if command.starts_with("expression:") && command.len() > 11 => {
                Ok(Self::Expression(command[11..].into()))
            }
            command if command.starts_with("motion:") && command.len() > 7 => {
                Ok(Self::Motion(command[7..].into()))
            }
            _ => bail!("未知命令：{command}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub key: String,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub mappings: Vec<Mapping>,
    pub view: View,
    pub hotkeys: Vec<Hotkey>,
    pub physics: bool,
    pub idle: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
            view: View::default(),
            hotkeys: vec![
                Hotkey {
                    key: "F5".into(),
                    action: Action::Calibrate,
                },
                Hotkey {
                    key: "F6".into(),
                    action: Action::ToggleTracking,
                },
                Hotkey {
                    key: "F7".into(),
                    action: Action::ResetView,
                },
                Hotkey {
                    key: "F8".into(),
                    action: Action::ClearExpression,
                },
            ],
            physics: true,
            idle: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Background {
    #[default]
    Transparent,
    Green,
    Dark,
}
impl Background {
    pub fn label(self) -> &'static str {
        match self {
            Self::Transparent => "透明",
            Self::Green => "绿幕",
            Self::Dark => "深色",
        }
    }
    pub fn color(self) -> egui::Color32 {
        match self {
            Self::Transparent => egui::Color32::TRANSPARENT,
            Self::Green => egui::Color32::from_rgb(0, 255, 0),
            Self::Dark => egui::Color32::from_rgb(17, 19, 26),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub last_model: Option<PathBuf>,
    pub recent: Vec<PathBuf>,
    pub profiles: BTreeMap<String, Profile>,
    pub tracking: TrackingSettings,
    pub background: Background,
    pub output_size: [u32; 2],
    pub output_decorated: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            last_model: None,
            recent: vec![],
            profiles: BTreeMap::new(),
            tracking: TrackingSettings::default(),
            background: Background::default(),
            output_size: [1280, 720],
            output_decorated: true,
        }
    }
}

pub fn config_path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kage-studio/settings.json")
}
impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path).with_context(|| format!("无法读取配置 {}", path.display()))?;
        let mut config: Self =
            serde_json::from_slice(&bytes).context("配置 JSON 无效；原文件已保留")?;
        if config.version != 1 {
            bail!("不支持配置版本 {}；原文件已保留", config.version);
        }
        config.tracking.smoothing_ms = config.tracking.smoothing_ms.clamp(0.0, 500.0);
        config.tracking.face_id = config.tracking.face_id.max(0);
        config.output_size = config.output_size.map(|s| s.clamp(256, 3840));
        for profile in config.profiles.values_mut() {
            profile.view.zoom = profile.view.zoom.clamp(0.1, 6.0);
            profile.view.pan = profile.view.pan.map(|n| n.clamp(-4.0, 4.0));
        }
        Ok(config)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let dir = path.parent().context("配置路径缺少父目录")?;
        fs::create_dir_all(dir)?;
        let data = serde_json::to_vec_pretty(self)?;
        let temporary = path.with_extension("json.tmp");
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&data)?;
        file.sync_all()?;
        fs::rename(&temporary, path).context("无法原子保存配置")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mapping_clamps_and_supports_inversion() {
        let mut mapping = Mapping::default();
        assert_eq!(mapping.value(90.0), 30.0);
        mapping.output = [30.0, -30.0];
        assert_eq!(mapping.value(-30.0), 30.0);
        assert_eq!(mapping.value(0.0), 0.0);
        mapping.input = [0.0, 0.0];
        assert!(mapping.value(10.0).is_finite());
    }
    #[test]
    fn config_round_trip_retains_model_mapping_and_hotkeys() {
        let dir = std::env::temp_dir().join(format!("kage-config-test-{}", std::process::id()));
        let path = dir.join("settings.json");
        let mut config = Config {
            last_model: Some(PathBuf::from("/models/a.model3.json")),
            background: Background::Green,
            output_decorated: false,
            output_size: [720, 1280],
            ..Default::default()
        };
        let mut profile = Profile {
            view: View {
                zoom: 2.0,
                pan: [0.25, -0.5],
            },
            ..Default::default()
        };
        profile.mappings.push(Mapping {
            enabled: false,
            output: [30.0, -30.0],
            ..Default::default()
        });
        profile.hotkeys.push(Hotkey {
            key: "F1".into(),
            action: Action::Expression("smile".into()),
        });
        config
            .profiles
            .insert("/models/a.model3.json".into(), profile);
        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.last_model, config.last_model);
        assert_eq!(loaded.background, Background::Green);
        assert!(!loaded.output_decorated);
        assert_eq!(loaded.output_size, [720, 1280]);
        let profile = &loaded.profiles["/models/a.model3.json"];
        assert_eq!(profile.view.zoom, 2.0);
        assert_eq!(profile.view.pan, [0.25, -0.5]);
        assert!(!profile.mappings[0].enabled);
        assert_eq!(profile.mappings[0].output, [30.0, -30.0]);
        assert_eq!(
            loaded
                .profiles
                .values()
                .next()
                .unwrap()
                .hotkeys
                .last()
                .unwrap()
                .action,
            Action::Expression("smile".into())
        );
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_file(path).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn malformed_config_is_reported_and_unchanged() {
        let path =
            std::env::temp_dir().join(format!("kage-bad-config-{}.json", std::process::id()));
        fs::write(&path, b"{bad}").unwrap();
        assert!(Config::load(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{bad}");
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn action_commands_round_trip() {
        for a in [
            Action::Calibrate,
            Action::ToggleTracking,
            Action::ResetView,
            Action::ClearExpression,
            Action::Expression("笑容 / 01".into()),
            Action::Motion("TapBody/0".into()),
        ] {
            assert_eq!(Action::from_command(&a.command()).unwrap(), a);
        }
        assert!(Action::from_command("expression:").is_err());
    }
}
