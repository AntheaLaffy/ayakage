use std::{
    collections::BTreeMap,
    net::UdpSocket,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const PACKET_SIZE: usize = 1785;
pub const FEATURE_OFFSET: usize = 1729;
pub const DEFAULT_PORT: u16 = 11573;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Channel {
    HeadYaw,
    HeadPitch,
    HeadRoll,
    EyeLeft,
    EyeRight,
    BrowLeft,
    BrowRight,
    MouthOpen,
    MouthSmile,
    EyeX,
    EyeY,
    BodyYaw,
}

impl Channel {
    pub const ALL: [Self; 12] = [
        Self::HeadYaw,
        Self::HeadPitch,
        Self::HeadRoll,
        Self::EyeLeft,
        Self::EyeRight,
        Self::BrowLeft,
        Self::BrowRight,
        Self::MouthOpen,
        Self::MouthSmile,
        Self::EyeX,
        Self::EyeY,
        Self::BodyYaw,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::HeadYaw => "头部左右",
            Self::HeadPitch => "头部上下",
            Self::HeadRoll => "头部倾斜",
            Self::EyeLeft => "左眼开合",
            Self::EyeRight => "右眼开合",
            Self::BrowLeft => "左眉高低",
            Self::BrowRight => "右眉高低",
            Self::MouthOpen => "嘴部开合",
            Self::MouthSmile => "嘴角 / 微笑",
            Self::EyeX => "视线左右",
            Self::EyeY => "视线上下",
            Self::BodyYaw => "身体左右",
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn neutral(self) -> f32 {
        if matches!(self, Self::EyeLeft | Self::EyeRight) {
            1.0
        } else {
            0.0
        }
    }
    pub fn range(self) -> [f32; 2] {
        match self {
            Self::HeadYaw | Self::HeadPitch | Self::HeadRoll | Self::BodyYaw => [-30.0, 30.0],
            Self::EyeLeft | Self::EyeRight | Self::MouthOpen => [0.0, 1.0],
            _ => [-1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct FaceFrame {
    pub face_id: i32,
    pub timestamp: f64,
    pub success: bool,
    pub euler: [f32; 3],
    pub eyes: [f32; 2],
    pub features: [f32; 14],
    pub gaze: [f32; 2],
}

fn float(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("validated packet length"),
    )
}

/// OpenSeeFace packs each scalar separately: the success byte has no padding.
/// See upstream facetracker.py, rather than treating this as a C-aligned struct.
pub fn decode_datagram(bytes: &[u8]) -> Result<Vec<FaceFrame>> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(PACKET_SIZE) {
        bail!(
            "OpenSeeFace 数据长度应为 {PACKET_SIZE} 的倍数，实际为 {}",
            bytes.len()
        );
    }
    bytes
        .as_chunks::<PACKET_SIZE>()
        .0
        .iter()
        .map(|packet| {
            let timestamp = f64::from_le_bytes(packet[0..8].try_into()?);
            let face_id = i32::from_le_bytes(packet[8..12].try_into()?);
            // Upstream sends eye_r first, then eye_l (tracker.py eye_blink).
            let eyes = [float(packet, 24), float(packet, 20)];
            let euler = [float(packet, 49), float(packet, 53), float(packet, 57)];
            let features = std::array::from_fn(|i| float(packet, FEATURE_OFFSET + i * 4));
            if !timestamp.is_finite()
                || face_id < 0
                || packet[28] > 1
                || !eyes
                    .iter()
                    .chain(&euler)
                    .chain(&features)
                    .all(|f| f.is_finite())
            {
                bail!("OpenSeeFace 数据含有无效数值");
            }
            // The iris landmarks are 66/67. Normalize against the corresponding eye
            // corners in the same packet so camera resolution does not affect gaze.
            let point = |i: usize| [float(packet, 345 + i * 8 + 4), float(packet, 345 + i * 8)];
            let mut gaze = [0.0; 2];
            for (iris, a, b) in [(66, 36, 39), (67, 42, 45)] {
                let (p, l, r) = (point(iris), point(a), point(b));
                let width = (r[0] - l[0]).abs();
                if width > 1.0 && p.iter().chain(&l).chain(&r).all(|f| f.is_finite()) {
                    gaze[0] += ((p[0] - (l[0] + r[0]) * 0.5) / width * 4.0).clamp(-1.0, 1.0) * 0.5;
                    gaze[1] += (-(p[1] - (l[1] + r[1]) * 0.5) / width * 6.0).clamp(-1.0, 1.0) * 0.5;
                }
            }
            Ok(FaceFrame {
                face_id,
                timestamp,
                success: packet[28] != 0,
                euler,
                eyes,
                features,
                gaze,
            })
        })
        .collect()
}

pub fn angle_delta(value: f32, center: f32) -> f32 {
    (value - center + 180.0).rem_euclid(360.0) - 180.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrackingSettings {
    pub port: u16,
    pub face_id: i32,
    pub enabled: bool,
    pub smoothing_ms: f32,
    pub center: Option<[f32; 3]>,
}
impl Default for TrackingSettings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            face_id: 0,
            enabled: true,
            smoothing_ms: 75.0,
            center: None,
        }
    }
}

pub struct Tracking {
    socket: Option<UdpSocket>,
    pub error: Option<String>,
    pub latest: Option<FaceFrame>,
    pub last_seen: Option<Instant>,
    pub packets: u64,
    pub rejected: u64,
    pub faces: BTreeMap<i32, Instant>,
    pub values: [f32; 12],
    pub demo: bool,
    start: Instant,
}

impl Tracking {
    pub fn new(port: u16) -> Self {
        let mut tracking = Self {
            socket: None,
            error: None,
            latest: None,
            last_seen: None,
            packets: 0,
            rejected: 0,
            faces: BTreeMap::new(),
            values: std::array::from_fn(|i| Channel::ALL[i].neutral()),
            demo: false,
            start: Instant::now(),
        };
        tracking.bind(port);
        tracking
    }
    pub fn bind(&mut self, port: u16) {
        // Release the previous socket before rebinding the same address.
        self.socket = None;
        self.latest = None;
        self.last_seen = None;
        self.error = None;
        let result = UdpSocket::bind((std::net::Ipv4Addr::LOCALHOST, port)).and_then(|socket| {
            socket.set_nonblocking(true)?;
            Ok(socket)
        });
        match result {
            Ok(socket) => self.socket = Some(socket),
            Err(error) => self.error = Some(format!("无法监听 127.0.0.1:{port}：{error}")),
        }
    }
    pub fn connected(&self) -> bool {
        self.last_seen
            .is_some_and(|t| t.elapsed() < Duration::from_millis(650))
    }
    pub fn calibrate(&self, settings: &mut TrackingSettings) -> Result<()> {
        if !self.connected() || self.demo {
            bail!("请先连接真实 OpenSeeFace 追踪数据");
        }
        let latest = self.latest.as_ref().context("没有可校准的追踪帧")?;
        settings.center = Some(latest.euler);
        Ok(())
    }
    pub fn update(&mut self, settings: &TrackingSettings, dt: f32) {
        let mut buffer = [0u8; 65535];
        // A busy sender must not starve the UI render loop.
        for _ in 0..128 {
            let Some(socket) = self.socket.as_ref() else {
                break;
            };
            let length = match socket.recv(&mut buffer) {
                Ok(length) => length,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    self.error = Some(error.to_string());
                    break;
                }
            };
            match decode_datagram(&buffer[..length]) {
                Ok(frames) => {
                    for frame in frames {
                        self.packets += 1;
                        self.faces.insert(frame.face_id, Instant::now());
                        if frame.face_id == settings.face_id && frame.success {
                            self.latest = Some(frame);
                            self.last_seen = Some(Instant::now());
                        }
                    }
                }
                Err(_) => self.rejected += 1,
            }
        }
        self.faces
            .retain(|_, seen| seen.elapsed() < Duration::from_secs(5));
        let mut target: [f32; 12] = std::array::from_fn(|i| Channel::ALL[i].neutral());
        if settings.enabled && self.demo {
            let t = self.start.elapsed().as_secs_f32();
            target[0] = t.sin() * 20.0;
            target[1] = (t * 0.6).sin() * 12.0;
            target[2] = (t * 0.8).sin() * 8.0;
            target[3] = 1.0 - ((t * 0.8).sin().max(0.97) - 0.97) / 0.03;
            target[4] = target[3];
            target[7] = (t * 2.5).sin().max(0.0) * 0.7;
            target[11] = target[0] * 0.25;
        } else if settings.enabled
            && self.connected()
            && let Some(frame) = &self.latest
        {
            // OpenSeeFace's OpenCV coordinate frame has a 90-degree roll at
            // rest (also compensated in its Unity/OpenSee.cs receiver).
            let center = settings.center.unwrap_or([180.0, 0.0, 90.0]);
            target[0] = angle_delta(frame.euler[1], center[1]).clamp(-90.0, 90.0);
            target[1] = -angle_delta(frame.euler[0], center[0]).clamp(-90.0, 90.0);
            target[2] = -angle_delta(frame.euler[2], center[2]).clamp(-90.0, 90.0);
            target[3] = frame.eyes[0].clamp(0.0, 1.0);
            target[4] = frame.eyes[1].clamp(0.0, 1.0);
            target[5] = frame.features[3].clamp(-1.0, 1.0);
            target[6] = frame.features[6].clamp(-1.0, 1.0);
            target[7] = frame.features[12].clamp(0.0, 1.0);
            target[8] = ((frame.features[8] + frame.features[10]) * 0.5).clamp(-1.0, 1.0);
            target[9] = frame.gaze[0];
            target[10] = frame.gaze[1];
            target[11] = target[0] * 0.25;
        }
        let alpha = if settings.smoothing_ms < 1.0 {
            1.0
        } else {
            1.0 - (-dt.max(0.0) / (settings.smoothing_ms / 1000.0)).exp()
        };
        for (value, target) in self.values.iter_mut().zip(target) {
            *value += (target - *value) * alpha;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet(id: i32) -> Vec<u8> {
        let mut b = vec![0u8; PACKET_SIZE];
        b[8..12].copy_from_slice(&id.to_le_bytes());
        b[28] = 1;
        b[49..53].copy_from_slice(&180f32.to_le_bytes());
        b[53..57].copy_from_slice(&25f32.to_le_bytes());
        b[57..61].copy_from_slice(&90f32.to_le_bytes());
        b[FEATURE_OFFSET + 48..FEATURE_OFFSET + 52].copy_from_slice(&0.7f32.to_le_bytes());
        b
    }
    #[test]
    fn decodes_unaligned_osf_and_multiple_faces() {
        let b = [packet(0), packet(2)].concat();
        let frames = decode_datagram(&b).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1].face_id, 2);
        assert_eq!(frames[0].euler, [180.0, 25.0, 90.0]);
        assert_eq!(frames[0].features[12], 0.7);
    }
    #[test]
    fn rejects_truncated_and_nonfinite_packets() {
        assert!(decode_datagram(&packet(0)[..PACKET_SIZE - 1]).is_err());
        let mut b = packet(0);
        b[53..57].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode_datagram(&b).is_err());
        assert!(decode_datagram(&[]).is_err());
    }
    #[test]
    fn keeps_left_and_right_eyes_in_anatomical_order() {
        let mut bytes = packet(0);
        bytes[20..24].copy_from_slice(&0.25f32.to_le_bytes());
        bytes[24..28].copy_from_slice(&0.9f32.to_le_bytes());
        assert_eq!(decode_datagram(&bytes).unwrap()[0].eyes, [0.9, 0.25]);
    }
    #[test]
    fn smoothing_is_independent_of_render_frequency() {
        let mut sixty = Tracking::new(0);
        let mut one_twenty = Tracking::new(0);
        let frame = decode_datagram(&packet(0)).unwrap().remove(0);
        sixty.latest = Some(frame.clone());
        one_twenty.latest = Some(frame);
        sixty.last_seen = Some(Instant::now());
        one_twenty.last_seen = Some(Instant::now());
        let settings = TrackingSettings::default();
        for _ in 0..60 {
            sixty.update(&settings, 1.0 / 60.0);
        }
        for _ in 0..120 {
            one_twenty.update(&settings, 1.0 / 120.0);
        }
        assert!((sixty.values[0] - one_twenty.values[0]).abs() < 0.001);
        assert!(sixty.values[0] > 24.0);
    }
    #[test]
    fn calibration_wraps_at_180_degrees() {
        assert_eq!(angle_delta(-179.0, 179.0), 2.0);
        assert_eq!(angle_delta(179.0, -179.0), -2.0);
    }
    #[test]
    fn receives_real_udp_and_returns_to_neutral_after_loss() {
        let mut tracking = Tracking::new(0);
        let addr = tracking.socket.as_ref().unwrap().local_addr().unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender.send_to(&packet(0), addr).unwrap();
        let mut settings = TrackingSettings {
            smoothing_ms: 0.0,
            ..Default::default()
        };
        tracking.update(&settings, 0.016);
        assert!(tracking.connected());
        assert_eq!(tracking.values[0], 25.0);
        tracking.calibrate(&mut settings).unwrap();
        tracking.update(&settings, 0.016);
        assert_eq!(tracking.values[0], 0.0);
        tracking.last_seen = Some(Instant::now() - Duration::from_secs(1));
        tracking.update(&settings, 0.016);
        assert_eq!(tracking.values[3], 1.0);
        assert_eq!(tracking.values[7], 0.0);
    }
    #[test]
    fn selected_face_ignores_other_faces_and_unsuccessful_fits() {
        let mut tracking = Tracking::new(0);
        let addr = tracking.socket.as_ref().unwrap().local_addr().unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let settings = TrackingSettings {
            face_id: 2,
            smoothing_ms: 0.0,
            ..Default::default()
        };
        sender.send_to(&packet(0), addr).unwrap();
        tracking.update(&settings, 0.016);
        assert!(!tracking.connected());
        let mut selected = packet(2);
        selected[53..57].copy_from_slice(&(-15f32).to_le_bytes());
        sender
            .send_to(&[packet(0), selected.clone()].concat(), addr)
            .unwrap();
        tracking.update(&settings, 0.016);
        assert_eq!(tracking.latest.as_ref().unwrap().face_id, 2);
        assert_eq!(tracking.values[0], -15.0);
        selected[28] = 0;
        selected[53..57].copy_from_slice(&40f32.to_le_bytes());
        sender.send_to(&selected, addr).unwrap();
        tracking.update(&settings, 0.016);
        assert_eq!(tracking.values[0], -15.0);
    }
}
