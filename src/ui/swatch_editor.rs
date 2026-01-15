use egui_macroquad::egui::{self, Color32, Slider, Vec2};

use crate::app::App;
use crate::color::ColorSpace;
use crate::curves::{CurveKind, CurveType, EaseIn, EaseInOut, EaseOut, Linear};
use crate::ui::widgets::{draggable_list_item, draw_color_swatch, DragDropResult, DragDropState};

// =============================================================================
// HexEditState: Tracks color edits in the generated palette
// =============================================================================

/// State for tracking color edits - allows editing colors then pinning as control points
#[derive(Default, Clone)]
pub struct HexEditState {
    /// The colors as they were when we last synced (for change detection)
    original_colors: Vec<Color32>,
    /// Current edited colors (may differ from original if user edited)
    edited_colors: Vec<Color32>,
}

impl HexEditState {
    /// Sync with generated colors - resets edits when the underlying palette changes
    pub fn sync_with_generated(&mut self, generated: &[Color32]) {
        if self.original_colors.len() != generated.len() {
            // Size changed, reset everything
            self.original_colors = generated.to_vec();
            self.edited_colors = generated.to_vec();
        } else {
            // Check if generated colors changed (due to control point changes)
            let generated_changed = self.original_colors.iter()
                .zip(generated.iter())
                .any(|(a, b)| a != b);
            
            if generated_changed {
                self.original_colors = generated.to_vec();
                self.edited_colors = generated.to_vec();
            }
        }
    }

    /// Check if a specific color slot was edited by the user
    pub fn was_edited(&self, index: usize) -> bool {
        if index >= self.original_colors.len() || index >= self.edited_colors.len() {
            return false;
        }
        self.original_colors[index] != self.edited_colors[index]
    }

    /// Get the current color at an index (edited or original)
    pub fn get(&self, index: usize) -> Option<Color32> {
        self.edited_colors.get(index).copied()
    }

    /// Set an edited color
    pub fn set(&mut self, index: usize, color: Color32) {
        if index < self.edited_colors.len() {
            self.edited_colors[index] = color;
        }
    }

    /// Revert an edit back to the original generated color
    pub fn clear_edit(&mut self, index: usize) {
        if index < self.edited_colors.len() && index < self.original_colors.len() {
            self.edited_colors[index] = self.original_colors[index];
        }
    }
}

// =============================================================================
// SwatchEditorState
// =============================================================================

/// UI state for the swatch editor
pub struct SwatchEditorState {
    pub selected_curve_kind: CurveKind,
    pub curve_exponent: f32,
    pub linear_factor: f32,
    /// State for editing colors in the generated palette
    pub hex_edit_state: HexEditState,
    /// State for drag-drop reordering of control points
    pub control_point_drag_state: DragDropState,
}

impl Default for SwatchEditorState {
    fn default() -> Self {
        Self {
            selected_curve_kind: CurveKind::Linear,
            curve_exponent: 2.0,
            linear_factor: 1.0,
            hex_edit_state: HexEditState::default(),
            control_point_drag_state: DragDropState::default(),
        }
    }
}

impl SwatchEditorState {
    /// Sync state with the current swatch
    pub fn sync_with_swatch(&mut self, app: &App) {
        let swatch = app.current_swatch();
        self.selected_curve_kind = swatch.interpolation_curve.kind();
        
        match &swatch.interpolation_curve {
            CurveType::Linear(l) => self.linear_factor = l.factor,
            CurveType::EaseIn(e) => self.curve_exponent = e.exponent,
            CurveType::EaseOut(e) => self.curve_exponent = e.exponent,
            CurveType::EaseInOut(e) => self.curve_exponent = e.exponent,
            CurveType::Bezier(_) => {}
        }
    }
}

// =============================================================================
// Actions
// =============================================================================

enum ControlPointAction {
    Swap(u32, u32),
    Remove(u32),
}

enum ColorAction {
    SetColor(usize, Color32),
    Pin(usize),
    Revert(usize),
}

// =============================================================================
// Draw Functions
// =============================================================================

