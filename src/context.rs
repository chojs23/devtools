use eframe::{
    egui::{self, CursorIcon},
    CreationContext, Storage,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{CentralPanelTab, DARK_VISUALS, LIGHT_VISUALS},
    color::{Color, ColorFormat, Palettes},
    color_picker::ColorPicker,
    error::append_global_error,
    jwt::JwtEncoderDecoder,
    render::{TextureAllocator, TextureManager},
    screen_size::ScreenSize,
    settings::{self, ColorDisplayFmtEnum, Settings, BASE_DIR},
};

#[derive(Clone, Debug)]
pub struct AppCtx {
    pub settings: Settings,

    pub screen_size: ScreenSize,
    pub cursor_icon: CursorIcon,

    pub sidepanel: SidePanelData,

    pub jwt: JwtEncoderDecoder,
    pub picker: ColorPicker,
    pub palettes: Palettes,

    pub cursor_pick_color: Color,
    pub current_selected_color: Color,

    pub central_panel_tab: CentralPanelTab,

    pub zoom_window_dragged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidePanelData {
    /// Is the side panel visible
    pub show: bool,
    /// If true palette name is currently being edited
    pub edit_palette_name: bool,
    /// When triggering palette name edit this is used to
    /// switch focus to the textedit
    pub trigger_edit_focus: bool,
    /// Width of the button toolbar on the sidebar
    pub box_width: f32,
    /// Size of the whole Sidebar response
    pub response_size: egui::Vec2,
}

impl Default for AppCtx {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            cursor_icon: CursorIcon::default(),
            screen_size: ScreenSize::Desktop(0., 0.),
            sidepanel: SidePanelData {
                show: false,
                edit_palette_name: false,
                trigger_edit_focus: false,
                box_width: 0.,
                response_size: (0., 0.).into(),
            },
            jwt: JwtEncoderDecoder::default(),
            picker: ColorPicker::default(),
            palettes: Palettes::default(),
            cursor_pick_color: Color::black(),
            current_selected_color: Color::black(),
            central_panel_tab: CentralPanelTab::Jwt,
            zoom_window_dragged: false,
        }
    }
}

impl AppCtx {
    pub const KEY: &'static str = "app-global-ctx";

    /// Initialize a new context
    pub fn new(context: &CreationContext) -> Self {
        Self {
            settings: settings::load_global(context.storage).unwrap_or_default(),
            ..Default::default()
        }
    }

    /// Current color display format
    pub fn display_format(&self) -> ColorFormat {
        match self.settings.color_display_format {
            ColorDisplayFmtEnum::Hex => ColorFormat::Hex,
            ColorDisplayFmtEnum::HexUppercase => ColorFormat::HexUpercase,
            ColorDisplayFmtEnum::CssRgb => ColorFormat::CssRgb,
            ColorDisplayFmtEnum::CssHsl => ColorFormat::CssHsl {
                degree_symbol: true,
            },
            ColorDisplayFmtEnum::Custom(ref name) => {
                if self.settings.saved_color_formats.get(name).is_some() {
                    ColorFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    ColorDisplayFmtEnum::default_display_format()
                }
            }
        }
    }

    /// Format a color as a string using display color format from settings
    pub fn display_color(&self, color: &Color) -> String {
        color.display(
            self.display_format(),
            self.settings.rgb_working_space,
            self.settings.illuminant,
        )
    }

    /// Format a color as a string using clipboard color format from settings
    pub fn clipboard_color(&self, color: &Color) -> String {
        let format = match self
            .settings
            .color_clipboard_format
            .as_ref()
            .unwrap_or(&self.settings.color_display_format)
        {
            ColorDisplayFmtEnum::Hex => ColorFormat::Hex,
            ColorDisplayFmtEnum::HexUppercase => ColorFormat::HexUpercase,
            ColorDisplayFmtEnum::CssRgb => ColorFormat::CssRgb,
            ColorDisplayFmtEnum::CssHsl => ColorFormat::CssHsl {
                degree_symbol: false,
            },
            ColorDisplayFmtEnum::Custom(name) => {
                if self.settings.saved_color_formats.get(name).is_some() {
                    ColorFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    ColorDisplayFmtEnum::default_display_format()
                }
            }
        };
        color.display(
            format,
            self.settings.rgb_working_space,
            self.settings.illuminant,
        )
    }

