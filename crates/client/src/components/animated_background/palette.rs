#[cfg(target_arch = "wasm32")]
pub(super) fn palette(index: usize) -> &'static str {
    ["#9cdef2", "#e06c75", "#5fb6cc"][index % 3]
}

#[cfg(target_arch = "wasm32")]
pub(super) fn smooth_noise(x: f64, y: f64) -> f64 {
    let ix = x.floor();
    let iy = y.floor();
    let fx = x - ix;
    let fy = y - iy;
    let ux = fx * fx * (3.0 - 2.0 * fx);
    let uy = fy * fy * (3.0 - 2.0 * fy);
    let a = hash_noise(ix, iy);
    let b = hash_noise(ix + 1.0, iy);
    let c = hash_noise(ix, iy + 1.0);
    let d = hash_noise(ix + 1.0, iy + 1.0);
    a + (b - a) * ux + (c - a) * uy + (a - b - c + d) * ux * uy
}

#[cfg(target_arch = "wasm32")]
fn hash_noise(x: f64, y: f64) -> f64 {
    let value = (x * 12.9898 + y * 78.233).sin() * 43_758.545_3;
    value - value.floor()
}
