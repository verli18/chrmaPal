use egui_macroquad::egui::Color32;
use crate::curves::{SaturationCurve, HueShiftCurve};

pub fn lerp_color(col1: Color32, col2: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (col1.r() as f32 + (col2.r() as f32 - col1.r() as f32) * t) as u8,
        (col1.g() as f32 + (col2.g() as f32 - col1.g() as f32) * t) as u8,
        (col1.b() as f32 + (col2.b() as f32 - col1.b() as f32) * t) as u8,
    )
}

fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

pub fn rgb_to_oklab(col: Color32) -> (f32, f32, f32) {
    // Convert sRGB to linear RGB
    let r = srgb_to_linear(col.r() as f32 / 255.0);
    let g = srgb_to_linear(col.g() as f32 / 255.0);
    let b = srgb_to_linear(col.b() as f32 / 255.0);

    // Convert linear RGB to LMS cone space
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    // Apply cube root
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // Convert to Lab
    let lab_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let lab_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let lab_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    (lab_l, lab_a, lab_b)
}

fn oklab_to_rgb(lab_l: f32, lab_a: f32, lab_b: f32) -> Color32 {
    // Convert Lab to LMS
    let l_ = lab_l + 0.3963377774 * lab_a + 0.2158037573 * lab_b;
    let m_ = lab_l - 0.1055613458 * lab_a - 0.0638541728 * lab_b;
    let s_ = lab_l - 0.0894841775 * lab_a - 1.2914855480 * lab_b;

    // Cube the values
    let l = l_.powi(3);
    let m = m_.powi(3);
    let s = s_.powi(3);

    // Convert LMS to linear RGB
    let r_linear = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g_linear = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b_linear = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    // Convert linear RGB to sRGB and clamp
    let r_srgb = linear_to_srgb(r_linear.clamp(0.0, 1.0));
    let g_srgb = linear_to_srgb(g_linear.clamp(0.0, 1.0));
    let b_srgb = linear_to_srgb(b_linear.clamp(0.0, 1.0));

    Color32::from_rgb(
        (r_srgb * 255.0) as u8,
        (g_srgb * 255.0) as u8,
        (b_srgb * 255.0) as u8,
    )
}
    
pub fn lerp_color_oklab(col1: Color32, col2: Color32, t: f32) -> Color32 {
    let (l1, a1, b1) = rgb_to_oklab(col1);
    let (l2, a2, b2) = rgb_to_oklab(col2);

    let l = l1 + (l2 - l1) * t;
    let a = a1 + (a2 - a1) * t;
    let b = b1 + (b2 - b1) * t;

    oklab_to_rgb(l, a, b)
}

// Convert OkLab to OkLCh (cylindrical coordinates)
fn oklab_to_oklch(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let c = (a * a + b * b).sqrt(); // Chroma
    let h = b.atan2(a).to_degrees(); // Hue in degrees
    (l, c, h)
}

// Convert OkLCh back to OkLab
fn oklch_to_oklab(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    (l, a, b)
}

// ============================================================================
// Gamut Mapping Utilities
// Soft clipping and gamut mapping to avoid harsh color clipping
// ============================================================================

// Soft clamp using a smooth transition near the limit
fn soft_clamp(value: f32, max: f32) -> f32 {
    if value <= max * 0.8 {
        value
    } else {
        // Smoothly compress values above 80% of max
        let excess = value - max * 0.8;
        let headroom = max * 0.2;
        max * 0.8 + headroom * (1.0 - (-excess / headroom).exp())
    }
}

// Approximate maximum chroma for a given luminosity in sRGB gamut
// This is a rough approximation - real gamut boundaries are complex
fn max_chroma_for_luminosity(l: f32) -> f32 {
    // Parabolic approximation: max chroma is highest around L=0.5-0.7
    // and drops off toward black and white
    let peak_l = 0.6;
    let width = 0.5;
    let max_c = 0.35;
    
    let dist = (l - peak_l).abs() / width;
    max_c * (1.0 - dist.powi(2)).max(0.05)
}

