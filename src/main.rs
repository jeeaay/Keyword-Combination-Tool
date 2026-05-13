#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod theme;

use app::KeywordApp;
use eframe::egui;

/// 配置原生窗口并启动关键词桌面工具的 GUI 入口。
fn main() -> eframe::Result {
    // 加载图标数据
    let icon_data = include_bytes!("../assets/keyword.png");
    let icon = eframe::icon_data::from_png_bytes(icon_data).expect("Failed to load icon");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Keyword Composer")
            .with_icon(icon)
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([1024.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Keyword Composer",
        options,
        Box::new(|cc| Ok(Box::new(KeywordApp::new(cc)))),
    )
}
