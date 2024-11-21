use eframe::egui::{self, Color32, CursorIcon, ScrollArea};

use crate::{
    app::ui::AppUi,
    context::FrameCtx,
    error::append_global_error,
    ui::{DOUBLE_SPACE, HALF_SPACE, SPACE},
};

use super::Algorithm;

pub struct JwtUi;

impl AppUi for JwtUi {}

impl JwtUi {
    fn jwt_ui(ctx: &mut FrameCtx<'_>, ui: &mut egui::Ui) {
        ui.heading("JWT Encoder/Decoder");

        ui.add_space(DOUBLE_SPACE);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.vertical(|ui| {
                    ui.label("Encoded");
                    if ui.text_edit_multiline(&mut ctx.app.jwt.encoded).changed() {
                        let _ = ctx.app.jwt.verify();
                    }
                });

                ui.add_space(HALF_SPACE);

                ui.horizontal(|ui| {
                    ui.label("Algorithm");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::HS256, "HS256");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::HS384, "HS384");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::HS512, "HS512");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::RS256, "RS256");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::RS384, "RS384");
                    ui.radio_value(&mut ctx.app.jwt.algorithm, Algorithm::RS512, "RS512");
                });

                ui.add_space(SPACE);

                ui.horizontal(|ui| {
                    if ui
                        .button("⬆ Encode")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        match ctx.app.jwt.encode() {
                            Ok(_) => {}
                            Err(e) => {
                                append_global_error(e);
                            }
                        }
                    }

                    if ui
                        .button("⬇ Decode")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        match ctx.app.jwt.decode() {
                            Ok(_) => {}
                            Err(e) => {
                                append_global_error(e);
                            }
                        }
                    }

                    if ui
                        .button("⟲  Clear")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                    {
                        ctx.app.jwt.clear();
                    }

                    ui.label(
                        egui::RichText::new(format!(
                            "Verified {}",
                            if ctx.app.jwt.verified.is_some() {
                                if ctx.app.jwt.verified.unwrap() {
                                    "✔"
                                } else {
                                    "✖"
                                }
                            } else {
                                "?"
                            }
                        ))
                        .color(ctx.app.jwt.verified.map_or(
                            Color32::WHITE,
                            |v| {
                                if v {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                }
                            },
                        )),
                    )
                });

                ui.add_space(HALF_SPACE);

                ui.vertical(|ui| {
                    ui.label("Decoded");
                    ui.text_edit_multiline(&mut ctx.app.jwt.decoded);
                });

                ui.add_space(SPACE);

                ui.vertical(|ui| {
                    ui.label("Header");
                    ui.add_space(HALF_SPACE);
                    let header = ctx.app.jwt.get_header().unwrap_or_default();
                    ui.text_edit_multiline(&mut header.as_str());
                });
            });

            ui.vertical(|ui| match ctx.app.jwt.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                    ui.label("Secret");
                    if ui.text_edit_singleline(&mut ctx.app.jwt.secret).changed() {
                        let _ = ctx.app.jwt.verify();
                    }
                }
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                    ui.vertical(|ui| {
                        ui.label("Public Key");
                        let scroll_height = ui.available_height() - 30.0;
                        ScrollArea::vertical()
                            .id_source("public_key")
                            .max_height(scroll_height)
                            .stick_to_bottom(false)
                            .show(ui, |ui| {
                                if ui
                                    .text_edit_multiline(&mut ctx.app.jwt.public_key)
                                    .changed()
                                {
                                    let _ = ctx.app.jwt.verify();
                                }
                            });
                    });

                    ui.add_space(SPACE * 4.);

                    ui.vertical(|ui| {
                        ui.label("Private Key");
                        let scroll_height = ui.available_height() - 30.0;
                        ScrollArea::vertical()
                            .id_source("private_key")
                            .max_height(scroll_height)
                            .stick_to_bottom(false)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut ctx.app.jwt.private_key);
                            });
                    });
                }
            });
        });
    }
}
