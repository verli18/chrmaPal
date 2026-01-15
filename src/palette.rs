use egui_macroquad::egui::Color32;
use crate::color::{ColorSpace, lerp_color, extrapolate_color};
use crate::curves::{Curve, CurveType};

// =============================================================================
// ControlPoint: A color at a specific position in the swatch
// =============================================================================
// 
// Control points define the "anchors" of our gradient. The `position` is a
// normalized value [0.0, 1.0] representing where in the swatch this color
// should appear. Position 0 = leftmost (brightest), Position 1 = rightmost (darkest).
// This allows control points at arbitrary positions, not just at discrete swatch indices.

#[derive(Clone, Debug)]
pub struct ControlPoint {
    /// Unique identifier for this control point (stable across reordering)
    pub id: u32,
    /// Normalized position in [0.0, 1.0] where this color appears
    /// 0.0 = left/bright, 1.0 = right/dark
    pub position: f32,
    /// The color at this control point
    pub color: Color32,
}

impl ControlPoint {
    pub fn new(id: u32, position: f32, color: Color32) -> Self {
        Self {
            id,
            position: position.clamp(0.0, 1.0),
            color,
        }
    }
}

// =============================================================================
// Swatch: A single color ramp with control points and interpolation
// =============================================================================
// 
// A swatch generates a sequence of colors by interpolating between control
// points. The interpolation curve determines how colors blend (linear, eased,
// etc.). Supports RGB, OkLab, and OkLCh color spaces.
//
// Direction: Position 0.0 = bright (left), Position 1.0 = dark (right)
// This is the standard convention for color palettes.
//
// Special cases:
// - No control points: all black
// - Single control point: extrapolate using the curve (lighter before, darker after)
// - Control points not at edges: extrapolate beyond them

#[derive(Clone, Debug)]
pub struct Swatch {
    /// Number of colors to generate in this swatch
    pub size: usize,
    /// Control points defining the gradient (sorted by position)
    control_points: Vec<ControlPoint>,
    /// The curve used for interpolation between control points
    pub interpolation_curve: CurveType,
    /// The color space to use for interpolation
    pub color_space: ColorSpace,
    /// Counter for generating unique control point IDs
    next_control_point_id: u32,
}

impl Default for Swatch {
    fn default() -> Self {
        Self {
            size: 8,
            control_points: vec![
                // Bright color at position 0 (left)
                ControlPoint::new(0, 0.0, Color32::from_rgb(240, 230, 220)),
                // Dark color at position 1 (right)
                ControlPoint::new(1, 1.0, Color32::from_rgb(20, 20, 40)),
            ],
            interpolation_curve: CurveType::default(),
            color_space: ColorSpace::default(),
            next_control_point_id: 2, // Start after the two default points
        }
    }
}

impl Swatch {
    pub fn new(size: usize, control_points: Vec<ControlPoint>, curve: CurveType, color_space: ColorSpace) -> Self {
        // Find the max ID in the provided control points to set next_id correctly
        let max_id = control_points.iter().map(|cp| cp.id).max().unwrap_or(0);
        let mut swatch = Self {
            size,
            control_points,
            interpolation_curve: curve,
            color_space,
            next_control_point_id: max_id + 1,
        };
        swatch.sort_control_points();
        swatch
    }

    /// Ensures control points are sorted by position (required for interpolation)
    fn sort_control_points(&mut self) {
        self.control_points
            .sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    }

    /// Generates all colors for this swatch by interpolating between control points.
    /// 
    /// The algorithm handles several cases:
    /// - No control points: return all black
    /// - Single control point: extrapolate darker before, lighter after
    /// - Multiple control points: piecewise interpolation with extrapolation at edges
    pub fn generate_colors(&self) -> Vec<Color32> {
        if self.control_points.is_empty() {
            return vec![Color32::BLACK; self.size];
        }

        let mut colors = Vec::with_capacity(self.size);

        for i in 0..self.size {
            // Normalized position in [0.0, 1.0]
            let t = if self.size > 1 {
                i as f32 / (self.size - 1) as f32
            } else {
                0.5 // Single slot: use middle position
            };

            let color = self.sample_at(t);
            colors.push(color);
        }

        colors
    }

