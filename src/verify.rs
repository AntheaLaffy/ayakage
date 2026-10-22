use crate::{
    config::View,
    model::Model,
    render::{RenderTarget, Renderer},
    tracking::{Channel, FEATURE_OFFSET, PACKET_SIZE, Tracking, TrackingSettings},
};
use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{net::UdpSocket, path::Path};

pub fn check(path: &Path) -> Result<()> {
    let model = Model::load(path)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "model": model.name, "path": model.path,
            "parameters": model.runtime().parameter_infos().map(|p| json!({"id": p.id(), "min": p.minimum(), "max": p.maximum(), "default": p.default()})).collect::<Vec<_>>(),
            "drawables": model.runtime().meshes().len(), "textures": model.asset.textures().len(),
            "physics": model.runtime().physics().is_some(),
            "expressions": model.expressions.iter().map(|p| &p.0).collect::<Vec<_>>(),
            "motions": model.motions.iter().map(|p| &p.0).collect::<Vec<_>>(),
            "auto_mappings": model.default_mappings(),
        }))?
    );
    Ok(())
}

/// Uses an actual GPU render target and the same UDP decoder, model and renderer
/// as the desktop application. No display server or camera is required.
pub async fn render_test(path: &Path, output: &Path) -> Result<()> {
    std::fs::create_dir_all(output)?;
    let mut model = Model::load(path)?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: None,
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("ayakage.verify"),
            ..Default::default()
        })
        .await?;
    let mut renderer = Renderer::new(&device, &queue, &model)?;
    let target = RenderTarget::new(&device, [960, 720]);
    let mut profile = model.default_profile();
    profile.idle = false;
    profile.physics = false;
    let neutral = std::array::from_fn(|i| Channel::ALL[i].neutral());
    model.update(&profile, &neutral, 0.016)?;
    renderer.render(&device, &queue, &model, &target, &View::default())?;
    let baseline = target.read_pixels(&device, &queue)?;
    target.save_png(&device, &queue, &output.join("neutral.png"))?;
    let visible = baseline
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0)
        .count();
    let transparent = baseline
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] == 0)
        .count();
    let partial = baseline
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0 && p[3] < 255)
        .count();
    ensure!(visible > 1000, "模型没有足够可见像素");
    ensure!(transparent > 1000, "输出没有保留透明区域");

    // End-to-end check: construct the public OpenSeeFace wire format, transmit
    // it through localhost UDP, and feed the decoded channels into this model.
    let reserve = UdpSocket::bind("127.0.0.1:0")?;
    let port = reserve.local_addr()?.port();
    drop(reserve);
    let mut tracking = Tracking::new(port);
    ensure!(tracking.error.is_none(), "UDP 监听失败");
    let sender = UdpSocket::bind("127.0.0.1:0")?;
    let mut packet = vec![0u8; PACKET_SIZE];
    packet[28] = 1;
    packet[20..24].copy_from_slice(&1f32.to_le_bytes());
    packet[24..28].copy_from_slice(&1f32.to_le_bytes());
    packet[49..53].copy_from_slice(&180f32.to_le_bytes());
    packet[53..57].copy_from_slice(&25f32.to_le_bytes());
    packet[57..61].copy_from_slice(&90f32.to_le_bytes());
    packet[FEATURE_OFFSET + 48..FEATURE_OFFSET + 52].copy_from_slice(&0.8f32.to_le_bytes());
    sender.send_to(&packet, (std::net::Ipv4Addr::LOCALHOST, port))?;
    let settings = TrackingSettings {
        port,
        smoothing_ms: 0.0,
        ..Default::default()
    };
    tracking.update(&settings, 0.016);
    ensure!(tracking.connected(), "没有接收到 UDP 数据");
    model.update(&profile, &tracking.values, 0.016)?;
    renderer.render(&device, &queue, &model, &target, &View::default())?;
    let tracked = target.read_pixels(&device, &queue)?;
    target.save_png(&device, &queue, &output.join("tracked.png"))?;
    let changed = baseline
        .as_chunks::<4>()
        .0
        .iter()
        .zip(tracked.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a != b)
        .count();
    ensure!(
        changed > 100,
        "追踪数据没有驱动模型画面变化；请检查模型的参数映射"
    );
    let mut physics_changed_parameters = 0;
    if model.runtime().physics().is_some() {
        let mut simulated = model.runtime().clone();
        for _ in 0..60 {
            simulated.apply_physics(1.0 / 60.0);
        }
        physics_changed_parameters = model
            .runtime()
            .parameter_values()
            .iter()
            .zip(simulated.parameter_values())
            .filter(|(a, b)| (*a - *b).abs() > 0.001)
            .count();
        ensure!(physics_changed_parameters > 0, "模型物理没有更新参数");
    }
    let mut expression_changes = vec![];
    for (name, _) in model.expressions.clone() {
        model.clear_expression();
        model.update(&profile, &tracking.values, 1.0)?;
        model.expression(&name)?;
        model.update(&profile, &tracking.values, 1.0)?;
        renderer.render(&device, &queue, &model, &target, &View::default())?;
        let pixels = target.read_pixels(&device, &queue)?;
        let changed = tracked
            .as_chunks::<4>()
            .0
            .iter()
            .zip(pixels.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a != b)
            .count();
        if changed > 0 {
            target.save_png(&device, &queue, &output.join("expression.png"))?;
        }
        expression_changes.push(json!({"expression": name, "changed_pixels": changed}));
    }
    if !expression_changes.is_empty() {
        ensure!(
            expression_changes
                .iter()
                .any(|e| e["changed_pixels"].as_u64().unwrap_or(0) > 0),
            "模型的表情未改变画面"
        );
    }
    model.clear_expression();
    let mut motion_changes = None;
    if let Some((name, _)) = model.motions.first().cloned() {
        profile.mappings.clear();
        model.motion(&name)?;
        model.update(&profile, &neutral, 0.5)?;
        renderer.render(&device, &queue, &model, &target, &View::default())?;
        let pixels = target.read_pixels(&device, &queue)?;
        motion_changes = Some(
            baseline
                .as_chunks::<4>()
                .0
                .iter()
                .zip(pixels.as_chunks::<4>().0.iter())
                .filter(|(a, b)| a != b)
                .count(),
        );
        target.save_png(&device, &queue, &output.join("motion.png"))?;
    }
    let report = json!({
        "model": model.name, "adapter": adapter.get_info().name,
        "parameters": model.runtime().parameter_ids().len(), "drawables": model.runtime().meshes().len(),
        "visible_pixels": visible, "transparent_pixels": transparent, "partial_alpha_pixels": partial,
        "udp_received": tracking.packets, "tracking_changed_pixels": changed,
        "physics_changed_parameters": physics_changed_parameters,
        "expressions": expression_changes, "motion_changed_pixels": motion_changes,
        "artifacts": output,
    });
    let text = serde_json::to_string_pretty(&report)?;
    std::fs::write(output.join("report.json"), &text).context("无法写入验证报告")?;
    println!("{text}");
    Ok(())
}
