use macroquad::prelude::*;

mod app;
mod color;
mod curves;
mod palette;
mod rendering;
mod ui;
mod viewport;

use app::App;
use rendering::{draw_checker_background, draw_palette};
use ui::swatch_editor::SwatchEditorState;
use ui::palette_editor::PaletteEditorState;
use ui::{draw_palette_editor, draw_swatch_editor, draw_top_panel};

// =============================================================================
// Main application
// =============================================================================

#[macroquad::main("Palette Helper")]
async fn main() {
    let mut app = App::new();
    let mut swatch_editor_state = SwatchEditorState::default();
    let mut palette_editor_state = PaletteEditorState::default();

    // Sync editor state with initial swatch
    swatch_editor_state.sync_with_swatch(&app);

    loop {
        // Draw background with parallax
        draw_checker_background(&app.viewport);

        // Draw all palette swatches (auto-aligned)
        draw_palette(
            &app.viewport,
            &app.generated_colors,
            app.current_swatch_index,
        );

        // Track if egui wants input
        let mut egui_wants_pointer = false;

        // Draw egui UI
        egui_macroquad::ui(|egui_ctx| {
            egui_wants_pointer = egui_ctx.wants_pointer_input();

            // Draw all UI windows
            draw_top_panel(egui_ctx, &mut app);
            draw_swatch_editor(egui_ctx, &mut app, &mut swatch_editor_state);
            draw_palette_editor(egui_ctx, &mut app, &mut palette_editor_state);
        });

        // Handle viewport input (only if egui doesn't want it)
        app.viewport.handle_input(egui_wants_pointer);

        egui_macroquad::draw();
        next_frame().await;
    }
}