    /// Sample the gradient at a normalized position t in [0.0, 1.0]
    fn sample_at(&self, t: f32) -> Color32 {
        if self.control_points.is_empty() {
            return Color32::BLACK;
        }

        // Single control point: extrapolate based on distance from it
        if self.control_points.len() == 1 {
            return self.sample_single_point(t);
        }

        let first = &self.control_points[0];
        let last = self.control_points.last().unwrap();

        // Before first control point: extrapolate lighter (toward position 0 = bright)
        if t < first.position {
            return self.extrapolate_before(t, first);
        }

        // After last control point: extrapolate darker (toward position 1 = dark)
        if t > last.position {
            return self.extrapolate_after(t, last);
        }

        // Between control points: interpolate
        self.interpolate_between(t)
    }

    /// Handle single control point case: extrapolate in both directions
    /// Position 0 = bright (left), Position 1 = dark (right)
    fn sample_single_point(&self, t: f32) -> Color32 {
        let cp = &self.control_points[0];
        
        if t < cp.position {
            // Before the control point: go lighter (toward bright/left)
            let distance = cp.position - t;
            // Apply curve to the distance for non-linear extrapolation
            let curved_distance = self.interpolation_curve.sample(distance.min(1.0));
            extrapolate_color(cp.color, curved_distance, self.color_space) // positive = lighter
        } else if t > cp.position {
            // After the control point: go darker (toward dark/right)
            let distance = t - cp.position;
            let curved_distance = self.interpolation_curve.sample(distance.min(1.0));
            extrapolate_color(cp.color, -curved_distance, self.color_space) // negative = darker
        } else {
            cp.color
        }
    }

    /// Extrapolate before the first control point (toward lighter/brighter)
    fn extrapolate_before(&self, t: f32, first: &ControlPoint) -> Color32 {
        // How far before the first point (normalized to the "before" region)
        let region_size = first.position;
        if region_size <= 0.0 {
            return first.color;
        }
        
        // Distance from t to first point, normalized to [0, 1]
        let normalized_distance = (first.position - t) / region_size;
        let curved_distance = self.interpolation_curve.sample(normalized_distance);
        
        // Positive = lighter (going toward position 0 = bright)
        extrapolate_color(first.color, curved_distance, self.color_space)
    }

    /// Extrapolate after the last control point (toward darker)
    fn extrapolate_after(&self, t: f32, last: &ControlPoint) -> Color32 {
        let region_size = 1.0 - last.position;
        if region_size <= 0.0 {
            return last.color;
        }
        
        let normalized_distance = (t - last.position) / region_size;
        let curved_distance = self.interpolation_curve.sample(normalized_distance);
        
        // Negative = darker (going toward position 1 = dark)
        extrapolate_color(last.color, -curved_distance, self.color_space)
    }

    /// Interpolate between control points (t is within the control point range)
    fn interpolate_between(&self, t: f32) -> Color32 {
        // Find the two control points that bracket position t
        let (cp_before, cp_after) = self.find_bracketing_points(t);

        // Calculate local_t: how far between cp_before and cp_after we are
        let segment_length = cp_after.position - cp_before.position;
        let local_t = if segment_length > 0.0 {
            (t - cp_before.position) / segment_length
        } else {
            0.0 // Both points at same position, just use first color
        };

        // Apply the interpolation curve to get the curved interpolation factor
        let curved_t = self.interpolation_curve.sample(local_t);

        // Lerp between the two colors in the selected color space
        lerp_color(cp_before.color, cp_after.color, curved_t, self.color_space)
    }

