//! Custom egui widgets for the palette helper application

use egui_macroquad::egui::{self, Color32, Rect, Response, Sense, Stroke, Ui, Vec2};

// =============================================================================
// Drag and Drop List Support
// =============================================================================

/// State for tracking drag-and-drop operations using usize indices
#[derive(Default, Clone)]
pub struct DragDropState {
    /// Index of the item currently being dragged
    dragging_index: Option<usize>,
    /// Original position when drag started
    drag_start_pos: Option<egui::Pos2>,
}

impl DragDropState {
    pub fn is_dragging(&self, index: usize) -> bool {
        self.dragging_index == Some(index)
    }
    
    pub fn is_any_dragging(&self) -> bool {
        self.dragging_index.is_some()
    }
    
    pub fn dragging_index(&self) -> Option<usize> {
        self.dragging_index
    }
    
    pub fn start_drag(&mut self, index: usize, pos: egui::Pos2) {
        self.dragging_index = Some(index);
        self.drag_start_pos = Some(pos);
    }
    
    pub fn end_drag(&mut self) {
        self.dragging_index = None;
        self.drag_start_pos = None;
    }
}

/// Result of a drag-drop interaction
#[derive(Debug, Clone, Copy)]
pub enum DragDropResult {
    /// No interaction
    None,
    /// Item is being dragged
    Dragging,
    /// Item was dropped - swap source with target
    Dropped { source_index: usize, target_index: usize },
}

/// Draw a draggable list item with a drag handle
/// Returns the drag handle response, the drag result, and the inner content result
pub fn draggable_list_item<R>(
    ui: &mut Ui,
    state: &mut DragDropState,
    item_index: usize,
    is_highlighted: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> (Response, DragDropResult, R) {
    let mut result = DragDropResult::None;
    
    // Create a frame for the item
    let frame = if is_highlighted {
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(60, 100, 160, 80))
            .stroke(Stroke::new(1.0, Color32::from_rgb(100, 150, 220)))
            .inner_margin(4.0)
            .outer_margin(2.0)
            .corner_radius(4.0)
    } else if state.is_any_dragging() && !state.is_dragging(item_index) {
        // Show drop target indicator when something is being dragged over
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(50, 50, 60, 60))
            .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 120, 100)))
            .inner_margin(4.0)
            .outer_margin(2.0)
            .corner_radius(4.0)
    } else {
        egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(40, 40, 50, 60))
            .inner_margin(4.0)
            .outer_margin(2.0)
            .corner_radius(4.0)
    };
    
    let frame_response = frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Drag handle
            let handle_response = draw_drag_handle(ui, state.is_dragging(item_index));
            
            // Handle drag start
            if handle_response.drag_started() {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    state.start_drag(item_index, pos);
                }
            }
            
            // Draw contents
            let inner = add_contents(ui);
            
            (handle_response, inner)
        })
    });
    
    let item_rect = frame_response.response.rect;
    let (handle_response, inner) = frame_response.inner.inner;
    
    // Check for drop on this item (when another item is being dragged)
    if let Some(dragging_idx) = state.dragging_index() {
        if dragging_idx != item_index {
            // Check if pointer is over this item's rect
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                if item_rect.contains(pointer_pos) {
                    // Draw drop indicator
                    let painter = ui.painter();
                    let indicator_y = if pointer_pos.y < item_rect.center().y {
                        item_rect.top()
                    } else {
                        item_rect.bottom()
                    };
                    painter.line_segment(
                        [
                            egui::pos2(item_rect.left() + 4.0, indicator_y),
                            egui::pos2(item_rect.right() - 4.0, indicator_y),
                        ],
                        Stroke::new(2.0, Color32::from_rgb(100, 180, 255)),
                    );
                    
                    // Check for drop
                    if ui.input(|inp| inp.pointer.any_released()) {
                        result = DragDropResult::Dropped {
                            source_index: dragging_idx,
                            target_index: item_index,
                        };
                        state.end_drag();
                    }
                }
            }
        }
    }
    
    // Clear drag state if mouse released anywhere
    if ui.input(|inp| inp.pointer.any_released()) {
        if state.is_dragging(item_index) {
            result = DragDropResult::Dragging; // Was dragging but dropped elsewhere
        }
        state.end_drag();
    }
    
    // Show drag preview
    if state.is_dragging(item_index) {
        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            // Draw a ghost preview of the item being dragged
            let preview_rect = Rect::from_center_size(
                pointer_pos,
                Vec2::new(item_rect.width(), item_rect.height()),
            );
            
            let painter = ui.painter();
            painter.rect_filled(
                preview_rect,
                4.0,
                Color32::from_rgba_unmultiplied(80, 120, 180, 150),
            );
            painter.rect_stroke(
                preview_rect,
                4.0,
                Stroke::new(2.0, Color32::from_rgb(120, 160, 220)),
                egui::StrokeKind::Outside,
            );
        }
        result = DragDropResult::Dragging;
    }
    
    (handle_response, result, inner)
}

/// Draw a drag handle widget
fn draw_drag_handle(ui: &mut Ui, is_dragging: bool) -> Response {
    let size = Vec2::new(14.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let color = if is_dragging {
            Color32::from_rgb(150, 180, 220)
        } else if response.hovered() {
            Color32::from_rgb(120, 140, 160)
        } else {
            Color32::from_rgb(80, 90, 100)
        };
        
        // Draw three horizontal lines as drag indicator
        let line_spacing = 5.0;
        let line_width = 10.0;
        let center = rect.center();
        
        for i in -1..=1 {
            let y = center.y + i as f32 * line_spacing;
            painter.line_segment(
                [
                    egui::pos2(center.x - line_width / 2.0, y),
                    egui::pos2(center.x + line_width / 2.0, y),
                ],
                Stroke::new(2.0, color),
            );
        }
    }
    
    response
}

// =============================================================================
// Color Preview Widget
// =============================================================================

/// Draw a color preview as a single rectangle with sampled colors
pub fn draw_color_bar(ui: &mut Ui, colors: &[Color32], width: f32, height: f32) -> Response {
    let size = Vec2::new(width, height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    
    if ui.is_rect_visible(rect) && !colors.is_empty() {
        let painter = ui.painter();
        let segment_width = rect.width() / colors.len() as f32;
        
        for (i, color) in colors.iter().enumerate() {
            let segment_rect = Rect::from_min_size(
                egui::pos2(rect.min.x + i as f32 * segment_width, rect.min.y),
                Vec2::new(segment_width, rect.height()),
            );
            painter.rect_filled(segment_rect, 0.0, *color);
        }
        
        // Draw border around the whole preview
        painter.rect_stroke(
            rect, 
            2.0, 
            Stroke::new(1.0, Color32::from_rgb(60, 60, 70)),
            egui::StrokeKind::Outside,
        );
    }
    
    response
}

/// Draw a single color swatch
pub fn draw_color_swatch(ui: &mut Ui, color: Color32, size: Vec2) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 2.0, color);
        painter.rect_stroke(
            rect, 
            2.0, 
            Stroke::new(1.0, Color32::from_rgb(80, 80, 90)),
            egui::StrokeKind::Outside,
        );
    }
    
    response
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Sample N colors evenly from a color array
pub fn sample_colors(colors: &[Color32], n: usize) -> Vec<Color32> {
    if colors.is_empty() {
        return vec![Color32::BLACK; n];
    }
    if colors.len() <= n {
        return colors.to_vec();
    }
    
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let pos = i as f32 / (n - 1) as f32;
        let idx = (pos * (colors.len() - 1) as f32).round() as usize;
        result.push(colors[idx.min(colors.len() - 1)]);
    }
    result
}
