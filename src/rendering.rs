use egui_macroquad::egui::Color32;
use macroquad::prelude::*;

use crate::viewport::Viewport;

// =============================================================================
// Constants
// =============================================================================

pub const COLOR_SQUARE_SIZE: f32 = 48.0;
pub const COLOR_SQUARE_SPACING: f32 = 4.0;
const CHECKER_SIZE: f32 = 32.0;
const PARALLAX_FACTOR: f32 = 0.3; // Background scrolls at 30% of camera speed

// Background colors for checker pattern
const CHECKER_COLOR_A: Color = Color::new(0.0, 0.0, 0.0, 1.0); // Pure black
const CHECKER_COLOR_B: Color = Color::new(0.02, 0.02, 0.08, 1.0); // Very dark blue

// =============================================================================
// Background rendering with parallax
// =============================================================================

pub fn draw_checker_background(viewport: &Viewport) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    // Calculate parallax offset (background moves slower than foreground)
    let parallax_offset = viewport.offset * PARALLAX_FACTOR;

    // Effective checker size on screen (doesn't zoom, just shifts)
    let checker_size = CHECKER_SIZE;

    // Calculate how many checkers we need to cover the screen
    let start_x = -(parallax_offset.x % (checker_size * 2.0)) - checker_size * 2.0;
    let start_y = -(parallax_offset.y % (checker_size * 2.0)) - checker_size * 2.0;

    let mut y = start_y;
    let mut row = 0;
    while y < screen_h + checker_size * 2.0 {
        let mut x = start_x;
        let mut col = 0;
        while x < screen_w + checker_size * 2.0 {
            let is_light = (row + col) % 2 == 0;
            let color = if is_light {
                CHECKER_COLOR_A
            } else {
                CHECKER_COLOR_B
            };
            draw_rectangle(x, y, checker_size, checker_size, color);
            x += checker_size;
            col += 1;
        }
        y += checker_size;
        row += 1;
    }
}

// =============================================================================
// Palette rendering in world space
// =============================================================================

/// Layout constants for auto-aligned swatches
const SWATCH_VERTICAL_SPACING: f32 = 8.0;
const SWATCH_START_X: f32 = 0.0;
const SWATCH_START_Y: f32 = 0.0;
const INDEX_LABEL_OFFSET: f32 = 30.0; // Space for index label to the left

/// Draw all swatches in the palette, auto-aligned vertically
/// All swatches start at the same X position and stack vertically
pub fn draw_palette(
    viewport: &Viewport,
    swatches: &[Vec<Color32>],
    current_swatch_index: usize,
) {
    let mut y_offset = SWATCH_START_Y;
    
    for (swatch_idx, colors) in swatches.iter().enumerate() {
        let is_selected = swatch_idx == current_swatch_index;
        let position = Vec2::new(SWATCH_START_X, y_offset);
        draw_swatch(viewport, colors, position, swatch_idx, is_selected);
        
        // Move down for next swatch
        y_offset += COLOR_SQUARE_SIZE + SWATCH_VERTICAL_SPACING;
    }
}

/// Draw a single swatch at a given world position
/// The position marks the top-left of the first color square
fn draw_swatch(
    viewport: &Viewport,
    colors: &[Color32],
    position: Vec2,
    swatch_index: usize,
    is_selected: bool,
) {
    let num_colors = colors.len();
    if num_colors == 0 {
        return;
    }

    // All swatches are left-aligned, starting at the same X position
    let start_x = position.x;
    let start_y = position.y;
    let total_width =
        num_colors as f32 * (COLOR_SQUARE_SIZE + COLOR_SQUARE_SPACING) - COLOR_SQUARE_SPACING;

    // Draw selection highlight if this is the current swatch
    if is_selected {
        let screen_start = viewport.world_to_screen(Vec2::new(start_x - 4.0, start_y - 4.0));
        let screen_end = viewport.world_to_screen(Vec2::new(
            start_x + total_width + 4.0,
            start_y + COLOR_SQUARE_SIZE + 4.0,
        ));
        let size = screen_end - screen_start;
        draw_rectangle_lines(
            screen_start.x,
            screen_start.y,
            size.x,
            size.y,
            3.0,
            Color::new(0.4, 0.6, 1.0, 0.8),
        );
    }

    // Draw swatch index to the left of the first color
    let index_pos = viewport.world_to_screen(Vec2::new(start_x - INDEX_LABEL_OFFSET, start_y + COLOR_SQUARE_SIZE / 2.0));
    let font_size = (16.0 * viewport.zoom).max(12.0) as u16;
    let index_color = if is_selected {
        Color::new(0.4, 0.7, 1.0, 1.0) // Highlight color for selected swatch
    } else {
        Color::new(0.6, 0.6, 0.6, 1.0)
    };
    draw_text(
        &format!("{}", swatch_index + 1),
        index_pos.x,
        index_pos.y + font_size as f32 / 3.0,
        font_size as f32,
        index_color,
    );

    for (i, color) in colors.iter().enumerate() {
        // World position of this square
        let world_x = start_x + i as f32 * (COLOR_SQUARE_SIZE + COLOR_SQUARE_SPACING);
        let world_y = start_y;

        // Convert to screen coordinates
        let screen_pos = viewport.world_to_screen(Vec2::new(world_x, world_y));
        let screen_size = COLOR_SQUARE_SIZE * viewport.zoom;

        // Draw the color square
        draw_rectangle(
            screen_pos.x,
            screen_pos.y,
            screen_size,
            screen_size,
            Color::from_rgba(color.r(), color.g(), color.b(), color.a()),
        );

        // Draw border for better visibility
        draw_rectangle_lines(
            screen_pos.x,
            screen_pos.y,
            screen_size,
            screen_size,
            2.0,
            Color::new(0.2, 0.2, 0.2, 0.8),
        );

        // Draw index number below
        let idx_font_size = (14.0 * viewport.zoom).max(10.0) as u16;
        let label = format!("{}", i + 1);
        let text_pos = viewport.world_to_screen(Vec2::new(
            world_x + COLOR_SQUARE_SIZE / 2.0,
            world_y + COLOR_SQUARE_SIZE + 8.0,
        ));
        draw_text(
            &label,
            text_pos.x - 4.0,
            text_pos.y + idx_font_size as f32,
            idx_font_size as f32,
            Color::new(0.7, 0.7, 0.7, 1.0),
        );
    }
}
