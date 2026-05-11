mod app;
mod theme;

use app::KeywordApp;
use eframe::egui;

/// 配置原生窗口并启动关键词桌面工具的 GUI 入口。
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Keyword Composer")
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
