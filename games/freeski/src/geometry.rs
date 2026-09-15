//! Bounded native vector geometry shared by FreeSki artwork.
use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2};
const OUTLINE: Color32 = Color32::from_rgb(38, 57, 66);

pub(super) fn ellipse(
    p: &egui::Painter,
    origin: Pos2,
    scale: f32,
    width: f32,
    height: f32,
    color: Color32,
) {
    p.add(Shape::ellipse_filled(
        origin,
        Vec2::new(width, height) * scale,
        color,
    ));
}

pub(super) fn poly(
    p: &egui::Painter,
    origin: Pos2,
    scale: f32,
    points: &[(f32, f32)],
    color: Color32,
    outline: bool,
) {
    let positions: Vec<_> = points
        .iter()
        .map(|&(x, y)| origin + Vec2::new(x, y) * scale)
        .collect();
    // Ear clipping handles concave fur/limb contours without filling their gaps.
    let mut indices: Vec<usize> = (0..positions.len()).collect();
    let signed_area: f32 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(&(x, y), &(nx, ny))| x * ny - nx * y)
        .sum();
    let sign = signed_area.signum();
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let mut mesh = egui::Mesh::default();
    for &pos in &positions {
        mesh.colored_vertex(pos, color);
    }
    while indices.len() > 2 {
        let ear = (0..indices.len()).find(|&i| {
            let a = positions[indices[(i + indices.len() - 1) % indices.len()]];
            let b = positions[indices[i]];
            let c = positions[indices[(i + 1) % indices.len()]];
            cross(a, b, c) * sign > 0.0001
                && !indices.iter().enumerate().any(|(j, &v)| {
                    j != i
                        && j != (i + 1) % indices.len()
                        && j != (i + indices.len() - 1) % indices.len()
                        && cross(a, b, positions[v]) * sign >= 0.
                        && cross(b, c, positions[v]) * sign >= 0.
                        && cross(c, a, positions[v]) * sign >= 0.
                })
        });
        let Some(i) = ear else {
            break;
        };
        mesh.add_triangle(
            indices[(i + indices.len() - 1) % indices.len()] as u32,
            indices[i] as u32,
            indices[(i + 1) % indices.len()] as u32,
        );
        indices.remove(i);
    }
    p.add(Shape::mesh(mesh));
    if outline {
        p.add(Shape::closed_line(
            positions,
            Stroke::new((0.18 * scale).max(0.7), OUTLINE),
        ));
    }
}
