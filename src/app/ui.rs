use eframe::egui::{self, Color32, Id, Label, RichText};

use crate::context::FrameCtx;

use super::{App, ERROR_DISPLAY_DURATION};

pub trait AppUi {
    fn error_ui(app: &mut App, ctx: &mut FrameCtx<'_>, ui: &mut eframe::egui::Ui) {
        let mut top_padding = 0.;
        let mut err_idx = 0;
        app.display_errors.retain(|e| {
            let elapsed = crate::elapsed(e.timestamp());
            if elapsed >= ERROR_DISPLAY_DURATION {
                false
            } else {
                if let Some(rsp) = egui::Window::new("Error")
                    .collapsible(true)
                    .id(Id::new(format!("err_ntf_{err_idx}")))
                    .anchor(
                        egui::Align2::RIGHT_BOTTOM,
                        (-ctx.app.sidepanel.box_width - 25., -top_padding),
                    )
                    .hscroll(true)
                    .fixed_size((ctx.app.sidepanel.box_width + 7000., 50.))
                    .show(ui.ctx(), |ui| {
                        let label =
                            Label::new(RichText::new(e.message()).color(Color32::RED)).wrap(true);
                        ui.add(label);
                    })
                {
                    top_padding += rsp.response.rect.height() + 8.;
                    err_idx += 1;
                };
                true
            }
        });
    }

    fn ui(app: &mut App, ctx: &mut FrameCtx<'_>, ui: &mut egui::Ui) {
        Self::error_ui(app, ctx, ui);
    }
}
