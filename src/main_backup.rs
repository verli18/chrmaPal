use egui_macroquad::egui::{self, Color32, Slider, TopBottomPanel, Window};
use macroquad::prelude::*;

mod color_fun;
mod curves;
mod palette;

use color_fun::ColorSpace;
use curves::{CurveKind, CurveType, EaseIn, EaseInOut, EaseOut, Linear};
use palette::{Swatch, Palette};

// =============================================================================
// Constants
// =============================================================================

const COLOR_SQUARE_SIZE: f32 = 48.0;
const COLOR_SQUARE_SPACING: f32 = 4.0;
const CHECKER_SIZE: f32 = 32.0;
const PARALLAX_FACTOR: f32 = 0.3; // Background scrolls at 30% of camera speed

// Background colors for checker pattern
const CHECKER_COLOR_A: Color = Color::new(0.0, 0.0, 0.0, 1.0);           // Pure black
const CHECKER_COLOR_B: Color = Color::new(0.02, 0.02, 0.08, 1.0);        // Very dark blue

// =============================================================================
// Viewport: Handles camera panning and zooming
// =============================================================================

struct Viewport {
    /// Camera offset in world coordinates (what world position is at screen center)
    offset: Vec2,
    /// Zoom level (1.0 = normal, 2.0 = zoomed in 2x)
    zoom: f32,
    /// Minimum zoom level
    min_zoom: f32,
    /// Maximum zoom level
    max_zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.25,
            max_zoom: 4.0,
        }
    }
}

impl Viewport {
    /// Handle mouse input for panning and zooming
    /// Returns true if input was consumed
    fn handle_input(&mut self, egui_wants_input: bool) -> bool {
        if egui_wants_input {
            return false;
        }

        let mut consumed = false;

        // Pan with middle mouse button or left mouse + drag
        if is_mouse_button_down(MouseButton::Left)  {
            let delta = mouse_delta_position();
            // Invert and scale by zoom so dragging feels natural
            self.offset.x += delta.x / self.zoom * 200.0;
            self.offset.y += delta.y / self.zoom * 200.0;
            consumed = true;
        }

        // Zoom with scroll wheel
        let (_, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            let mouse_pos = Vec2::new(mouse_position().0, mouse_position().1);
            let world_before = self.screen_to_world(mouse_pos);
            
            // Apply zoom
            let zoom_factor = 1.1_f32.powf(scroll_y);
            self.zoom = (self.zoom * zoom_factor).clamp(self.min_zoom, self.max_zoom);
            
            // Adjust offset so the point under the mouse stays fixed
            let world_after = self.screen_to_world(mouse_pos);
            self.offset += world_before - world_after;
            
            consumed = true;
        }

        consumed
    }

    /// Convert screen coordinates to world coordinates
    fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let offset_from_center = screen_pos - screen_center;
        self.offset + offset_from_center / self.zoom
    }

    /// Convert world coordinates to screen coordinates
    fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        screen_center + (world_pos - self.offset) * self.zoom
    }

    /// Get the visible world bounds (for culling)
    fn visible_bounds(&self) -> (Vec2, Vec2) {
        let top_left = self.screen_to_world(Vec2::ZERO);
        let bottom_right = self.screen_to_world(Vec2::new(screen_width(), screen_height()));
        (top_left, bottom_right)
    }
}

// =============================================================================
// Background rendering with parallax
// =============================================================================