/// Draw the swatch editor window
pub fn draw_swatch_editor(
    egui_ctx: &egui::Context,
    app: &mut App,
    state: &mut SwatchEditorState,
) {
    // Sync hex edit state with generated colors
    if app.current_swatch_index < app.generated_colors.len() {
        state.hex_edit_state.sync_with_generated(&app.generated_colors[app.current_swatch_index]);
    }

    egui::Window::new("Swatch Editor").show(egui_ctx, |ui| {
        // Swatch size control
        let mut size = app.current_swatch().size;
        if ui
            .add(Slider::new(&mut size, 2..=32).text("Swatch size"))
            .changed()
        {
            app.current_swatch_mut().size = size;
            app.regenerate_current_colors();
        }

        ui.separator();

        // Color space selector
        ui.horizontal(|ui| {
            ui.label("Color Space:");
            let current_space = app.current_swatch().color_space;
            egui::ComboBox::from_id_salt("color_space")
                .selected_text(current_space.name())
                .show_ui(ui, |ui| {
                    for &space in ColorSpace::ALL {
                        if ui
                            .selectable_value(
                                &mut app.current_swatch_mut().color_space,
                                space,
                                space.name(),
                            )
                            .changed()
                        {
                            app.regenerate_current_colors();
                        }
                    }
                });
        });

        ui.separator();
        
        // Control points section
        draw_control_points_section(ui, app, state);

        ui.separator();
        
        // Interpolation curve section
        ui.label("Interpolation Curve:");
        draw_curve_editor(ui, app, state);

        ui.separator();

        // Editable color values section
        draw_color_values_section(ui, app, state);
    });
}

fn draw_control_points_section(ui: &mut egui::Ui, app: &mut App, state: &mut SwatchEditorState) {
    ui.label("Control Points (drag to reorder):");

    // Get control point data - we use the index in the control_points vec
    let control_points: Vec<(usize, u32, f32, Color32)> = app
        .current_swatch()
        .control_points()
        .iter()
        .enumerate()
        .map(|(idx, cp)| (idx, cp.id, cp.position, cp.color))
        .collect();

    if control_points.is_empty() {
        ui.label("No control points. Edit colors below and click 'Pin' to add.");
        return;
    }

    let mut actions: Vec<ControlPointAction> = Vec::new();

    for (idx, id, pos, color) in &control_points {
        let idx = *idx;
        let id = *id;
        
        let (_handle, drag_result, _) = draggable_list_item(
            ui,
            &mut state.control_point_drag_state,
            idx,
            false,
            |ui| {
                // Position display
                ui.label(format!("{:.0}%", pos * 100.0));


                // Color picker
                let mut new_color = *color;
                if ui.color_edit_button_srgba(&mut new_color).changed() {
                    app.current_swatch_mut().set_control_point_color_by_id(id, new_color);
                    app.regenerate_current_colors();
                }

                // Delete button
                if ui.button("×").clicked() {
                    actions.push(ControlPointAction::Remove(id));
                }
            },
        );

        // Handle drag-drop result
        if let DragDropResult::Dropped { source_index, target_index } = drag_result {
            // Get the IDs from the indices
            if source_index < control_points.len() && target_index < control_points.len() {
                let source_id = control_points[source_index].1;
                let target_id = control_points[target_index].1;
                actions.push(ControlPointAction::Swap(source_id, target_id));
            }
        }
    }

    // Apply actions
    for action in actions {
        match action {
            ControlPointAction::Swap(id_a, id_b) => {
                app.current_swatch_mut().swap_control_points_by_id(id_a, id_b);
            }
            ControlPointAction::Remove(id) => {
                app.current_swatch_mut().remove_control_point_by_id(id);
            }
        }
        app.regenerate_current_colors();
    }
}

