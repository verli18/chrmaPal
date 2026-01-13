// ============================================================================
// Saturation Curves
// Maps luminosity (0.0 to 1.0) to saturation multiplier
// These curves control how saturation changes across the light-dark range
// ============================================================================

/// Curve types for saturation mapping
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SaturationCurve {
    /// No saturation change
    Flat,
    /// Linear increase from dark to light
    LinearUp,
    /// Linear decrease from dark to light  
    LinearDown,
    /// More saturation at extremes (dark and light), less in midtones
    Extremes,
    /// More saturation in midtones, less at extremes
    Midtones,
    /// Classic "dark more saturated" look
    DarkSaturated,
    /// Classic "light more saturated" look
    LightSaturated,
}

impl SaturationCurve {
    /// Get the saturation to ADD for a given luminosity
    /// - luminosity: 0.0 (black) to 1.0 (white)
    /// - strength: how much saturation to add at peak (0.0 = none, 1.0 = full)
    /// Returns: saturation amount to ADD (0.0 to ~0.15 in OkLCh chroma units)
    pub fn evaluate(&self, luminosity: f32, strength: f32) -> f32 {
        // Get curve shape value (0.0 to 1.0)
        let curve_value = match self {
            SaturationCurve::Flat => 0.5, // Constant mid-level
            
            SaturationCurve::LinearUp => {
                // 0 at dark, 1 at light
                luminosity
            }
            
            SaturationCurve::LinearDown => {
                // 1 at dark, 0 at light
                1.0 - luminosity
            }
            
            SaturationCurve::Extremes => {
                // U-shaped curve: high at 0 and 1, low at 0.5
                let centered = luminosity - 0.5;
                (centered.abs() * 2.0).powf(1.5)
            }
            
            SaturationCurve::Midtones => {
                // Inverted U-shaped curve: low at 0 and 1, high at 0.5
                let centered = luminosity - 0.5;
                1.0 - (centered.abs() * 2.0).powf(1.5)
            }
            
            SaturationCurve::DarkSaturated => {
                // More effect in shadows, smooth falloff
                (1.0 - luminosity).powf(0.7)
            }
            
            SaturationCurve::LightSaturated => {
                // More effect in highlights, smooth falloff
                luminosity.powf(0.7)
            }
        };
        
        // Return saturation (chroma) amount to ADD
        // Max chroma in OkLCh is around 0.4, so 0.15 is a reasonable max boost
        curve_value * strength * 0.15
    }
    
    /// Get display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            SaturationCurve::Flat => "Flat (no change)",
            SaturationCurve::LinearUp => "Linear + (light more sat)",
            SaturationCurve::LinearDown => "Linear - (dark more sat)",
            SaturationCurve::Extremes => "Extremes (U-curve)",
            SaturationCurve::Midtones => "Midtones (reverse U-curve)",
            SaturationCurve::DarkSaturated => "Dark saturated",
            SaturationCurve::LightSaturated => "Light saturated",
        }
    }
    
    /// Get all curve types for iteration
    pub fn all() -> &'static [SaturationCurve] {
        &[
            SaturationCurve::Flat,
            SaturationCurve::LinearUp,
            SaturationCurve::LinearDown,
            SaturationCurve::Extremes,
            SaturationCurve::Midtones,
            SaturationCurve::DarkSaturated,
            SaturationCurve::LightSaturated,
        ]
    }
}

// ============================================================================
// Hue Shift Curves
// Maps luminosity (0.0 to 1.0) to hue shift strength multiplier
// Controls where the hue shift effect is strongest
// ============================================================================

/// Curve types for hue shift strength
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HueShiftCurve {
    /// The original extremes-based shift (0 at middle, strong at extremes)
    Extremes,
    /// Uniform shift strength across all luminosities
    Flat,
    /// Linear increase from dark to light
    LinearUp,
    /// Linear decrease from dark to light
    LinearDown,
    /// More shift in shadows
    Shadows,
    /// More shift in highlights
    Highlights,
    /// More shift in midtones
    Midtones,
}

impl HueShiftCurve {
    /// Get the hue shift strength multiplier for a given luminosity
    /// - luminosity: 0.0 (black) to 1.0 (white)
    /// Returns: multiplier for hue shift strength (0.0 to 1.0)
    pub fn evaluate(&self, luminosity: f32) -> f32 {
        match self {
            HueShiftCurve::Extremes => {
                // 0 at middle, 1 at extremes (original behavior)
                (luminosity - 0.5).abs() * 2.0
            }
            
            HueShiftCurve::Flat => {
                // Uniform shift everywhere
                1.0
            }
            
            HueShiftCurve::LinearUp => {
                // 0 at dark, 1 at light
                luminosity
            }
            
            HueShiftCurve::LinearDown => {
                // 1 at dark, 0 at light
                1.0 - luminosity
            }
            
            HueShiftCurve::Shadows => {
                // More shift in shadows, smooth falloff
                (1.0 - luminosity).powf(0.5)
            }
            
            HueShiftCurve::Highlights => {
                // More shift in highlights, smooth falloff
                luminosity.powf(0.5)
            }
            
            HueShiftCurve::Midtones => {
                // Peak at 0.5, falls off toward extremes
                let centered = luminosity - 0.5;
                1.0 - (centered.abs() * 2.0).powf(1.5)
            }
        }
    }
    
    /// Get display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            HueShiftCurve::Extremes => "Extremes (default)",
            HueShiftCurve::Flat => "Flat (uniform)",
            HueShiftCurve::LinearUp => "Linear ↑",
            HueShiftCurve::LinearDown => "Linear ↓",
            HueShiftCurve::Shadows => "Shadows",
            HueShiftCurve::Highlights => "Highlights",
            HueShiftCurve::Midtones => "Midtones",
        }
    }
    
    /// Get all curve types for iteration
    pub fn all() -> &'static [HueShiftCurve] {
        &[
            HueShiftCurve::Extremes,
            HueShiftCurve::Flat,
            HueShiftCurve::LinearUp,
            HueShiftCurve::LinearDown,
            HueShiftCurve::Shadows,
            HueShiftCurve::Highlights,
            HueShiftCurve::Midtones,
        ]
    }
}

// ============================================================================
// Plotting helper - generates points for visualizing a curve
// ============================================================================

/// Generate points for plotting a saturation curve
/// Returns Vec of (x, y) where x is luminosity and y is the saturation to add
pub fn plot_curve(curve: SaturationCurve, strength: f32, num_points: usize) -> Vec<(f32, f32)> {
    (0..num_points)
        .map(|i| {
            let x = i as f32 / (num_points - 1) as f32;
            let y = curve.evaluate(x, strength);
            (x, y)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flat_curve() {
        let curve = SaturationCurve::Flat;
        // Flat curve returns 0.5 * strength * 0.15 = 0.075 at full strength
        let result = curve.evaluate(0.5, 1.0);
        assert!(result > 0.0 && result < 0.2);
    }
    
    #[test]
    fn test_strength_zero_gives_zero() {
        for curve in SaturationCurve::all() {
            // With strength 0, all curves should return 0.0 (no saturation added)
            assert_eq!(curve.evaluate(0.5, 0.0), 0.0);
        }
    }
}
