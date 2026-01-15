use macroquad::prelude::*;

// =============================================================================
// Viewport: Handles camera panning and zooming
// =============================================================================

pub struct Viewport {
    /// Camera offset in world coordinates (what world position is at screen center)
    pub offset: Vec2,
    /// Zoom level (1.0 = normal, 2.0 = zoomed in 2x)
    pub zoom: f32,
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
    pub fn handle_input(&mut self, egui_wants_input: bool) -> bool {
        if egui_wants_input {
            return false;
        }

        let mut consumed = false;

        // Pan with left mouse button drag
        if is_mouse_button_down(MouseButton::Left) {
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
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let offset_from_center = screen_pos - screen_center;
        self.offset + offset_from_center / self.zoom
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        screen_center + (world_pos - self.offset) * self.zoom
    }

    /// Get the visible world bounds (for culling)
    #[allow(dead_code)]
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let top_left = self.screen_to_world(Vec2::ZERO);
        let bottom_right = self.screen_to_world(Vec2::new(screen_width(), screen_height()));
        (top_left, bottom_right)
    }
}
