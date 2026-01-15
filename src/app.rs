use egui_macroquad::egui::Color32;

use crate::palette::{Palette, Swatch};
use crate::viewport::Viewport;

// =============================================================================
// App: Central application state
// =============================================================================

pub struct App {
    /// The palette containing all swatches
    pub palette: Palette,
    /// Index of the currently selected swatch
    pub current_swatch_index: usize,
    /// Viewport for panning and zooming
    pub viewport: Viewport,
    /// Cached generated colors for each swatch (regenerated when swatches change)
    pub generated_colors: Vec<Vec<Color32>>,
}

impl App {
    pub fn new() -> Self {
        let palette = Palette::new();
        
        let mut app = Self {
            palette,
            current_swatch_index: 0,
            viewport: Viewport::default(),
            generated_colors: Vec::new(),
        };
        
        app.regenerate_all_colors();
        app
    }

    /// Get the currently selected swatch
    pub fn current_swatch(&self) -> &Swatch {
        &self.palette.swatches[self.current_swatch_index]
    }

    /// Get the currently selected swatch mutably
    pub fn current_swatch_mut(&mut self) -> &mut Swatch {
        &mut self.palette.swatches[self.current_swatch_index]
    }

    /// Regenerate colors for all swatches
    pub fn regenerate_all_colors(&mut self) {
        self.generated_colors = self
            .palette
            .swatches
            .iter()
            .map(|swatch| swatch.generate_colors())
            .collect();
    }

    /// Regenerate colors for the current swatch only
    pub fn regenerate_current_colors(&mut self) {
        if self.current_swatch_index < self.generated_colors.len() {
            self.generated_colors[self.current_swatch_index] =
                self.palette.swatches[self.current_swatch_index].generate_colors();
        }
    }

    /// Add a new swatch to the palette
    pub fn add_swatch(&mut self, swatch: Swatch) {
        self.palette.swatches.push(swatch);
        self.generated_colors.push(Vec::new());
        // Regenerate colors for the new swatch
        let idx = self.palette.swatches.len() - 1;
        self.generated_colors[idx] = self.palette.swatches[idx].generate_colors();
    }

    /// Remove a swatch from the palette by index
    pub fn remove_swatch(&mut self, index: usize) {
        if self.palette.swatches.len() <= 1 {
            // Don't allow removing the last swatch
            return;
        }
        
        if index < self.palette.swatches.len() {
            self.palette.swatches.remove(index);
            self.generated_colors.remove(index);
            
            // Adjust current swatch index if needed
            if self.current_swatch_index >= self.palette.swatches.len() {
                self.current_swatch_index = self.palette.swatches.len() - 1;
            }
        }
    }

    /// Move a swatch up in the list (decrease index)
    pub fn move_swatch_up(&mut self, index: usize) {
        if index > 0 && index < self.palette.swatches.len() {
            self.palette.swatches.swap(index, index - 1);
            self.generated_colors.swap(index, index - 1);
            
            // Update current swatch index if we moved the selected swatch
            if self.current_swatch_index == index {
                self.current_swatch_index = index - 1;
            } else if self.current_swatch_index == index - 1 {
                self.current_swatch_index = index;
            }
        }
    }

    /// Move a swatch down in the list (increase index)
    pub fn move_swatch_down(&mut self, index: usize) {
        if index + 1 < self.palette.swatches.len() {
            self.palette.swatches.swap(index, index + 1);
            self.generated_colors.swap(index, index + 1);
            
            // Update current swatch index if we moved the selected swatch
            if self.current_swatch_index == index {
                self.current_swatch_index = index + 1;
            } else if self.current_swatch_index == index + 1 {
                self.current_swatch_index = index;
            }
        }
    }

    /// Duplicate a swatch
    pub fn duplicate_swatch(&mut self, index: usize) {
        if index < self.palette.swatches.len() {
            let swatch_clone = self.palette.swatches[index].clone();
            
            // Insert after the original
            let insert_idx = index + 1;
            self.palette.swatches.insert(insert_idx, swatch_clone);
            self.generated_colors.insert(insert_idx, Vec::new());
            
            // Regenerate colors for the new swatch
            self.generated_colors[insert_idx] = self.palette.swatches[insert_idx].generate_colors();
        }
    }

    /// Swap two swatches by index
    pub fn swap_swatches(&mut self, a: usize, b: usize) {
        let len = self.palette.swatches.len();
        if a >= len || b >= len || a == b {
            return;
        }
        
        self.palette.swatches.swap(a, b);
        self.generated_colors.swap(a, b);
        
        // Update current swatch index to follow the selected swatch
        if self.current_swatch_index == a {
            self.current_swatch_index = b;
        } else if self.current_swatch_index == b {
            self.current_swatch_index = a;
        }
    }

    /// Select a swatch by index
    pub fn select_swatch(&mut self, index: usize) {
        if index < self.palette.swatches.len() {
            self.current_swatch_index = index;
        }
    }

    /// Get the number of swatches
    pub fn swatch_count(&self) -> usize {
        self.palette.swatches.len()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
