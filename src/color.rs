use egui_macroquad::egui::Color32;

// =============================================================================
// Color Space Enum - selectable at runtime
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorSpace {
    #[default]
    Rgb,
    OkLab,
    OkLCh,
}

impl ColorSpace {
    pub const ALL: &'static [ColorSpace] = &[ColorSpace::Rgb, ColorSpace::OkLab, ColorSpace::OkLCh];
    
    pub fn name(&self) -> &'static str {
        match self {
            ColorSpace::Rgb => "RGB",
            ColorSpace::OkLab => "OkLab",
            ColorSpace::OkLCh => "OkLCh",
        }
    }
}

// =============================================================================
// Public Interpolation API
// =============================================================================

/// Interpolate between two colors in the specified color space.
/// `t` is in [0.0, 1.0], where t=0 returns c1 and t=1 returns c2.
pub fn lerp_color(c1: Color32, c2: Color32, t: f32, space: ColorSpace) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    
    match space {
        ColorSpace::Rgb => lerp_rgb(c1, c2, t),
        ColorSpace::OkLab => lerp_oklab(c1, c2, t),
        ColorSpace::OkLCh => lerp_oklch(c1, c2, t),
    }
}

/// Extrapolate a color from a single reference point.
/// `direction` indicates how far and in which direction to shift:
/// - direction < 0: shift toward black/darker
/// - direction > 0: shift toward white/lighter
/// The magnitude determines how much to shift.
pub fn extrapolate_color(reference: Color32, direction: f32, space: ColorSpace) -> Color32 {
    match space {
        ColorSpace::Rgb => extrapolate_rgb(reference, direction),
        ColorSpace::OkLab => extrapolate_oklab(reference, direction),
        ColorSpace::OkLCh => extrapolate_oklch(reference, direction),
    }
}

// =============================================================================
// RGB Interpolation
// =============================================================================

fn lerp_rgb(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let r = lerp_f32(c1.r() as f32, c2.r() as f32, t);
    let g = lerp_f32(c1.g() as f32, c2.g() as f32, t);
    let b = lerp_f32(c1.b() as f32, c2.b() as f32, t);
    let a = lerp_f32(c1.a() as f32, c2.a() as f32, t);
    
    Color32::from_rgba_unmultiplied(
        r.round() as u8,
        g.round() as u8,
        b.round() as u8,
        a.round() as u8,
    )
}

fn extrapolate_rgb(reference: Color32, direction: f32) -> Color32 {
    // For RGB, we shift toward black (direction < 0) or white (direction > 0)
    let target = if direction < 0.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    };
    
    // Use absolute value as interpolation factor, but can go beyond 1.0
    let t = direction.abs();
    
    let r = lerp_f32(reference.r() as f32, target.r() as f32, t).clamp(0.0, 255.0);
    let g = lerp_f32(reference.g() as f32, target.g() as f32, t).clamp(0.0, 255.0);
    let b = lerp_f32(reference.b() as f32, target.b() as f32, t).clamp(0.0, 255.0);
    
    Color32::from_rgb(r.round() as u8, g.round() as u8, b.round() as u8)
}

// =============================================================================
// OkLab Interpolation
// =============================================================================

fn lerp_oklab(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let (l1, a1, b1) = rgb_to_oklab(c1);
    let (l2, a2, b2) = rgb_to_oklab(c2);
    
    let l = lerp_f32(l1, l2, t);
    let a = lerp_f32(a1, a2, t);
    let b = lerp_f32(b1, b2, t);
    
    oklab_to_rgb(l, a, b)
}

fn extrapolate_oklab(reference: Color32, direction: f32) -> Color32 {
    let (l, a, b) = rgb_to_oklab(reference);
    
    // Shift lightness based on direction, keep a and b (chromatic components)
    let new_l = if direction < 0.0 {
        // Toward black: reduce lightness
        (l - direction.abs()).clamp(0.0, 1.0)
    } else {
        // Toward white: increase lightness
        (l + direction).clamp(0.0, 1.0)
    };
    
    oklab_to_rgb(new_l, a, b)
}

// =============================================================================
// OkLCh Interpolation (perceptually uniform with hue interpolation)
// =============================================================================

fn lerp_oklch(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let (l1, c1_chroma, h1) = rgb_to_oklch(c1);
    let (l2, c2_chroma, h2) = rgb_to_oklch(c2);
    
    let l = lerp_f32(l1, l2, t);
    let c = lerp_f32(c1_chroma, c2_chroma, t);
    
    // Interpolate hue on the shortest path around the circle
    let h = lerp_hue(h1, h2, t);
    
    oklch_to_rgb(l, c, h)
}

fn extrapolate_oklch(reference: Color32, direction: f32) -> Color32 {
    let (l, c, h) = rgb_to_oklch(reference);
    
    // Shift lightness, preserve chroma and hue
    let new_l = if direction < 0.0 {
        (l - direction.abs()).clamp(0.0, 1.0)
    } else {
        (l + direction).clamp(0.0, 1.0)
    };
    
    oklch_to_rgb(new_l, c, h)
}

/// Interpolate hue angles, taking the shortest path around the circle
fn lerp_hue(h1: f32, h2: f32, t: f32) -> f32 {
    let mut delta = h2 - h1;
    
    // Normalize delta to [-180, 180]
    while delta > 180.0 {
        delta -= 360.0;
    }
    while delta < -180.0 {
        delta += 360.0;
    }
    
    let mut result = h1 + delta * t;
    
    // Normalize result to [0, 360)
    while result < 0.0 {
        result += 360.0;
    }
    while result >= 360.0 {
        result -= 360.0;
    }
    
    result
}

// =============================================================================
// sRGB <-> Linear RGB conversion
// =============================================================================

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

// =============================================================================
// RGB <-> OkLab conversion
// =============================================================================

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

pub fn oklab_to_rgb(lab_l: f32, lab_a: f32, lab_b: f32) -> Color32 {
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

// =============================================================================
// OkLab <-> OkLCh conversion
// =============================================================================

/// Convert OkLab to OkLCh (cylindrical coordinates)
fn oklab_to_oklch(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    let c = (a * a + b * b).sqrt(); // Chroma
    let h = b.atan2(a).to_degrees(); // Hue in degrees
    let h = if h < 0.0 { h + 360.0 } else { h }; // Normalize to [0, 360)
    (l, c, h)
}

/// Convert OkLCh back to OkLab
fn oklch_to_oklab(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    (l, a, b)
}

/// Convert RGB directly to OkLCh
pub fn rgb_to_oklch(col: Color32) -> (f32, f32, f32) {
    let (l, a, b) = rgb_to_oklab(col);
    oklab_to_oklch(l, a, b)
}

/// Convert OkLCh directly to RGB
pub fn oklch_to_rgb(l: f32, c: f32, h: f32) -> Color32 {
    let (lab_l, lab_a, lab_b) = oklch_to_oklab(l, c, h);
    oklab_to_rgb(lab_l, lab_a, lab_b)
}

// =============================================================================
// Utility functions
// =============================================================================

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

