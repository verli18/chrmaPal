/// A curve maps a normalized input `t` in [0.0, 1.0] to an output value.
/// This is the core abstraction for all interpolation in the palette system.
/// 
/// By making this a public trait, any interpolation logic (color blending,
/// parameter tweening, etc.) can be generic over the curve type.
pub trait Curve {
    fn sample(&self, t: f32) -> f32;
}

/// The simplest curve: output equals input (optionally scaled by a factor).
/// With factor=1.0, this is pure linear interpolation.
#[derive(Clone, Copy, Debug)]
pub struct Linear {
    pub factor: f32,
}

impl Default for Linear {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

impl Curve for Linear {
    fn sample(&self, t: f32) -> f32 {
        t * self.factor
    }
}

/// Ease-in curve: starts slow, accelerates toward the end.
/// Higher exponent = more dramatic easing.
#[derive(Clone, Copy, Debug)]
pub struct EaseIn {
    pub exponent: f32,
}

impl Default for EaseIn {
    fn default() -> Self {
        Self { exponent: 2.0 }
    }
}

impl Curve for EaseIn {
    fn sample(&self, t: f32) -> f32 {
        t.powf(self.exponent)
    }
}

/// Ease-out curve: starts fast, decelerates toward the end.
/// This is mathematically the "reflection" of EaseIn.
#[derive(Clone, Copy, Debug)]
pub struct EaseOut {
    pub exponent: f32,
}

impl Default for EaseOut {
    fn default() -> Self {
        Self { exponent: 2.0 }
    }
}

impl Curve for EaseOut {
    fn sample(&self, t: f32) -> f32 {
        // Flip input, apply ease-in, flip output
        1.0 - (1.0 - t).powf(self.exponent)
    }
}

/// Ease-in-out curve: slow at both ends, fast in the middle.
/// Creates a smooth S-curve transition.
#[derive(Clone, Copy, Debug)]
pub struct EaseInOut {
    pub exponent: f32,
}

impl Default for EaseInOut {
    fn default() -> Self {
        Self { exponent: 2.0 }
    }
}

impl Curve for EaseInOut {
    fn sample(&self, t: f32) -> f32 {
        if t < 0.5 {
            // First half: scaled ease-in
            0.5 * (2.0 * t).powf(self.exponent)
        } else {
            // Second half: scaled ease-out (mirrored)
            1.0 - 0.5 * (2.0 * (1.0 - t)).powf(self.exponent)
        }
    }
}

/// Cubic Bezier curve defined by 4 control points.
/// p0 and p3 are typically 0.0 and 1.0 for a standard 0→1 curve.
#[derive(Clone, Copy, Debug)]
pub struct Bezier {
    pub p0: f32,
    pub p1: f32,
    pub p2: f32,
    pub p3: f32,
}

impl Default for Bezier {
    fn default() -> Self {
        Self {
            p0: 0.0,
            p1: 0.33,
            p2: 0.66,
            p3: 1.0,
        }
    }
}

impl Curve for Bezier {
    fn sample(&self, t: f32) -> f32 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        uuu * self.p0 + 3.0 * uu * t * self.p1 + 3.0 * u * tt * self.p2 + ttt * self.p3
    }
}

// =============================================================================
// CurveType enum: Runtime-selectable curve variant
// =============================================================================
// 
// We use an enum here because the UI needs to switch between curve types at
// runtime. The enum implements `Curve` by delegating to the appropriate variant.
// This gives us the best of both worlds:
// - Static dispatch when the type is known
// - Runtime selection via the enum

/// All available curve types, selectable at runtime.
/// Each variant stores its own parameters.
#[derive(Clone, Copy, Debug)]
pub enum CurveType {
    Linear(Linear),
    EaseIn(EaseIn),
    EaseOut(EaseOut),
    EaseInOut(EaseInOut),
    Bezier(Bezier),
}

impl Default for CurveType {
    fn default() -> Self {
        CurveType::Linear(Linear::default())
    }
}

impl Curve for CurveType {
    fn sample(&self, t: f32) -> f32 {
        match self {
            CurveType::Linear(c) => c.sample(t),
            CurveType::EaseIn(c) => c.sample(t),
            CurveType::EaseOut(c) => c.sample(t),
            CurveType::EaseInOut(c) => c.sample(t),
            CurveType::Bezier(c) => c.sample(t),
        }
    }
}

/// Helper enum for UI: identifies which curve type is selected without parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveKind {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bezier,
}

impl CurveType {
    /// Returns the kind of this curve (for UI matching).
    pub fn kind(&self) -> CurveKind {
        match self {
            CurveType::Linear(_) => CurveKind::Linear,
            CurveType::EaseIn(_) => CurveKind::EaseIn,
            CurveType::EaseOut(_) => CurveKind::EaseOut,
            CurveType::EaseInOut(_) => CurveKind::EaseInOut,
            CurveType::Bezier(_) => CurveKind::Bezier,
        }
    }

    /// Creates a new CurveType of the given kind with default parameters.
    pub fn from_kind(kind: CurveKind) -> Self {
        match kind {
            CurveKind::Linear => CurveType::Linear(Linear::default()),
            CurveKind::EaseIn => CurveType::EaseIn(EaseIn::default()),
            CurveKind::EaseOut => CurveType::EaseOut(EaseOut::default()),
            CurveKind::EaseInOut => CurveType::EaseInOut(EaseInOut::default()),
            CurveKind::Bezier => CurveType::Bezier(Bezier::default()),
        }
    }
}