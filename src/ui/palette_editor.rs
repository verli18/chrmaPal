use egui_macroquad::egui::{self, Color32};

use crate::app::App;
use crate::palette::Swatch;
use crate::ui::widgets::{draggable_list_item, draw_color_bar, sample_colors, DragDropResult, DragDropState};

/// Number of sample colors to show in the swatch preview
const PREVIEW_SAMPLES: usize = 4;
/// Size of the preview rectangle
const PREVIEW_WIDTH: f32 = 100.0;
const PREVIEW_HEIGHT: f32 = 20.0;

/// UI state for the palette editor
#[derive(Default)]
pub struct PaletteEditorState {
    /// State for drag-drop reordering
    pub drag_state: DragDropState,
}

/// Actions to perform on swatches
enum SwatchAction {
    Select(usize),
    Swap(usize, usize),
    Delete(usize),
}

/// Draw the palette editor window
pub fn draw_palette_editor(egui_ctx: &egui::Context, app: &mut App, state: &mut PaletteEditorState) {
    egui::Window::new("Palette Editor")
        .default_width(280.0)
        .show(egui_ctx, |ui| {
            ui.heading("Swatches");
            ui.separator();

            let num_swatches = app.palette.swatches.len();
            let mut action: Option<SwatchAction> = None;

            for i in 0..num_swatches {
                let is_selected = i == app.current_swatch_index;
                let swatch = &app.palette.swatches[i];

                // Get sample colors for this swatch
                let colors = if i < app.generated_colors.len() {
                    sample_colors(&app.generated_colors[i], PREVIEW_SAMPLES)
                } else {
                    vec![Color32::BLACK; PREVIEW_SAMPLES]
                };

                let (_handle, drag_result, _) = draggable_list_item(
                    ui,
                    &mut state.drag_state,
                    i,
                    is_selected,
                    |ui| {
                        // Swatch preview - draw as a single rectangle with color samples
                        draw_color_bar(ui, &colors, PREVIEW_WIDTH, PREVIEW_HEIGHT);

                        // Swatch info
                        ui.label(format!("{} colors", swatch.size));

                        // Select button (shown for all, but styled differently if selected)
                        ui.add_enabled_ui(!is_selected, |ui| {
                            if ui.button("Select").clicked() {
                                action = Some(SwatchAction::Select(i));
                            }
                        });

                        // Delete button (only if more than one swatch)
                        if num_swatches > 1 {
                            if ui.button("×").clicked() {
                                action = Some(SwatchAction::Delete(i));
                            }
                        }
                    },
                );

                // Handle drag-drop result
                if let DragDropResult::Dropped { source_index, target_index } = drag_result {
                    action = Some(SwatchAction::Swap(source_index, target_index));
                }
            }

            // Apply actions
            if let Some(act) = action {
                match act {
                    SwatchAction::Select(idx) => app.select_swatch(idx),
                    SwatchAction::Swap(from, to) => app.swap_swatches(from, to),
                    SwatchAction::Delete(idx) => app.remove_swatch(idx),
                }
            }

            ui.separator();

            // Add new swatch button
            if ui.button("+ Add Swatch").clicked() {
                app.add_swatch(Swatch::default());
            }
        });
}
