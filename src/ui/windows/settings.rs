use eframe::egui::{self, Color32, ComboBox, Ui, Window};
use std::fmt::Display;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui::CursorIcon;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

use crate::{
    context::{AppCtx, FrameCtx},
    settings::{ColorDisplayFmtEnum, Settings},
    ui::{
        components::{DOUBLE_SPACE, HALF_SPACE, SPACE},
        traits::UiWindow,
    },
    APP_NAME,
};

use super::{WINDOW_X_OFFSET, WINDOW_Y_OFFSET};

const UI_SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.25..=5.0;

#[derive(Debug, Default)]
pub struct SettingsWindow {
    pub show: bool,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl UiWindow for SettingsWindow {
    fn toggle(&mut self) {
        self.show = !self.show;
        if !self.show {
            self.clear_error();
            self.clear_message();
        }
    }

    fn is_open(&self) -> bool {
        self.show
    }

    fn display(&mut self, ctx: &mut FrameCtx<'_>) {
        if self.is_open() {
            let offset = ctx.egui.style().spacing.slider_width * WINDOW_X_OFFSET;
            let mut show = true;
            let is_dark_mode = ctx.egui.style().visuals.dark_mode;
            Window::new("settings")
                .frame(super::default_frame(is_dark_mode))
                .open(&mut show)
                .default_pos((offset, WINDOW_Y_OFFSET))
                .show(ctx.egui, |ui| {
                    super::apply_default_style(ui, is_dark_mode);
                    if let Some(err) = &self.error {
                        ui.colored_label(Color32::RED, err);
                    }
                    if let Some(msg) = &self.message {
                        ui.colored_label(Color32::GREEN, msg);
                    }

                    self.ui_scale_slider(ctx.app, ui);
                    ui.add_space(HALF_SPACE);
                    self.color_formats(ctx.app, ui);
                    ui.add_space(SPACE);
                    ui.checkbox(&mut ctx.app.settings.cache_colors, "Cache colors");
                    ui.add_space(DOUBLE_SPACE);

                    self.save_settings_btn(ctx.app, ui);
                });

            if !show {
                self.show = false;
                self.clear_error();
                self.clear_message();
            }
        }
    }
}

impl SettingsWindow {
    fn set_error(&mut self, error: impl Display) {
        self.clear_message();
        self.error = Some(error.to_string());
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    fn set_message(&mut self, message: impl Display) {
        self.clear_error();
        self.message = Some(message.to_string());
    }

    fn clear_message(&mut self) {
        self.message = None;
    }

    fn save_settings_btn(&mut self, app_ctx: &mut AppCtx, _ui: &mut Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        if _ui
            .button("Save settings")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            if let Some(dir) = Settings::dir(APP_NAME) {
                if !dir.exists() {
                    if let Err(e) = fs::create_dir_all(&dir) {
                        self.set_error(e);
                    }
                }
                let path = dir.join("settings.yaml");
                if let Err(e) = app_ctx.settings.save(&path) {
                    self.set_error(e);
                } else {
                    self.set_message(format!("Successfully saved settings to {}", path.display()));
                }
            }
        }
    }

    fn color_formats(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
        ComboBox::from_label("Color display format")
            .selected_text(app_ctx.settings.color_display_format.as_ref())
            .show_ui(ui, |ui| {
                color_format_selection_fill(&mut app_ctx.settings.color_display_format, ui);
            });
        ui.add_space(HALF_SPACE);
        ComboBox::from_label("Color clipboard format")
            .selected_text(
                app_ctx
                    .settings
                    .color_clipboard_format
                    .as_ref()
                    .map(|f| f.as_ref())
                    .unwrap_or("Same as display"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut app_ctx.settings.color_clipboard_format,
                    None,
                    "Same as display",
                );
                color_format_selection_fill(&mut app_ctx.settings.color_clipboard_format, ui);
            });
        // ComboBox::from_label("Palette clipboard format")
        //     .selected_text(app_ctx.settings.palette_clipboard_format.as_ref())
        //     .show_ui(ui, |ui| {
        //         ui.selectable_value(
        //             &mut app_ctx.settings.palette_clipboard_format,
        //             PaletteFormat::Gimp,
        //             PaletteFormat::Gimp.as_ref(),
        //         );
        //         ui.selectable_value(
        //             &mut app_ctx.settings.palette_clipboard_format,
        //             PaletteFormat::HexList,
        //             PaletteFormat::HexList.as_ref(),
        //         );
        //         for (name, fmt) in app_ctx.settings.saved_palette_formats.clone() {
        //             ui.selectable_value(
        //                 &mut app_ctx.settings.palette_clipboard_format,
        //                 PaletteFormat::Custom(name.clone(), fmt),
        //                 name,
        //             );
        //         }
        //     });
        ui.checkbox(
            &mut app_ctx.settings.auto_copy_picked_color,
            "Auto copy picked color",
        );
        ui.add_space(HALF_SPACE);
        // ui.horizontal(|ui| {
        //     if ui.button("Color formats …").clicked() {
        //         self.custom_formats_window.show = true;
        //     }
        //     if ui.button("Palette formats …").clicked() {
        //         self.palette_formats_window.show = true;
        //     }
        // });
    }

    fn ui_scale_slider(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        ui.horizontal(|ui| {
            ui.label("UI Scale");
            let mut ppp = app_ctx.settings.pixels_per_point;
            let rsp = ui.add(egui::Slider::new(&mut ppp, UI_SCALE_RANGE));
            if !rsp.dragged() {
                app_ctx.settings.pixels_per_point = ppp;
            }
        });
    }
}

/// Fill the values for a color format selection.
///
/// Used to fill both the display and clipboard format selections.
fn color_format_selection_fill<T: From<ColorDisplayFmtEnum> + PartialEq>(
    fmt_ref: &mut T,
    ui: &mut Ui,
) {
    ui.selectable_value(
        fmt_ref,
        ColorDisplayFmtEnum::Hex.into(),
        ColorDisplayFmtEnum::Hex.as_ref(),
    );
    ui.selectable_value(
        fmt_ref,
        ColorDisplayFmtEnum::HexUppercase.into(),
        ColorDisplayFmtEnum::HexUppercase.as_ref(),
    );
    ui.selectable_value(
        fmt_ref,
        ColorDisplayFmtEnum::CssRgb.into(),
        ColorDisplayFmtEnum::CssRgb.as_ref(),
    );
    ui.selectable_value(
        fmt_ref,
        ColorDisplayFmtEnum::CssHsl.into(),
        ColorDisplayFmtEnum::CssHsl.as_ref(),
    );
}
