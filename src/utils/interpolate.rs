pub fn interpolate(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b *t
}

pub fn cosine_interpolate(a: f32, b: f32, t: f32) -> f32 {
    let t_clamped = t.clamp(0.0, 1.0);
    let t2 = (1.0 - (t_clamped * std::f32::consts::PI).cos()) * 0.5;
    a * (1.0 - t2) + b * t2
}

pub fn exp_interpolate(a: f32, b: f32, t: f32) -> f32 {
    let t_clamped = t.clamp(0.0, 1.0);
    let factor = (t_clamped * 2.0_f32.ln()).exp();
    a + (b - a) * factor
}

pub fn power_curve(x: f32, exponent: f32) -> f32 {
    x.powf(exponent)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a as f32 + (b as f32 - a as f32) * t
}

pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn smoothstep_mid(x: f32, n: f32) -> f32 {
    if x < 0.5 {
        0.5 * (2.0 * x).powf(n)
    } else {
        1.0 - 0.5 * (2.0 - 2.0 * x).powf(n)
    }
}