    /// Find the two control points that bracket position t.
    /// Returns (before, after) where before.position <= t <= after.position
    fn find_bracketing_points(&self, t: f32) -> (&ControlPoint, &ControlPoint) {
        // Find the segment containing t
        for i in 0..self.control_points.len() - 1 {
            let current = &self.control_points[i];
            let next = &self.control_points[i + 1];
            if t >= current.position && t <= next.position {
                return (current, next);
            }
        }

        // Fallback (shouldn't happen if t is within range)
        let last = self.control_points.last().unwrap();
        (last, last)
    }

    // =========================================================================
    // Control point management
    // =========================================================================

    pub fn control_points(&self) -> &[ControlPoint] {
        &self.control_points
    }

    pub fn control_points_mut(&mut self) -> &mut Vec<ControlPoint> {
        &mut self.control_points
    }

    pub fn add_control_point(&mut self, position: f32, color: Color32) {
        let id = self.next_control_point_id;
        self.next_control_point_id += 1;
        self.control_points.push(ControlPoint::new(id, position, color));
        self.sort_control_points();
    }

    /// Remove a control point by index. Now allows removing all points.
    pub fn remove_control_point(&mut self, index: usize) {
        if index < self.control_points.len() {
            self.control_points.remove(index);
        }
    }

    /// Remove a control point by its stable ID
    pub fn remove_control_point_by_id(&mut self, id: u32) {
        self.control_points.retain(|cp| cp.id != id);
    }

    pub fn set_control_point_color(&mut self, index: usize, color: Color32) {
        if let Some(cp) = self.control_points.get_mut(index) {
            cp.color = color;
        }
    }

    /// Set control point color by its stable ID
    pub fn set_control_point_color_by_id(&mut self, id: u32, color: Color32) {
        if let Some(cp) = self.control_points.iter_mut().find(|cp| cp.id == id) {
            cp.color = color;
        }
    }

    /// Swap the positions of two control points by their IDs
    /// This swaps their positions in the gradient, not their indices in the vector
    pub fn swap_control_points_by_id(&mut self, id_a: u32, id_b: u32) {
        // Find positions of both control points
        let pos_a = self.control_points.iter().find(|cp| cp.id == id_a).map(|cp| cp.position);
        let pos_b = self.control_points.iter().find(|cp| cp.id == id_b).map(|cp| cp.position);
        
        if let (Some(pos_a), Some(pos_b)) = (pos_a, pos_b) {
            // Swap their positions
            if let Some(cp) = self.control_points.iter_mut().find(|cp| cp.id == id_a) {
                cp.position = pos_b;
            }
            if let Some(cp) = self.control_points.iter_mut().find(|cp| cp.id == id_b) {
                cp.position = pos_a;
            }
            self.sort_control_points();
        }
    }

    /// Set control point position by its stable ID, then re-sort
    pub fn set_control_point_position_by_id(&mut self, id: u32, position: f32) {
        if let Some(cp) = self.control_points.iter_mut().find(|cp| cp.id == id) {
            cp.position = position.clamp(0.0, 1.0);
        }
        self.sort_control_points();
    }

    /// Find the index of a control point by its ID (useful after sorting)
    pub fn find_control_point_index_by_id(&self, id: u32) -> Option<usize> {
        self.control_points.iter().position(|cp| cp.id == id)
    }

    pub fn set_control_point_position(&mut self, index: usize, position: f32) {
        if let Some(cp) = self.control_points.get_mut(index) {
            cp.position = position.clamp(0.0, 1.0);
        }
        self.sort_control_points();
    }

    /// Check if a position already has a control point (within a tolerance)
    pub fn has_control_point_at(&self, position: f32, tolerance: f32) -> Option<usize> {
        self.control_points
            .iter()
            .position(|cp| (cp.position - position).abs() <= tolerance)
    }
}

// =============================================================================
// Palette: A collection of swatches
// =============================================================================

#[derive(Clone, Debug, Default)]
pub struct Palette {
    pub swatches: Vec<Swatch>,
}

impl Palette {
    pub fn new() -> Self {
        Self {
            swatches: vec![Swatch::default()],
        }

    }

    pub fn add_swatch(&mut self, swatch: Swatch) {
        self.swatches.push(swatch);
    }
}