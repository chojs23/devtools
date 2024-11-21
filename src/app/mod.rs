pub mod ui;

mod windows;

use std::sync::RwLock;

use eframe::{
    egui::{self, Button, CursorIcon, Layout, Margin, Rgba, ScrollArea, Ui, Visuals},
    CreationContext, Theme,
};
use once_cell::sync::{Lazy, OnceCell};

use windows::SettingsWindow;

static ADD_DESCRIPTION: &str = "Add this color to saved colors";

use crate::{
    app::colors::*,
    context::{AppCtx, FrameCtx},
    error::{append_global_error, DisplayError, ERROR_STACK},
    render::TextureManager,
    save_to_clipboard,
    screen_size::ScreenSize,
    ui::*,
    zoom_picker::ZoomPicker,
};

use self::colorbox::ColorBox;

pub static LIGHT_VISUALS: Lazy<Visuals> = Lazy::new(light_visuals);
pub static DARK_VISUALS: Lazy<Visuals> = Lazy::new(dark_visuals);
pub static TEXTURE_MANAGER: Lazy<RwLock<TextureManager>> =
    Lazy::new(|| RwLock::new(TextureManager::default()));
pub static CONTEXT: OnceCell<RwLock<AppCtx>> = OnceCell::new();

static ERROR_DISPLAY_DURATION: u64 = 10;

pub const CURRENT_COLOR_BOX_SIZE: f32 = 40.0;
pub const DEFAULT_PIXELS_PER_POINT: f32 = 1.0;

#[derive(Clone, Debug)]
pub enum CentralPanelTab {
    Jwt,
    ColorPicker,
}

#[derive(Default)]
pub struct Windows {
    pub settings: SettingsWindow,
    // pub help: HelpWindow,
}

pub struct App {
    pub windows: Windows,
    pub display_errors: Vec<DisplayError>,
    pub zoom_picker: ZoomPicker,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(mut app_ctx) = CONTEXT.get().and_then(|ctx| ctx.write().ok()) {
            let res = TEXTURE_MANAGER.try_write();
            if let Err(e) = &res {
                append_global_error(e);
                return;
            }
            let mut tex_manager = res.unwrap();

            let mut ctx = FrameCtx {
                app: &mut app_ctx,
                egui: ctx,
                tex_manager: &mut tex_manager,
                frame: Some(frame),
            };
            // ctx.egui.output().cursor_icon = ctx.app.cursor_icon;

            let screen_size = ScreenSize::from(ctx.egui.available_rect());
            if ctx.app.screen_size != screen_size {
                ctx.set_styles(screen_size);
            }

            ctx.egui
                .set_pixels_per_point(ctx.app.settings.pixels_per_point);
            ctx.app.check_settings_change();

            self.top_panel(&mut ctx);

            self.central_panel(&mut ctx);

            self.display_windows(&mut ctx);

            // #[cfg(not(target_arch = "wasm32"))]
            // ctx.set_window_size(ctx.egui.used_size());

            if let Ok(mut stack) = ERROR_STACK.try_lock() {
                while let Some(error) = stack.errors.pop_front() {
                    self.display_errors.push(error);
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if !ctx.egui.is_pointer_over_area() {
                // This paint request makes sure that the color displayed as color under cursor
                // gets updated even when the pointer is not in the egui window area.
                ctx.egui.request_repaint();

                if ctx.app.zoom_window_dragged {
                    // When zooming we want to continually repaint for smooth experience
                    // even if the pointer is not over main window area
                    return;
                }

                // Otherwise sleep to save some cycles
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            ctx.app.current_selected_color = ctx.app.picker.current_color;
        }
    }
}

impl App {
    pub fn init(context: &CreationContext) -> Box<dyn eframe::App + 'static> {
        let mut app_ctx = AppCtx::new(context);

        let app = Box::new(Self {
            windows: Windows::default(),
            display_errors: Default::default(),
            zoom_picker: ZoomPicker::default(),
        });

        let prefer_dark = context
            .integration_info
            .system_theme
            .map(|t| matches!(t, Theme::Dark))
            .unwrap_or(true);

        if let Ok(mut tex_manager) = TEXTURE_MANAGER.write() {
            let mut ctx = FrameCtx {
                app: &mut app_ctx,
                egui: &context.egui_ctx,
                tex_manager: &mut tex_manager,
                frame: None,
            };

            ctx.app.load_palettes(context.storage);

            if prefer_dark {
                ctx.set_dark_theme();
            } else {
                ctx.set_light_theme();
            }
        }

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Firacode".to_string(),
            egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/FiraCode/FiraCode-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "Firacode".to_owned());

        context.egui_ctx.set_fonts(fonts);

        // if app_ctx.settings.pixels_per_point == DEFAULT_PIXELS_PER_POINT {
        //     app_ctx.settings.pixels_per_point = context
        //         .integration_info
        //         .native_pixels_per_point
        //         .unwrap_or(DEFAULT_PIXELS_PER_POINT);
        // }

        CONTEXT.try_insert(RwLock::new(app_ctx)).unwrap();

        app
    }

