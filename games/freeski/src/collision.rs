use crate::engine::Point;
/// Closed overlap interval along a segment against a circle, including starts inside.
pub fn circle_interval(a: Point, b: Point, c: Point, radius: f64) -> Option<(f64, f64)> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let x = a.x - c.x;
    let y = a.y - c.y;
    let aa = dx * dx + dy * dy;
    let cc = x * x + y * y - radius * radius;
    if aa < 1e-20 {
        return (cc <= 0.).then_some((0., 1.));
    }
    let bb = 2. * (x * dx + y * dy);
    let disc = bb * bb - 4. * aa * cc;
    if disc < 0. {
        return None;
    }
    let root = disc.sqrt();
    let lo = ((-bb - root) / (2. * aa)).max(0.);
    let hi = ((-bb + root) / (2. * aa)).min(1.);
    (lo <= hi).then_some((lo, hi))
}