fn draw_checker_background(viewport: &Viewport) {
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
            let color = if is_light { CHECKER_COLOR_A } else { CHECKER_COLOR_B };
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

fn draw_palette_swatches(
    viewport: &Viewport,
    colors: &[Color32],
    hex_edit_state: &HexEditState,
) {
    let num_colors = colors.len();
    if num_colors == 0 {
        return;
    }

    // Calculate total width of the palette in world space
    let total_width = num_colors as f32 * (COLOR_SQUARE_SIZE + COLOR_SQUARE_SPACING) - COLOR_SQUARE_SPACING;
    let start_x = -total_width / 2.0; // Center the palette at world origin

    for (i, color) in colors.iter().enumerate() {
        // World position of this square
        let world_x = start_x + i as f32 * (COLOR_SQUARE_SIZE + COLOR_SQUARE_SPACING);
        let world_y = -COLOR_SQUARE_SIZE / 2.0; // Center vertically at y=0

        // Convert to screen coordinates
        let screen_pos = viewport.world_to_screen(Vec2::new(world_x, world_y));
        let screen_size = COLOR_SQUARE_SIZE * viewport.zoom;

        // Use edited color if available
        let display_color = hex_edit_state.get(i).unwrap_or(*color);
        
        // Draw the color square
        draw_rectangle(
            screen_pos.x,
            screen_pos.y,
            screen_size,
            screen_size,
            Color::from_rgba(display_color.r(), display_color.g(), display_color.b(), display_color.a()),
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

        // Draw edit indicator if this slot was edited
        if hex_edit_state.was_edited(i) {
            let indicator_size = 8.0 * viewport.zoom;
            draw_rectangle(
                screen_pos.x + screen_size / 2.0 - indicator_size / 2.0,
                screen_pos.y - indicator_size - 4.0 * viewport.zoom,
                indicator_size,
                indicator_size,
                Color::from_rgba(255, 200, 50, 255),
            );
        }

        // Draw index number below
        let font_size = (14.0 * viewport.zoom).max(10.0) as u16;
        let label = format!("{}", i + 1);
        let text_pos = viewport.world_to_screen(Vec2::new(
            world_x + COLOR_SQUARE_SIZE / 2.0,
            world_y + COLOR_SQUARE_SIZE + 8.0,
        ));
        draw_text(
            &label,
            text_pos.x - 4.0,
            text_pos.y + font_size as f32,
            font_size as f32,
            Color::new(0.7, 0.7, 0.7, 1.0),
        );
    }
}

// =============================================================================
// HexEditState (unchanged from before)
// =============================================================================

/// State for tracking color edits in the hex view
struct HexEditState {
    /// The colors as they were when we last generated (for change detection)
    original_colors: Vec<Color32>,
    /// Current edited colors (may differ from generated if user edited)
    edited_colors: Vec<Color32>,
}

impl HexEditState {
    fn new() -> Self {
        Self {
            original_colors: Vec::new(),
            edited_colors: Vec::new(),
        }
    }

    fn sync_with_generated(&mut self, generated: &[Color32]) {
        if self.original_colors.len() != generated.len() {
            self.original_colors = generated.to_vec();
            self.edited_colors = generated.to_vec();
        } else {
            let generated_changed = self.original_colors.iter()
                .zip(generated.iter())
                .any(|(a, b)| a != b);
            
            if generated_changed {
                self.original_colors = generated.to_vec();
                self.edited_colors = generated.to_vec();
            }
        }
    }

    fn was_edited(&self, index: usize) -> bool {
        if index >= self.original_colors.len() || index >= self.edited_colors.len() {
            return false;
        }
        self.original_colors[index] != self.edited_colors[index]
    }

    fn get(&self, index: usize) -> Option<Color32> {
        self.edited_colors.get(index).copied()
    }

    fn set(&mut self, index: usize, color: Color32) {
        if index < self.edited_colors.len() {
            self.edited_colors[index] = color;
        }
    }

    fn clear_edit(&mut self, index: usize) {
        if index < self.edited_colors.len() && index < self.original_colors.len() {
            self.edited_colors[index] = self.original_colors[index];
        }
    }
}

struct App {
    palette: palette::Palette,
    current_swatch_index: usize,
    viewport: Viewport,
    hex_edit_state: HexEditState,
    generated_palette: Vec<Vec<Color32>>,
}

impl App {
    fn new() -> Self {
        Self {
            palette: palette::Palette::new(),
            current_swatch_index: 0,
            viewport: Viewport::default(),
            hex_edit_state: HexEditState::new(),
            generated_palette: Vec::new(),
        }
    }
}
// =============================================================================
// Main application
// =============================================================================

#[macroquad::main("Palette Helper")]
async fn main() {
    let mut app = App::new();
    let mut current_swatch = &mut app.palette.swatches[app.current_swatch_index];

    // UI state
    let mut selected_curve_kind = current_swatch.interpolation_curve.kind();
    let mut curve_exponent: f32 = 2.0;
    let mut linear_factor: f32 = 1.0;
    let mut hex_edit_state = HexEditState::new();

    loop {
        // Generate colors
        let generated_colors = current_swatch.generate_colors();
        hex_edit_state.sync_with_generated(&generated_colors);

        // Draw background with parallax
        draw_checker_background(&app.viewport);

        // Draw palette swatches
        draw_palette_swatches(&app.viewport, &generated_colors, &hex_edit_state);

        // Track if egui wants input (set inside the ui closure)
        let mut egui_wants_pointer = false;
        let mut egui_wants_keyboard = false;

        // Draw egui UI
        egui_macroquad::ui(|egui_ctx| {
            egui_wants_pointer = egui_ctx.wants_pointer_input();
            egui_wants_keyboard = egui_ctx.wants_keyboard_input();

            egui::Window::new("Swatch Editor")
                .show(egui_ctx, |ui| {
                    // Swatch size control
                    let mut size = current_swatch.size;
                    if ui.add(Slider::new(&mut size, 2..=32).text("Swatch size")).changed() {
                        current_swatch.size = size;
                    }

                    ui.separator();
                    
                    // Color space selector
                    ui.horizontal(|ui| {
                        ui.label("Color Space:");
                        egui::ComboBox::from_id_salt("color_space")
                            .selected_text(current_swatch.color_space.name())
                            .show_ui(ui, |ui| {
                                for &space in ColorSpace::ALL {
                                    ui.selectable_value(&mut current_swatch.color_space, space, space.name());
                                }
                            });
                    });

                    ui.separator();
                    ui.label("Control Points:");

                    if current_swatch.control_points().is_empty() {
                        ui.label("(No control points - all black)");
                    }

                    // Edit control point colors
                    let num_points = current_swatch.control_points().len();
                    let mut point_to_remove: Option<usize> = None;
                    
                    for i in 0..num_points {
                        ui.horizontal(|ui| {
                            let mut pos = current_swatch.control_points()[i].position;
                            if ui.add(Slider::new(&mut pos, 0.0..=1.0).fixed_decimals(2).show_value(false)).changed() {
                                current_swatch.set_control_point_position(i, pos);
                            }
                            ui.label(format!("{:.0}%", pos * 100.0));

                            let mut color = current_swatch.control_points()[i].color;
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                current_swatch.set_control_point_color(i, color);
                            }
                            
                            if ui.button("×").clicked() {
                                point_to_remove = Some(i);
                            }
                        });
                    }
                    
                    if let Some(idx) = point_to_remove {
                        current_swatch.remove_control_point(idx);
                    }
                    
                    if ui.button("+ Add Control Point").clicked() {
                        let mid_pos = if current_swatch.control_points().is_empty() {
                            0.5
                        } else {
                            let positions: Vec<f32> = current_swatch.control_points().iter().map(|cp| cp.position).collect();
                            find_best_gap(&positions)
                        };
                        current_swatch.add_control_point(mid_pos, Color32::from_rgb(128, 128, 128));
                    }

                    ui.separator();
                    ui.label("Interpolation Curve:");

                    egui::ComboBox::from_label("Curve Type")
                        .selected_text(format!("{:?}", selected_curve_kind))
                        .show_ui(ui, |ui| {
                            for kind in [CurveKind::Linear, CurveKind::EaseIn, CurveKind::EaseOut, CurveKind::EaseInOut] {
                                if ui.selectable_value(&mut selected_curve_kind, kind, format!("{:?}", kind)).changed() {
                                    current_swatch.interpolation_curve = match kind {
                                        CurveKind::Linear => CurveType::Linear(Linear { factor: linear_factor }),
                                        CurveKind::EaseIn => CurveType::EaseIn(EaseIn { exponent: curve_exponent }),
                                        CurveKind::EaseOut => CurveType::EaseOut(EaseOut { exponent: curve_exponent }),
                                        CurveKind::EaseInOut => CurveType::EaseInOut(EaseInOut { exponent: curve_exponent }),
                                        CurveKind::Bezier => CurveType::from_kind(kind),
                                    };
                                }
                            }
                        });

                    match selected_curve_kind {
                        CurveKind::Linear => {
                            if ui.add(Slider::new(&mut linear_factor, 0.1..=2.0).text("Factor")).changed() {
                                current_swatch.interpolation_curve = CurveType::Linear(Linear { factor: linear_factor });
                            }
                        }
                        CurveKind::EaseIn | CurveKind::EaseOut | CurveKind::EaseInOut => {
                            if ui.add(Slider::new(&mut curve_exponent, 0.5..=5.0).text("Exponent")).changed() {
                                current_swatch.interpolation_curve = match selected_curve_kind {
                                    CurveKind::EaseIn => CurveType::EaseIn(EaseIn { exponent: curve_exponent }),
                                    CurveKind::EaseOut => CurveType::EaseOut(EaseOut { exponent: curve_exponent }),
                                    CurveKind::EaseInOut => CurveType::EaseInOut(EaseInOut { exponent: curve_exponent }),
                                    _ => unreachable!(),
                                };
                            }
                        }
                        CurveKind::Bezier => {
                            ui.label("Bezier controls not yet implemented");
                        }
                    }

                    ui.separator();

                    // Hex color editing section
                    ui.collapsing("Color Values (editable)", |ui| {
                        ui.label("Edit colors directly. Changed colors show ● and can be pinned.");
                        ui.add_space(4.0);
                        
                        let num_colors = hex_edit_state.edited_colors.len();
                        let mut action: Option<(usize, HexAction)> = None;
                        
                        for i in 0..num_colors {
                            let was_edited = hex_edit_state.was_edited(i);
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("{:2}:", i + 1));
                                
                                if was_edited {
                                    ui.label("●");
                                } else {
                                    ui.label(" ");
                                }
                                
                                let mut color = hex_edit_state.get(i).unwrap_or(Color32::BLACK);
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    action = Some((i, HexAction::SetColor(color)));
                                }
                                
                                ui.label(format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()));
                                
                                if was_edited {
                                    if ui.button("Pin").clicked() {
                                        action = Some((i, HexAction::Pin));
                                    }
                                    if ui.button("↩").clicked() {
                                        action = Some((i, HexAction::Revert));
                                    }
                                }
                            });
                        }
                        
                        if let Some((idx, hex_action)) = action {
                            match hex_action {
                                HexAction::SetColor(color) => {
                                    hex_edit_state.set(idx, color);
                                }
                                HexAction::Pin => {
                                    let position = if num_colors > 1 {
                                        idx as f32 / (num_colors - 1) as f32
                                    } else {
                                        0.5
                                    };
                                    
                                    let color = hex_edit_state.get(idx).unwrap_or(Color32::BLACK);
                                    let tolerance = 0.5 / num_colors as f32;
                                    if let Some(cp_idx) = current_swatch.has_control_point_at(position, tolerance) {
                                        current_swatch.set_control_point_color(cp_idx, color);
                                    } else {
                                        current_swatch.add_control_point(position, color);
                                    }
                                    hex_edit_state.clear_edit(idx);
                                }
                                HexAction::Revert => {
                                    hex_edit_state.clear_edit(idx);
                                }
                            }
                        }
                    });
                });

            egui::Window::new("Palette editor")
                .show(egui_ctx, |ui| {
                    for (i, swatch) in app.palette.swatches.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Swatch {}:", i + 1));
                            ui.label(format!("{} colors", swatch.size));
                            if ui.button("Select").clicked() {
                                app.current_swatch_index = i;
                                current_swatch = &mut app.palette.swatches[app.current_swatch_index];
                            }
                        });
                    }
                });

            TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Chrma pal studio");
                    ui.menu_button("File", |ui| {
                        if ui.button("New Palette").clicked() {
                           unimplemented!();
                        }
                        if ui.button("Load Palette...").clicked() {
                            // Load logic here
                        }
                        if ui.button("Save Palette...").clicked() {
                            // Save logic here
                        }

                        if ui.button("Export").clicked() {
                            // Export logic here
                        }

                        if ui.button("Exit").clicked() {
                            std::process::exit(0);
                        }
                    });

                    ui.menu_button("View", |ui| {
                        if ui.button("Reset Viewport").clicked() {
                            app.viewport = Viewport::default();
                        }
                    });
                    ui.menu_button("Palette..", |ui| {
                    });

                    ui.menu_button("test", |ui| {
                        if ui.button("test").clicked() {
                            // Export logic here
                        }
                    });
                });
                });

            });

        // Handle viewport input (only if egui doesn't want it)
        app.viewport.handle_input(egui_wants_pointer);

        egui_macroquad::draw();
        next_frame().await;
    }
}

// =============================================================================
// Helper types and functions
// =============================================================================

enum HexAction {
    SetColor(Color32),
    Pin,
    Revert,
}

fn find_best_gap(positions: &[f32]) -> f32 {
    if positions.is_empty() {
        return 0.5;
    }
    
    let mut sorted = positions.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let mut best_gap = sorted[0];
    let mut best_pos = sorted[0] / 2.0;
    
    for i in 0..sorted.len() - 1 {
        let gap = sorted[i + 1] - sorted[i];
        if gap > best_gap {
            best_gap = gap;
            best_pos = (sorted[i] + sorted[i + 1]) / 2.0;
        }
    }
    
    let gap_after = 1.0 - sorted.last().unwrap();
    if gap_after > best_gap {
        best_pos = (1.0 + sorted.last().unwrap()) / 2.0;
    }
    
    best_pos
}