    fn top_ui(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        ui.horizontal(|ui| {
            macro_rules! add_button_if {
                ($text:expr, $condition:expr, $block:tt) => {
                    add_button_if!($text, $condition, $block, $block);
                };
                ($text:expr, $condition:expr, $block_a:tt, $block_b:tt) => {
                    if $condition {
                        if ui
                            .button($text)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        $block_a;
                    } else {
                        let btn = Button::new($text).fill(Rgba::from_black_alpha(0.));
                        if ui
                            .add(btn)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        $block_b;
                    }
                };
            }
            add_button_if!(
                "JWT",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Jwt),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Jwt;
                }
            );
            add_button_if!(
                "ColorPicker",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::ColorPicker),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::ColorPicker;
                    ctx.app.sidepanel.show = false;
                }
            );

            ui.add_space(DOUBLE_SPACE);

            // add_button_if!(
            //     "hues",
            //     self.windows.hues.is_open,
            //     { self.windows.hues.is_open = false },
            //     { self.windows.hues.is_open = true }
            // );
            // add_button_if!(
            //     "shades",
            //     self.windows.shades.is_open,
            //     { self.windows.shades.is_open = false },
            //     { self.windows.shades.is_open = true }
            // );
            // add_button_if!(
            //     "tints",
            //     self.windows.tints.is_open,
            //     { self.windows.tints.is_open = false },
            //     { self.windows.tints.is_open = true }
            // );

            ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                // if ui
                //     .button(icon::HELP)
                //     .on_hover_text("Show help")
                //     .on_hover_cursor(CursorIcon::Help)
                //     .clicked()
                // {
                //     self.windows.help.toggle_window();
                // }
                if ui
                    .button(icon::EXPAND)
                    .on_hover_text("Show/hide side panel")
                    .on_hover_cursor(CursorIcon::ResizeHorizontal)
                    .clicked()
                {
                    ctx.app.sidepanel.show = !ctx.app.sidepanel.show;
                }
                if ui
                    .button(icon::SETTINGS)
                    .on_hover_text("Settings")
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.windows.settings.show = true;
                }
                self.dark_light_switch(ctx, ui);
            });
        });
    }

    fn dark_light_switch(&mut self, ctx: &mut FrameCtx, ui: &mut Ui) {
        let btn = if ctx.is_dark_mode() {
            icon::LIGHT_MODE
        } else {
            icon::DARK_MODE
        };
        if ui
            .button(btn)
            .on_hover_text("Switch ui color theme")
            .on_hover_cursor(CursorIcon::PointingHand)
            .clicked()
        {
            ctx.set_theme();
        }
    }

    fn display_windows(&mut self, ctx: &mut FrameCtx<'_>) {
        self.windows.settings.display(ctx);
        // self.windows.settings.custom_formats_window.display(
        //     &mut ctx.app.settings,
        //     ctx.egui,
        //     ctx.app.picker.current_color,
        // );
        // self.windows.settings.palette_formats_window.display(ctx);
        // if let Err(e) = self.windows.export.display(ctx) {
        //     append_global_error(e);
        // }

        // self.shades_window(ctx);
        // self.tints_window(ctx);
        // self.hues_window(ctx);
        // self.windows.help.display(ctx.egui);
    }

    fn top_panel(&mut self, ctx: &mut FrameCtx<'_>) {
        let frame = egui::Frame {
            fill: if ctx.egui.style().visuals.dark_mode {
                *D_BG_00
            } else {
                *L_BG_0
            },
            inner_margin: Margin::symmetric(15., 10.),
            ..Default::default()
        };
        egui::TopBottomPanel::top("top panel")
            .frame(frame)
            .show(ctx.egui, |ui| {
                self.top_ui(ctx, ui);
            });
    }

    fn central_panel(&mut self, ctx: &mut FrameCtx<'_>) {
        let _frame = egui::Frame {
            fill: if ctx.egui.style().visuals.dark_mode {
                *D_BG_0
            } else {
                *L_BG_2
            },

            inner_margin: Margin {
                left: 10.,
                top: 5.,
                right: 0.,
                bottom: 0.,
            },
            ..Default::default()
        };

        egui::CentralPanel::default()
            .frame(_frame)
            .show(ctx.egui, |ui| match ctx.app.central_panel_tab {
                CentralPanelTab::Jwt => self.jwt_ui(ctx, ui),
                CentralPanelTab::ColorPicker => self.color_picker_ui(ctx, ui),
            });
    }

    fn color_picker_ui(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut egui::Ui) {
        let mut top_padding = 0.;
        let mut err_idx = 0;
        self.error_ui(ctx, ui);

        ui.horizontal(|ui| {
            ui.add_space(HALF_SPACE);
            // if ctx.app.settings.harmony_display_box {
            //     self.display_harmonies(ctx, ui);
            // }
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Current color: ");
                    if ui
                        .button(icon::COPY)
                        .on_hover_text("Copy color to clipboard")
                        .on_hover_cursor(CursorIcon::Alias)
                        .clicked()
                    {
                        if let Err(e) = save_to_clipboard(
                            ctx.app.clipboard_color(&ctx.app.picker.current_color),
                        ) {
                            append_global_error(format!(
                                "Failed to save color to clipboard - {}",
                                e
                            ));
                        }
                    }
                    if ui
                        .button(icon::ADD)
                        .on_hover_text(ADD_DESCRIPTION)
                        .on_hover_cursor(CursorIcon::Copy)
                        .clicked()
                    {
                        ctx.app.add_cur_color();
                    }
                });
                let cb = ColorBox::builder()
                    .size((CURRENT_COLOR_BOX_SIZE, CURRENT_COLOR_BOX_SIZE))
                    .color(ctx.app.picker.current_color)
                    .label(true)
                    // .hover_help(COLORBOX_PICK_TOOLTIP)
                    .border(true)
                    .build();
                ui.horizontal(|ui| {
                    cb.display(ctx, ui);
                });

                self.zoom_picker.display(ctx, ui);
            });
        });

        ui.add_space(SPACE);

        ScrollArea::vertical()
            .id_source("picker scroll")
            .show(ui, |ui| {
                // self.harmonies_header(ctx, ui);
                // self.sliders(ctx, ui);
                // self.hex_input(ctx, ui);
                let mut available_space = ui.available_size_before_wrap();
                if ctx.app.sidepanel.show {
                    available_space.x -= ctx.app.sidepanel.response_size.x;
                }
                ui.allocate_space(available_space);
            });
    }
}