// Soft gamut mapping: preserve hue while reducing chroma if out of gamut
fn soft_gamut_map_oklab(l: f32, a: f32, b: f32) -> Color32 {
    // Convert to RGB to check gamut
    let (r, g, b_lin) = oklab_to_linear_rgb(l, a, b);
    
    // Check if in gamut
    if r >= 0.0 && r <= 1.0 && g >= 0.0 && g <= 1.0 && b_lin >= 0.0 && b_lin <= 1.0 {
        // In gamut, convert normally
        let r_srgb = linear_to_srgb(r);
        let g_srgb = linear_to_srgb(g);
        let b_srgb = linear_to_srgb(b_lin);
        return Color32::from_rgb(
            (r_srgb * 255.0) as u8,
            (g_srgb * 255.0) as u8,
            (b_srgb * 255.0) as u8,
        );
    }
    
    // Out of gamut: reduce chroma while preserving hue
    let (_, c, h) = oklab_to_oklch(l, a, b);
    
    // Binary search for maximum in-gamut chroma
    let mut low = 0.0;
    let mut high = c;
    
    for _ in 0..16 {
        let mid = (low + high) / 2.0;
        let (test_l, test_a, test_b) = oklch_to_oklab(l, mid, h);
        let (tr, tg, tb) = oklab_to_linear_rgb(test_l, test_a, test_b);
        
        if tr >= 0.0 && tr <= 1.0 && tg >= 0.0 && tg <= 1.0 && tb >= 0.0 && tb <= 1.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    
    let (final_l, final_a, final_b) = oklch_to_oklab(l, low, h);
    let (fr, fg, fb) = oklab_to_linear_rgb(final_l, final_a, final_b);
    
    let r_srgb = linear_to_srgb(fr.clamp(0.0, 1.0));
    let g_srgb = linear_to_srgb(fg.clamp(0.0, 1.0));
    let b_srgb = linear_to_srgb(fb.clamp(0.0, 1.0));
    
    Color32::from_rgb(
        (r_srgb * 255.0) as u8,
        (g_srgb * 255.0) as u8,
        (b_srgb * 255.0) as u8,
    )
}

// Helper: OkLab to linear RGB (without clamping/conversion to sRGB)
fn oklab_to_linear_rgb(lab_l: f32, lab_a: f32, lab_b: f32) -> (f32, f32, f32) {
    let l_ = lab_l + 0.3963377774 * lab_a + 0.2158037573 * lab_b;
    let m_ = lab_l - 0.1055613458 * lab_a - 0.0638541728 * lab_b;
    let s_ = lab_l - 0.0894841775 * lab_a - 1.2914855480 * lab_b;

    let l = l_.powi(3);
    let m = m_.powi(3);
    let s = s_.powi(3);

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;
    
    (r, g, b)
}

// ============================================================================
// OkLCh-based Palette Ramp Generation
// This is the recommended approach for pixel art palettes
// Works by interpolating in OkLCh space with hue rotation
// ============================================================================

/// Convert a Color32 to OkLCh (Lightness, Chroma, Hue)
pub fn rgb_to_oklch(col: Color32) -> (f32, f32, f32) {
    let (l, a, b) = rgb_to_oklab(col);
    oklab_to_oklch(l, a, b)
}

/// Interpolate hue along the shortest path, with optional extra rotation
fn lerp_hue(h1: f32, h2: f32, t: f32, extra_rotation: f32) -> f32 {
    // Normalize hues to 0-360
    let h1 = ((h1 % 360.0) + 360.0) % 360.0;
    let h2 = ((h2 % 360.0) + 360.0) % 360.0;
    
    // Find shortest path
    let mut diff = h2 - h1;
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    
    // Add extra rotation (positive = rotate through more colors)
    let total_rotation = diff + extra_rotation;
    
    let result = h1 + total_rotation * t;
    ((result % 360.0) + 360.0) % 360.0
}

/// Generate a single color in a ramp using OkLCh interpolation
/// - col1, col2: endpoint colors
/// - t: position in ramp (0.0 = col1, 1.0 = col2)
/// - hue_shift: extra hue rotation in degrees (positive = shift through warm, negative = through cold)
/// - chroma_curve: how chroma varies along the ramp
/// - chroma_boost: multiplier for chroma (1.0 = use interpolated chroma)
pub fn generate_ramp_color_oklch(
    col1: Color32,
    col2: Color32,
    t: f32,
    hue_shift: f32,
    chroma_curve: SaturationCurve,
    chroma_boost: f32,
) -> Color32 {
    let (l1, c1, h1) = rgb_to_oklch(col1);
    let (l2, c2, h2) = rgb_to_oklch(col2);
    
    // Interpolate lightness linearly
    let l = l1 + (l2 - l1) * t;
    
    // Interpolate hue with extra rotation for "hue shifting" effect
    // The hue_shift adds rotation: positive pushes through warmer hues
    let h = lerp_hue(h1, h2, t, hue_shift);
    
    // Base chroma interpolation
    let base_c = c1 + (c2 - c1) * t;
    
    // Apply chroma curve - this shapes the saturation along the ramp
    // The curve returns a value 0-0.15 based on lightness, we use it as an additive boost
    let curve_boost = chroma_curve.evaluate(l, chroma_boost);
    
    // Combine: base chroma + curve-based boost
    // Also scale by chroma_boost for overall saturation control
    let c = (base_c * (0.5 + chroma_boost * 0.5) + curve_boost).max(0.0);
    
    // Convert back through gamut mapping
    let (lab_l, lab_a, lab_b) = oklch_to_oklab(l, c, h);
    soft_gamut_map_oklab(lab_l, lab_a, lab_b)
}

/// Lerp in OkLCh with proper hue interpolation (no extra hue shift, for blending mode)
pub fn lerp_color_oklch(col1: Color32, col2: Color32, t: f32) -> Color32 {
    let (l1, c1, h1) = rgb_to_oklch(col1);
    let (l2, c2, h2) = rgb_to_oklch(col2);
    
    // Interpolate lightness and chroma linearly
    let l = l1 + (l2 - l1) * t;
    let c = c1 + (c2 - c1) * t;
    
    // Interpolate hue along shortest path
    let h = lerp_hue(h1, h2, t, 0.0);
    
    let (lab_l, lab_a, lab_b) = oklch_to_oklab(l, c, h);
    soft_gamut_map_oklab(lab_l, lab_a, lab_b)
}

// ============================================================================
// Luminosity-Based Hue Shifting Functions
// Shifts dark colors toward cold (blue ~240°) and light colors toward warm (orange ~30°)
// Also adjusts saturation based on a curve
// ============================================================================

// Cold and warm target hues (in degrees)
const COLD_HUE: f32 = 240.0; // Blue
const WARM_HUE: f32 = 30.0;  // Orange

// Calculate the shortest path between two hues (handles wraparound)
fn hue_shift_amount(current_hue: f32, target_hue: f32, strength: f32) -> f32 {
    let mut diff = target_hue - current_hue;
    
    // Find shortest path around the color wheel
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    
    diff * strength
}

// Luminosity-based hue shift using OkLCh (perceptually uniform)
// - strength: 0.0 = no shift, 1.0 = full shift to cold/warm
// - shift_curve: controls where the shift is strongest based on luminosity
// - Dark colors (low L) shift toward COLD_HUE (blue)
// - Light colors (high L) shift toward WARM_HUE (orange)
// - sat_curve: controls how saturation is added based on luminosity
// - sat_strength: how much saturation to add (0.0 = none, 1.0 = max)
pub fn hue_shift_oklch(col: Color32, strength: f32, shift_curve: HueShiftCurve, sat_curve: SaturationCurve, sat_strength: f32) -> Color32 {
    let (l, a, b) = rgb_to_oklab(col);
    let (l_val, c, h) = oklab_to_oklch(l, a, b);
    
    // Smooth blend between cold and warm based on luminosity
    // Instead of abrupt switch at 0.5, use smooth interpolation
    // dark_weight: 1.0 at L=0, 0.0 at L=1
    let warm_weight = l_val; // How much to shift toward warm
    let cold_weight = 1.0 - l_val; // How much to shift toward cold
    
    // Calculate shift amount toward each target, weighted by luminosity
    let cold_shift = hue_shift_amount(h, COLD_HUE, 1.0) * cold_weight;
    let warm_shift = hue_shift_amount(h, WARM_HUE, 1.0) * warm_weight;
    
    // Get shift strength multiplier from curve
    let curve_multiplier = shift_curve.evaluate(l_val);
    let effective_strength = strength * curve_multiplier;
    
    // Combined shift: blend of cold and warm shifts
    let total_shift = (cold_shift + warm_shift) * effective_strength;
    let new_h = (h + total_shift + 360.0) % 360.0;
    
    // Get saturation to ADD from the curve (already scaled by sat_strength)
    let sat_to_add = sat_curve.evaluate(l_val, sat_strength);
    
    // Add saturation to existing chroma with soft clamping
    // Use adaptive max based on luminosity (extremes have less chroma headroom)
    let max_chroma = max_chroma_for_luminosity(l_val);
    let new_c = soft_clamp(c + sat_to_add, max_chroma);
    
    let (new_l, new_a, new_b) = oklch_to_oklab(l_val, new_c, new_h);
    soft_gamut_map_oklab(new_l, new_a, new_b)
}

// Convert RGB to HSV
fn rgb_to_hsv(col: Color32) -> (f32, f32, f32) {
    let r = col.r() as f32 / 255.0;
    let g = col.g() as f32 / 255.0;
    let b = col.b() as f32 / 255.0;
    
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    
    // Hue calculation
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    
    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };
    
    // Value
    let v = max;
    
    (if h < 0.0 { h + 360.0 } else { h }, s, v)
}

// Convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

// Get luminosity from RGB using standard formula
fn rgb_luminosity(col: Color32) -> f32 {
    let r = col.r() as f32 / 255.0;
    let g = col.g() as f32 / 255.0;
    let b = col.b() as f32 / 255.0;
    0.299 * r + 0.587 * g + 0.114 * b
}

// Luminosity-based hue shift using HSV
// - strength: 0.0 = no shift, 1.0 = full shift to cold/warm
// - shift_curve: controls where the shift is strongest based on luminosity
// - sat_curve: controls how saturation is added based on luminosity
// - sat_strength: how much saturation to add
pub fn hue_shift_hsv(col: Color32, strength: f32, shift_curve: HueShiftCurve, sat_curve: SaturationCurve, sat_strength: f32) -> Color32 {
    let (h, s, v) = rgb_to_hsv(col);
    let luminosity = rgb_luminosity(col);
    
    // Smooth blend between cold and warm based on luminosity
    // Instead of abrupt switch at 0.5, use smooth interpolation
    let warm_weight = luminosity; // How much to shift toward warm
    let cold_weight = 1.0 - luminosity; // How much to shift toward cold
    
    // Calculate shift amount toward each target, weighted by luminosity
    let cold_shift = hue_shift_amount(h, COLD_HUE, 1.0) * cold_weight;
    let warm_shift = hue_shift_amount(h, WARM_HUE, 1.0) * warm_weight;
    
    let curve_multiplier = shift_curve.evaluate(luminosity);
    let effective_strength = strength * curve_multiplier;
    
    // Combined shift: blend of cold and warm shifts
    let total_shift = (cold_shift + warm_shift) * effective_strength;
    let new_h = (h + total_shift + 360.0) % 360.0;
    
    // Get saturation to ADD from the curve (scaled for HSV's 0-1 range)
    // The curve returns OkLCh chroma units (~0-0.15), scale appropriately for HSV (0-1)
    let sat_to_add = sat_curve.evaluate(luminosity, sat_strength) * 4.0;
    
    // Soft clamp saturation to avoid harsh clipping
    let new_s = soft_clamp(s + sat_to_add, 1.0);
    
    hsv_to_rgb(new_h, new_s, v)
}

// Apply no hue shift (passthrough function for consistency)
pub fn hue_shift_none(col: Color32, _strength: f32, _shift_curve: HueShiftCurve, _sat_curve: SaturationCurve, _sat_strength: f32) -> Color32 {
    col
}