    /// Load palettes from appropriate location based on the target arch
    pub fn load_palettes(&mut self, _storage: Option<&dyn Storage>) {
        if self.settings.cache_colors {
            #[cfg(target_arch = "wasm32")]
            if let Some(storage) = _storage {
                match Palettes::load_from_storage(storage) {
                    Ok(palettes) => self.palettes = palettes,
                    Err(e) => append_global_error(format!("failed to load palettes, {e:?}")),
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = Palettes::dir(BASE_DIR) {
                match Palettes::load(path.join(Palettes::FILE_NAME)) {
                    Ok(palettes) => self.palettes = palettes,
                    Err(e) => append_global_error(format!("failed to load palettes, {e:?}")),
                }
            }
        }
    }

    /// Save palettes to appropriate location based on the target arch
    pub fn save_palettes(&self, _storage: &mut dyn Storage) {
        #[cfg(target_arch = "wasm32")]
        if self.settings.cache_colors {
            if let Err(e) = self.palettes.save_to_storage(_storage) {
                append_global_error(format!("failed to save palettes, {e:?}"));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(dir) = Palettes::dir("dev_tools") {
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            if let Err(e) = self.palettes.save(dir.join(Palettes::FILE_NAME)) {
                append_global_error(format!("failed to save palettes, {e:?}"));
            }
        }
    }

    /// Adds a color to the currently selected palette
    pub fn add_color(&mut self, color: Color) {
        if !self.palettes.current_mut().palette.add(color) {
            let color_str = self.display_color(&color);
            append_global_error(format!("Color {} already saved!", color_str));
        } else {
            self.sidepanel.show = true;
        }
    }

    pub fn add_cur_color(&mut self) {
        self.add_color(self.picker.current_color)
    }

    /// Replaces cursor icon with `icon`
    pub fn toggle_mouse(&mut self, icon: CursorIcon) {
        self.cursor_icon = if icon == self.cursor_icon {
            CursorIcon::default()
        } else {
            icon
        }
    }

    pub fn check_settings_change(&mut self) {
        // if self.settings.chromatic_adaptation_method
        //     != self.picker.sliders.chromatic_adaptation_method
        // {
        //     self.picker.sliders.chromatic_adaptation_method =
        //         self.settings.chromatic_adaptation_method;
        // }
        // if self.settings.rgb_working_space != self.picker.sliders.rgb_working_space {
        //     self.picker.new_workspace = Some(self.settings.rgb_working_space);
        //     if self.settings.illuminant != self.picker.sliders.illuminant {
        //         self.picker.new_illuminant = Some(self.settings.illuminant);
        //     }
        // }
    }
}

pub struct FrameCtx<'frame> {
    pub app: &'frame mut AppCtx,
    pub egui: &'frame egui::Context,
    pub tex_manager: &'frame mut TextureManager,
    pub frame: Option<&'frame mut eframe::Frame>,
}

impl<'frame> FrameCtx<'frame> {
    pub fn tex_allocator(&self) -> TextureAllocator {
        Some(self.egui.tex_manager())
    }

    pub fn is_dark_mode(&self) -> bool {
        self.app.settings.is_dark_mode
    }

    pub fn set_dark_theme(&mut self) {
        self.app.settings.is_dark_mode = true;
        self.egui.set_visuals(DARK_VISUALS.clone());
    }

    pub fn set_light_theme(&mut self) {
        self.app.settings.is_dark_mode = false;
        self.egui.set_visuals(LIGHT_VISUALS.clone());
    }

    pub fn set_theme(&mut self) {
        if self.is_dark_mode() {
            self.set_light_theme();
        } else {
            self.set_dark_theme();
        }
    }

    pub fn set_styles(&mut self, screen_size: ScreenSize) {
        self.app.screen_size = screen_size;

        let slider_size = match screen_size {
            ScreenSize::Phone(w, _) => w * 0.5,
            ScreenSize::Desktop(w, _) if w > 1500. => w * 0.2,
            ScreenSize::Tablet(w, _) | ScreenSize::Laptop(w, _) | ScreenSize::Desktop(w, _) => {
                w * 0.35
            }
        };

        let mut style = (*self.egui.style()).clone();
        style.spacing.slider_width = slider_size / 2.;
        self.egui.set_style(style);
    }

    // #[cfg(not(target_arch = "wasm32"))]
    // pub fn set_window_size(&mut self, size: egui::Vec2) {
    //     if let Some(frame) = self.frame.as_mut() {}
    // }
}