fn draw_curve_editor(ui: &mut egui::Ui, app: &mut App, state: &mut SwatchEditorState) {
    egui::ComboBox::from_label("Curve Type")
        .selected_text(format!("{:?}", state.selected_curve_kind))
        .show_ui(ui, |ui| {
            for kind in [
                CurveKind::Linear,
                CurveKind::EaseIn,
                CurveKind::EaseOut,
                CurveKind::EaseInOut,
            ] {
                if ui
                    .selectable_value(&mut state.selected_curve_kind, kind, format!("{:?}", kind))
                    .changed()
                {
                    app.current_swatch_mut().interpolation_curve = match kind {
                        CurveKind::Linear => CurveType::Linear(Linear {
                            factor: state.linear_factor,
                        }),
                        CurveKind::EaseIn => CurveType::EaseIn(EaseIn {
                            exponent: state.curve_exponent,
                        }),
                        CurveKind::EaseOut => CurveType::EaseOut(EaseOut {
                            exponent: state.curve_exponent,
                        }),
                        CurveKind::EaseInOut => CurveType::EaseInOut(EaseInOut {
                            exponent: state.curve_exponent,
                        }),
                        CurveKind::Bezier => CurveType::from_kind(kind),
                    };
                    app.regenerate_current_colors();
                }
            }
        });

    let mut needs_regenerate = false;

    match state.selected_curve_kind {
        CurveKind::Linear => {
            if ui
                .add(Slider::new(&mut state.linear_factor, 0.1..=2.0).text("Factor"))
                .changed()
            {
                app.current_swatch_mut().interpolation_curve = CurveType::Linear(Linear {
                    factor: state.linear_factor,
                });
                needs_regenerate = true;
            }
        }
        CurveKind::EaseIn | CurveKind::EaseOut | CurveKind::EaseInOut => {
            if ui
                .add(Slider::new(&mut state.curve_exponent, 0.5..=5.0).text("Exponent"))
                .changed()
            {
                app.current_swatch_mut().interpolation_curve = match state.selected_curve_kind {
                    CurveKind::EaseIn => CurveType::EaseIn(EaseIn {
                        exponent: state.curve_exponent,
                    }),
                    CurveKind::EaseOut => CurveType::EaseOut(EaseOut {
                        exponent: state.curve_exponent,
                    }),
                    CurveKind::EaseInOut => CurveType::EaseInOut(EaseInOut {
                        exponent: state.curve_exponent,
                    }),
                    _ => unreachable!(),
                };
                needs_regenerate = true;
            }
        }
        CurveKind::Bezier => {
            ui.label("Bezier controls not yet implemented");
        }
    }

    if needs_regenerate {
        app.regenerate_current_colors();
    }
}

fn draw_color_values_section(ui: &mut egui::Ui, app: &mut App, state: &mut SwatchEditorState) {
    ui.collapsing("Colors (editable)", |ui| {
        ui.label("Edit colors. Changed colors show ● - Pin to create control point.");
        ui.add_space(4.0);

        let num_colors = state.hex_edit_state.edited_colors.len();
        let mut action: Option<ColorAction> = None;

        for i in 0..num_colors {
            let was_edited = state.hex_edit_state.was_edited(i);
            
            ui.horizontal(|ui| {
                // Index
                ui.label(format!("{:2}:", i + 1));
                
                // Edit indicator
                if was_edited {
                    ui.label("●");
                } else {
                    ui.label(" ");
                }
                
                // Color swatch and picker
                let color = state.hex_edit_state.get(i).unwrap_or(Color32::BLACK);
                draw_color_swatch(ui, color, Vec2::new(24.0, 16.0));
                
                let mut edit_color = color;
                if ui.color_edit_button_srgba(&mut edit_color).changed() {
                    action = Some(ColorAction::SetColor(i, edit_color));
                }
                
                // Hex display
                ui.label(format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()));
                
                // Pin/Revert buttons (only show when edited)
                if was_edited {
                    if ui.button("Pin").clicked() {
                        action = Some(ColorAction::Pin(i));
                    }
                    if ui.button("↩").clicked() {
                        action = Some(ColorAction::Revert(i));
                    }
                }
            });
        }

        // Apply actions
        if let Some(act) = action {
            match act {
                ColorAction::SetColor(idx, color) => {
                    state.hex_edit_state.set(idx, color);
                }
                ColorAction::Pin(idx) => {
                    // Calculate position based on index
                    let position = if num_colors > 1 {
                        idx as f32 / (num_colors - 1) as f32
                    } else {
                        0.5
                    };
                    
                    let color = state.hex_edit_state.get(idx).unwrap_or(Color32::BLACK);
                    let tolerance = 0.5 / num_colors as f32;
                    
                    // Check if there's already a control point near this position
                    if let Some(cp_idx) = app.current_swatch().has_control_point_at(position, tolerance) {
                        // Update existing control point
                        app.current_swatch_mut().set_control_point_color(cp_idx, color);
                    } else {
                        // Add new control point
                        app.current_swatch_mut().add_control_point(position, color);
                    }
                    
                    state.hex_edit_state.clear_edit(idx);
                    app.regenerate_current_colors();
                }
                ColorAction::Revert(idx) => {
                    state.hex_edit_state.clear_edit(idx);
                }
            }
        }
    });
}
