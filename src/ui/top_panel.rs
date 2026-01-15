use egui_macroquad::egui::{self, TopBottomPanel};

use crate::app::App;
use crate::viewport::Viewport;

/// Draw the top menu panel
pub fn draw_top_panel(egui_ctx: &egui::Context, app: &mut App) {
    TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Chrma Palette Studio");

            ui.menu_button("File", |ui| {
                if ui.button("New Palette").clicked() {
                    // TODO: Implement new palette
                    ui.close_menu();
                }
                if ui.button("Load Palette...").clicked() {
                    // TODO: Load logic here
                    ui.close_menu();
                }
                if ui.button("Save Palette...").clicked() {
                    // TODO: Save logic here
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Export...").clicked() {
                    // TODO: Export logic here
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // TODO: Undo
                    ui.close_menu();
                }
                if ui.button("Redo").clicked() {
                    // TODO: Redo
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Reset Viewport").clicked() {
                    app.viewport = Viewport::default();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Zoom In").clicked() {
                    app.viewport.zoom *= 1.25;
                    ui.close_menu();
                }
                if ui.button("Zoom Out").clicked() {
                    app.viewport.zoom *= 0.8;
                    ui.close_menu();
                }
                if ui.button("Zoom to Fit").clicked() {
                    app.viewport.zoom = 1.0;
                    app.viewport.offset = macroquad::prelude::Vec2::ZERO;
                    ui.close_menu();
                }
            });

            ui.menu_button("Palette", |ui| {
                if ui.button("Add Swatch").clicked() {
                    app.add_swatch(crate::palette::Swatch::default());
                    ui.close_menu();
                }
                if ui.button("Duplicate Current").clicked() {
                    let idx = app.current_swatch_index;
                    app.duplicate_swatch(idx);
                    ui.close_menu();
                }
            });

            // Display info
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Swatch {}/{} | {} colors | Zoom: {:.0}%",
                    app.current_swatch_index + 1,
                    app.swatch_count(),
                    app.current_swatch().size,
                    app.viewport.zoom * 100.0
                ));
            });
        });
    });
}
