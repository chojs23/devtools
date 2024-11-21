use crate::{
    app::{
        windows::{self, WINDOW_X_OFFSET, WINDOW_Y_OFFSET},
        AppCtx,
    },
    // color::{ChromaticAdaptationMethod, ColorHarmony, Illuminant, PaletteFormat, RgbWorkingSpace},
    context::FrameCtx,
    settings::{ColorDisplayFmtEnum, Settings, BASE_DIR},
    ui::{DOUBLE_SPACE, HALF_SPACE, SPACE},
};

use eframe::egui::{self, Color32, ComboBox, Ui, Window};
use std::fmt::Display;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui::CursorIcon;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

// use crate::app::windows::{CustomFormatsWindow, PaletteFormatsWindow};

const UI_SCALE_RANGE: std::ops::RangeInclusive<f32> = 0.25..=5.0;

#[derive(Debug, Default)]
pub struct SettingsWindow {
    pub show: bool,
    pub error: Option<String>,
    pub message: Option<String>,
    selected_display_fmt: String,
    selected_clipboard_fmt: String,
    // pub custom_formats_window: CustomFormatsWindow,
    // pub palette_formats_window: PaletteFormatsWindow,
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

    pub fn display(&mut self, ctx: &mut FrameCtx<'_>) {
        if self.show {
            let offset = ctx.egui.style().spacing.slider_width * WINDOW_X_OFFSET;
            let mut show = true;
            let is_dark_mode = ctx.egui.style().visuals.dark_mode;
            Window::new("settings")
                .frame(windows::default_frame(is_dark_mode))
                .open(&mut show)
                .default_pos((offset, WINDOW_Y_OFFSET))
                .show(ctx.egui, |ui| {
                    windows::apply_default_style(ui, is_dark_mode);
                    if let Some(err) = &self.error {
                        ui.colored_label(Color32::RED, err);
                    }
                    if let Some(msg) = &self.message {
                        ui.colored_label(Color32::GREEN, msg);
                    }

                    self.ui_scale_slider(ctx.app, ui);
                    ui.add_space(HALF_SPACE);
                    self.color_formats(ctx.app, ui);
                    ui.add_space(HALF_SPACE);
                    // self.rgb_working_space(ctx.app, ui);
                    // ui.add_space(HALF_SPACE);
                    // self.illuminant(ctx.app, ui);
                    // ui.add_space(HALF_SPACE);
                    // self.chromatic_adaptation_method(ctx.app, ui);
                    // ui.add_space(HALF_SPACE);
                    // self.color_harmony(ctx.app, ui);
                    ui.add_space(HALF_SPACE);
                    ui.checkbox(&mut ctx.app.settings.cache_colors, "Cache colors");
                    ui.add_space(DOUBLE_SPACE);
                    // self.color_spaces(ctx.app, ui);
                    ui.add_space(SPACE);

                    self.save_settings_btn(ctx.app, ui);
                });

            if !show {
                self.show = false;
                self.clear_error();
                self.clear_message();
            }
        }
    }

