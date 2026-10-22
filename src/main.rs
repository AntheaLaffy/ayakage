mod app;
mod config;
mod control;
mod model;
mod render;
mod tracking;
mod verify;

use anyhow::{Context, Result, bail};
use std::path::PathBuf;

const PRODUCT_NAME: &str = "绫影 · Ayakage";
const OUTPUT_TITLE: &str = "Ayakage · Output";

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,ayakage=info"),
    )
    .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut launch = app::LaunchOptions::default();
    let mut check = None;
    let mut render_test = None;
    let mut output_dir = PathBuf::from(".local/verification");
    let mut command = None;
    let mut control_port = control::PORT;
    let mut i = 0;
    while i < args.len() {
        let value = |i: usize| args.get(i + 1).cloned().context("选项缺少参数");
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "{PRODUCT_NAME} {} · Linux Live2D VTuber studio\n\n使用方法：ayakage [选项]\n  --model PATH         打开 .model3.json 或模型目录\n  --output             同时打开独立直播输出窗口\n  --demo               使用明确标注的模拟追踪数据\n  --config PATH        使用独立配置文件\n  --command ACTION     向运行中的应用发送快捷键命令\n  --control-port PORT  本机命令端口（默认 11574）\n  --check-model PATH   检查真实模型文件及其参数\n  --render-test PATH   无窗口 GPU 渲染及追踪驱动验证\n  --output-dir PATH    渲染验证产物目录\n  --screenshot PATH    保存启动后的界面截图\n  --smoke-test         截图完成后退出（配合 --screenshot）\n  --version            显示版本\n\n命令：calibrate, toggle-tracking, reset-view, clear-expression,\n      expression:NAME, motion:GROUP/INDEX",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(());
            }
            "--version" | "-V" => {
                println!("ayakage {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--model" => {
                launch.model = Some(value(i)?.into());
                i += 1;
            }
            "--config" => {
                launch.config_path = Some(value(i)?.into());
                i += 1;
            }
            "--screenshot" => {
                launch.screenshot = Some(value(i)?.into());
                i += 1;
            }
            "--output-dir" => {
                output_dir = value(i)?.into();
                i += 1;
            }
            "--check-model" => {
                check = Some(PathBuf::from(value(i)?));
                i += 1;
            }
            "--render-test" => {
                render_test = Some(PathBuf::from(value(i)?));
                i += 1;
            }
            "--command" => {
                command = Some(value(i)?);
                i += 1;
            }
            "--control-port" => {
                control_port = value(i)?.parse().context("命令端口应为 1–65535 的整数")?;
                anyhow::ensure!(control_port != 0, "命令端口不能为 0");
                i += 1;
            }
            "--demo" => launch.demo = true,
            "--output" => launch.output = true,
            "--smoke-test" => launch.smoke = true,
            argument if !argument.starts_with('-') && launch.model.is_none() => {
                launch.model = Some(argument.into())
            }
            argument => bail!("未知参数：{argument}；使用 --help 查看用法"),
        }
        i += 1;
    }
    if let Some(command) = command {
        return control::send(&command, control_port);
    }
    if let Some(path) = check {
        return verify::check(&path);
    }
    if let Some(path) = render_test {
        return pollster::block_on(verify::render_test(&path, &output_dir));
    }
    if launch.smoke && launch.screenshot.is_none() {
        bail!("--smoke-test 需要 --screenshot PATH");
    }
    if let Some(path) = &launch.screenshot
        && let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let control = control::Control::new(control_port)?;
    let smoke = launch.smoke;
    let smoke_success = launch.smoke_success.clone();
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title(PRODUCT_NAME)
            .with_app_id("ayakage")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1160.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        PRODUCT_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::Studio::new(cc, launch, control)?))),
    )
    .map_err(|error| anyhow::anyhow!("无法启动 Ayakage：{error}"))?;
    anyhow::ensure!(
        !smoke || smoke_success.load(std::sync::atomic::Ordering::Relaxed),
        "窗口检查未完成或模型加载失败，请查看日志"
    );
    Ok(())
}
