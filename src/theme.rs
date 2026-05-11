use std::{fs, sync::Arc};

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily};

pub const ACCENT: Color32 = Color32::from_rgb(110, 168, 255);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(142, 196, 255);
pub const BORDER: Color32 = Color32::from_rgb(53, 63, 84);
pub const PANEL: Color32 = Color32::from_rgb(15, 19, 30);
pub const SURFACE: Color32 = Color32::from_rgb(24, 30, 44);
pub const SURFACE_ELEVATED: Color32 = Color32::from_rgb(32, 39, 56);
pub const INPUT_BG: Color32 = Color32::from_rgb(19, 24, 36);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(160, 172, 196);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(239, 243, 255);

/// 应用统一深色主题和基础间距，保证桌面工具的长期使用体验。
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = PANEL;
    visuals.panel_fill = PANEL;
    visuals.faint_bg_color = SURFACE;
    visuals.extreme_bg_color = INPUT_BG;
    visuals.code_bg_color = INPUT_BG;
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.weak_text_color = Some(TEXT_MUTED);
    visuals.text_cursor.stroke.color = TEXT_PRIMARY;
    visuals.text_cursor.stroke.width = 1.5;
    visuals.window_stroke.color = BORDER;
    visuals.widgets.open.bg_fill = SURFACE_ELEVATED;
    visuals.widgets.open.weak_bg_fill = SURFACE_ELEVATED;
    visuals.widgets.open.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.weak_bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.inactive.bg_fill = SURFACE_ELEVATED;
    visuals.widgets.inactive.weak_bg_fill = INPUT_BG;
    visuals.widgets.inactive.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(36, 44, 64);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(30, 38, 56);
    visuals.widgets.hovered.fg_stroke.color = TEXT_PRIMARY;
    visuals.widgets.active.bg_fill = Color32::from_rgb(48, 58, 82);
    visuals.widgets.active.weak_bg_fill = Color32::from_rgb(40, 50, 72);
    visuals.widgets.active.fg_stroke.color = TEXT_PRIMARY;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke.color = TEXT_PRIMARY;
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.menu_corner_radius = CornerRadius::same(12);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
    visuals.widgets.active.corner_radius = CornerRadius::same(12);
    visuals.widgets.open.corner_radius = CornerRadius::same(12);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);
    style.spacing.indent = 18.0;
    style.spacing.window_margin = egui::Margin::same(18);
    style.visuals.hyperlink_color = ACCENT;
    ctx.set_style(style);

    apply_windows_cjk_font(ctx);
}

/// 返回统一卡片圆角，方便不同面板保持同一视觉语言。
pub fn card_radius() -> CornerRadius {
    CornerRadius::same(16)
}

/// 在 Windows 环境下优先加载常见中文字体，解决中文显示为方框的问题。
fn apply_windows_cjk_font(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    for (name, path) in windows_cjk_font_candidates() {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };

        let font_name = name.to_owned();
        fonts
            .font_data
            .insert(font_name.clone(), Arc::new(FontData::from_owned(bytes)));

        if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
            family.insert(0, font_name.clone());
        }
        if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
            family.insert(0, font_name);
        }

        ctx.set_fonts(fonts);
        return;
    }
}

/// 返回 Windows 常见中文字体候选路径，按推荐优先级排序。
fn windows_cjk_font_candidates() -> [(&'static str, &'static str); 6] {
    [
        (
            "MicrosoftYaHei",
            r"C:\Windows\Fonts\msyh.ttc",
        ),
        (
            "MicrosoftYaHeiBold",
            r"C:\Windows\Fonts\msyhbd.ttc",
        ),
        ("SimHei", r"C:\Windows\Fonts\simhei.ttf"),
        ("KaiTi", r"C:\Windows\Fonts\simkai.ttf"),
        ("NSimSun", r"C:\Windows\Fonts\NSimsun.ttf"),
        ("SimSun", r"C:\Windows\Fonts\simsun.ttc"),
    ]
}