    fn save_settings_btn(&mut self, app_ctx: &mut AppCtx, _ui: &mut Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        if _ui
            .button("Save settings")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            if let Some(dir) = Settings::dir(BASE_DIR) {
                if !dir.exists() {
                    if let Err(e) = fs::create_dir_all(&dir) {
                        self.set_error(e);
                    }
                }
                let path = dir.join(Settings::FILE_NAME);
                if let Err(e) = app_ctx.settings.save(&path) {
                    self.set_error(e);
                } else {
                    self.set_message(format!("Successfully saved settings to {}", path.display()));
                }
            }
        }
    }

    // fn color_harmony(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
    //     ComboBox::from_label("Color harmony")
    //         .selected_text(app_ctx.settings.harmony.as_ref())
    //         .show_ui(ui, |ui| {
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Complementary,
    //                 ColorHarmony::Complementary.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Triadic,
    //                 ColorHarmony::Triadic.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Tetradic,
    //                 ColorHarmony::Tetradic.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Analogous,
    //                 ColorHarmony::Analogous.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::SplitComplementary,
    //                 ColorHarmony::SplitComplementary.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Square,
    //                 ColorHarmony::Square.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.harmony,
    //                 ColorHarmony::Monochromatic,
    //                 ColorHarmony::Monochromatic.as_ref(),
    //             );
    //         });
    // }

    // fn color_spaces(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
    //     ui.label("Colors spaces:");
    //     ui.horizontal(|ui| {
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.rgb, "RGB");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.cmyk, "CMYK");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.hsv, "HSV");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.hsl, "HSL");
    //     });
    //     ui.add_space(SPACE);
    //     ui.label("CIE Color spaces:");
    //     ui.horizontal(|ui| {
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.luv, "Luv");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.lch_uv, "LCH(uv)");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.lab, "Lab");
    //         ui.checkbox(&mut app_ctx.settings.color_spaces.lch_ab, "LCH(ab)");
    //     });
    // }
    //
    // fn illuminant(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
    //     ComboBox::from_label("Illuminant")
    //         .selected_text(app_ctx.settings.illuminant.as_ref())
    //         .show_ui(ui, |ui| {
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::A,
    //                 Illuminant::A.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::B,
    //                 Illuminant::B.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::C,
    //                 Illuminant::C.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::D50,
    //                 Illuminant::D50.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::D55,
    //                 Illuminant::D55.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::D65,
    //                 Illuminant::D65.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::D75,
    //                 Illuminant::D75.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::E,
    //                 Illuminant::E.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::F2,
    //                 Illuminant::F2.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::F7,
    //                 Illuminant::F7.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.illuminant,
    //                 Illuminant::F11,
    //                 Illuminant::F11.as_ref(),
    //             );
    //         });
    // }
    //
    // fn chromatic_adaptation_method(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
    //     ComboBox::from_label("Chromatic adaptation method")
    //         .selected_text(app_ctx.settings.chromatic_adaptation_method.as_ref())
    //         .show_ui(ui, |ui| {
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.chromatic_adaptation_method,
    //                 ChromaticAdaptationMethod::Bradford,
    //                 ChromaticAdaptationMethod::Bradford.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.chromatic_adaptation_method,
    //                 ChromaticAdaptationMethod::VonKries,
    //                 ChromaticAdaptationMethod::VonKries.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.chromatic_adaptation_method,
    //                 ChromaticAdaptationMethod::XYZScaling,
    //                 ChromaticAdaptationMethod::XYZScaling.as_ref(),
    //             );
    //         });
    // }
    //
    // fn rgb_working_space(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
    //     ComboBox::from_label("RGB Working Space")
    //         .selected_text(app_ctx.settings.rgb_working_space.as_ref())
    //         .show_ui(ui, |ui| {
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::Adobe,
    //                 RgbWorkingSpace::Adobe.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::Apple,
    //                 RgbWorkingSpace::Apple.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::CIE,
    //                 RgbWorkingSpace::CIE.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::ECI,
    //                 RgbWorkingSpace::ECI.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::NTSC,
    //                 RgbWorkingSpace::NTSC.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::PAL,
    //                 RgbWorkingSpace::PAL.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::ProPhoto,
    //                 RgbWorkingSpace::ProPhoto.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::SRGB,
    //                 RgbWorkingSpace::SRGB.as_ref(),
    //             );
    //             ui.selectable_value(
    //                 &mut app_ctx.settings.rgb_working_space,
    //                 RgbWorkingSpace::WideGamut,
    //                 RgbWorkingSpace::WideGamut.as_ref(),
    //             );
    //         });
    // }

    fn color_formats(&mut self, app_ctx: &mut AppCtx, ui: &mut Ui) {
        ComboBox::from_label("Color display format")
            .selected_text(app_ctx.settings.color_display_format.as_ref())
            .show_ui(ui, |ui| {
                color_format_selection_fill(
                    &mut app_ctx.settings.color_display_format,
                    app_ctx.settings.saved_color_formats.keys(),
                    ui,
                );
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
                color_format_selection_fill(
                    &mut app_ctx.settings.color_clipboard_format,
                    app_ctx.settings.saved_color_formats.keys(),
                    ui,
                );
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
fn color_format_selection_fill<'a, T: From<ColorDisplayFmtEnum> + PartialEq>(
    fmt_ref: &mut T,
    customs: impl IntoIterator<Item = &'a String>,
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
    for custom in customs {
        ui.selectable_value(
            fmt_ref,
            ColorDisplayFmtEnum::Custom(custom.clone()).into(),
            format!("*{}", custom),
        );
    }
}
