//! Connector outline polygon points (same logic as schematic_view fill polygon).
//! Used by bounds for accurate connector bounding box.

use std::f64::consts::PI;

use crate::model::{ConnectorKind, ConnectorOutline, ConnectorSymbol};

/// Samples a Catmull-Rom spline through points, returning (x,y) in local coords.
fn catmull_rom_spline_points(
    points: &[(f64, f64)],
    segments_per_edge: usize,
) -> Vec<(f64, f64)> {
    if points.len() < 2 {
        return Vec::new();
    }
    let mut result = Vec::with_capacity((points.len() - 1) * segments_per_edge + 1);
    for i in 0..points.len().saturating_sub(1) {
        let p0 = if i > 0 { points[i - 1] } else { points[0] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() { points[i + 2] } else { p2 };
        let n = if i == 0 { segments_per_edge + 1 } else { segments_per_edge };
        for j in 0..=n {
            if i > 0 && j == 0 {
                continue;
            }
            let t = (j as f64) / (segments_per_edge as f64);
            let t2 = t * t;
            let t3 = t2 * t;
            let x = 0.5 * ((2.0 * p1.0)
                + (-p0.0 + p2.0) * t
                + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
            let y = 0.5 * ((2.0 * p1.1)
                + (-p0.1 + p2.1) * t
                + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
            result.push((x, y));
        }
    }
    result
}

fn apply_line_cuts_to_rect(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    lines: &[(f64, f64, f64, f64)],
) -> Vec<(f64, f64)> {
    const EPS: f64 = 1e-6;
    let (x2, y2) = (x + w, y + h);

    #[derive(Clone, Copy, PartialEq)]
    enum Edge {
        Top,
        Right,
        Bottom,
        Left,
    }
    fn on_edge(px: f64, py: f64, x: f64, y: f64, w: f64, h: f64) -> Option<Edge> {
        let (x2, y2) = (x + w, y + h);
        if (py - y).abs() < EPS && x - EPS <= px && px <= x2 + EPS {
            Some(Edge::Top)
        } else if (px - x2).abs() < EPS && y - EPS <= py && py <= y2 + EPS {
            Some(Edge::Right)
        } else if (py - y2).abs() < EPS && x - EPS <= px && px <= x2 + EPS {
            Some(Edge::Bottom)
        } else if (px - x).abs() < EPS && y - EPS <= py && py <= y2 + EPS {
            Some(Edge::Left)
        } else {
            None
        }
    }
    fn adjacent(e1: Edge, e2: Edge) -> bool {
        matches!(
            (e1, e2),
            (Edge::Top, Edge::Right)
                | (Edge::Right, Edge::Top)
                | (Edge::Right, Edge::Bottom)
                | (Edge::Bottom, Edge::Right)
                | (Edge::Bottom, Edge::Left)
                | (Edge::Left, Edge::Bottom)
                | (Edge::Left, Edge::Top)
                | (Edge::Top, Edge::Left)
        )
    }

    let mut pts = vec![(x, y), (x2, y), (x2, y2), (x, y2)];
    let corners: [(Edge, Edge); 4] = [
        (Edge::Top, Edge::Left),
        (Edge::Top, Edge::Right),
        (Edge::Right, Edge::Bottom),
        (Edge::Bottom, Edge::Left),
    ];

    for (x1, y1, x2l, y2l) in lines {
        let e1 = on_edge(*x1, *y1, x, y, w, h);
        let e2 = on_edge(*x2l, *y2l, x, y, w, h);
        let (Some(e1), Some(e2)) = (e1, e2) else {
            continue;
        };
        if !adjacent(e1, e2) {
            continue;
        }
        for (i, (c1, c2)) in corners.iter().enumerate() {
            if (e1 == *c1 && e2 == *c2) || (e1 == *c2 && e2 == *c1) {
                let (p1, p2) = ((*x1, *y1), (*x2l, *y2l));
                let (first, second) = match i {
                    0 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (y1 - y).abs() < (y2l - y).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (x1 - x).abs() < (x2l - x).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    1 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (x1 - x2).abs() < (x2l - x2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (y1 - y).abs() < (y2l - y).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    2 => {
                        if e1 == Edge::Right || e2 == Edge::Right {
                            if (y1 - y2).abs() < (y2l - y2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (x1 - x2).abs() < (x2l - x2).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    3 => {
                        if e1 == Edge::Bottom || e2 == Edge::Bottom {
                            if x1 > x2l {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if y1 > y2l {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    _ => (p1, p2),
                };
                pts.remove(i);
                pts.insert(i, second);
                pts.insert(i, first);
                break;
            }
        }
    }
    pts
}

fn rect_to_rounded_polygon_plug(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    lines: &[(f64, f64, f64, f64)],
    round_rad: f64,
) -> Vec<(f64, f64)> {
    const EPS: f64 = 1e-6;
    const SEGMENTS_PER_QUARTER: usize = 64;
    let (x2, y2) = (x + w, y + h);

    #[derive(Clone, Copy, PartialEq)]
    enum Edge {
        Top,
        Right,
        Bottom,
        Left,
    }
    fn on_edge(px: f64, py: f64, x: f64, y: f64, w: f64, h: f64) -> Option<Edge> {
        let (x2, y2) = (x + w, y + h);
        if (py - y).abs() < EPS && x - EPS <= px && px <= x2 + EPS {
            Some(Edge::Top)
        } else if (px - x2).abs() < EPS && y - EPS <= py && py <= y2 + EPS {
            Some(Edge::Right)
        } else if (py - y2).abs() < EPS && x - EPS <= px && px <= x2 + EPS {
            Some(Edge::Bottom)
        } else if (px - x).abs() < EPS && y - EPS <= py && py <= y2 + EPS {
            Some(Edge::Left)
        } else {
            None
        }
    }
    fn adjacent(e1: Edge, e2: Edge) -> bool {
        matches!(
            (e1, e2),
            (Edge::Top, Edge::Right)
                | (Edge::Right, Edge::Top)
                | (Edge::Right, Edge::Bottom)
                | (Edge::Bottom, Edge::Right)
                | (Edge::Bottom, Edge::Left)
                | (Edge::Left, Edge::Bottom)
                | (Edge::Left, Edge::Top)
        )
    }

    let corner_centers: [(f64, f64); 4] = [
        (x + round_rad, y + round_rad),
        (x2 - round_rad, y + round_rad),
        (x2 - round_rad, y2 - round_rad),
        (x + round_rad, y2 - round_rad),
    ];
    let arc_ranges: [(f64, f64); 4] = [
        (180.0, 90.0),
        (270.0, 90.0),
        (0.0, 90.0),
        (90.0, 90.0),
    ];

    let mut cut_corners = [None; 4];
    for (x1, y1, x2l, y2l) in lines {
        let e1 = on_edge(*x1, *y1, x, y, w, h);
        let e2 = on_edge(*x2l, *y2l, x, y, w, h);
        let (Some(e1), Some(e2)) = (e1, e2) else {
            continue;
        };
        if !adjacent(e1, e2) {
            continue;
        }
        for (i, (c1, c2)) in [
            (Edge::Top, Edge::Left),
            (Edge::Top, Edge::Right),
            (Edge::Right, Edge::Bottom),
            (Edge::Bottom, Edge::Left),
        ]
        .iter()
        .enumerate()
        {
            if (e1 == *c1 && e2 == *c2) || (e1 == *c2 && e2 == *c1) {
                let (p1, p2) = ((*x1, *y1), (*x2l, *y2l));
                let (first, second) = match i {
                    0 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (y1 - y).abs() < (y2l - y).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (x1 - x).abs() < (x2l - x).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    1 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (x1 - x2).abs() < (x2l - x2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (y1 - y).abs() < (y2l - y).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    2 => {
                        if e1 == Edge::Right || e2 == Edge::Right {
                            if (y1 - y2).abs() < (y2l - y2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if (x1 - x2).abs() < (x2l - x2).abs() {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    3 => {
                        if e1 == Edge::Bottom || e2 == Edge::Bottom {
                            if x1 > x2l {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else if y1 > y2l {
                            (p1, p2)
                        } else {
                            (p2, p1)
                        }
                    }
                    _ => (p1, p2),
                };
                cut_corners[i] = Some((first, second));
                break;
            }
        }
    }

    let mut pts = Vec::new();
    let rad = round_rad.min(w * 0.5).min(h * 0.5);

    for i in 0..4 {
        let (cx, cy) = corner_centers[i];
        if let Some((first, second)) = cut_corners[i] {
            let (corner_x, corner_y) = [(x, y), (x2, y), (x2, y2), (x, y2)][i];
            let (fx, fy) = first;
            let (sx, sy) = second;
            let r_cut = ((fx - corner_x).hypot(fy - corner_y))
                .min((sx - corner_x).hypot(sy - corner_y))
                .min(rad);
            let a1 = (fy - corner_y).atan2(fx - corner_x);
            let a2 = (sy - corner_y).atan2(sx - corner_x);
            let start = a1 * 180.0 / PI;
            let mut sweep = (a2 - a1) * 180.0 / PI;
            if sweep < 0.0 {
                sweep += 360.0;
            }
            if sweep > 180.0 {
                sweep -= 360.0;
            }
            let n = (SEGMENTS_PER_QUARTER as f64 * sweep.abs() / 90.0).ceil().max(2.0) as usize;
            for j in 1..n {
                let t = (j as f64) / (n as f64);
                let angle_deg = start + t * sweep;
                let angle_rad = angle_deg * PI / 180.0;
                let px = corner_x + r_cut * angle_rad.cos();
                let py = corner_y + r_cut * angle_rad.sin();
                pts.push((px, py));
            }
        } else {
            let (start_deg, sweep_deg) = arc_ranges[i];
            for j in 0..=SEGMENTS_PER_QUARTER {
                let t = (j as f64) / (SEGMENTS_PER_QUARTER as f64);
                let angle_deg = start_deg + t * sweep_deg;
                let angle_rad = angle_deg * PI / 180.0;
                let px = cx + rad * angle_rad.cos();
                let py = cy + rad * angle_rad.sin();
                pts.push((px, py));
            }
        }
    }
    pts
}

/// Returns ALL points from ALL outline primitives for full bounding box.
/// Use this for connector bounds — includes fill shape plus circles (pin holes), lines, etc.
pub fn connector_bounds_points_local(conn: &ConnectorSymbol) -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = Vec::new();

    if conn.outline.is_empty() {
        points.push((conn.extent_x, conn.extent_y));
        points.push((conn.extent_x + conn.width, conn.extent_y));
        points.push((conn.extent_x + conn.width, conn.extent_y + conn.height));
        points.push((conn.extent_x, conn.extent_y + conn.height));
        return points;
    }

    let lines: Vec<(f64, f64, f64, f64)> = conn
        .outline
        .iter()
        .filter_map(|p| match p {
            ConnectorOutline::Line { x1, y1, x2, y2 } => Some((*x1, *y1, *x2, *y2)),
            _ => None,
        })
        .collect();
    let has_lines = !lines.is_empty();

    for prim in &conn.outline {
        add_primitive_bounds_points(&mut points, prim, conn.kind, has_lines, &lines);
    }

    points
}

fn add_primitive_bounds_points(
    points: &mut Vec<(f64, f64)>,
    prim: &ConnectorOutline,
    kind: ConnectorKind,
    has_lines: bool,
    lines: &[(f64, f64, f64, f64)],
) {
    match prim {
        ConnectorOutline::Line { x1, y1, x2, y2 } => {
            points.push((*x1, *y1));
            points.push((*x2, *y2));
        }
        ConnectorOutline::Polyline { points: pts } | ConnectorOutline::Polygon { points: pts } => {
            points.extend(pts.iter().copied());
        }
        ConnectorOutline::Arc {
            x: cx,
            y: cy,
            radius,
            start_angle,
            travel_angle,
        } => {
            if radius.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                let n = ((travel_angle.abs() / 3.0).ceil() as usize).max(16);
                for i in 0..=n {
                    let t = (i as f64) / (n as f64);
                    let angle_deg = start_angle + t * travel_angle;
                    let angle_rad = angle_deg * PI / 180.0;
                    points.push((cx + radius * angle_rad.cos(), cy + radius * angle_rad.sin()));
                }
            }
        }
        ConnectorOutline::WidthArc {
            x: cx,
            y: cy,
            radius,
            start_angle,
            travel_angle,
            width,
        } => {
            let r_inner = *radius;
            let r_outer = radius + width;
            if r_outer.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                let n = ((travel_angle.abs() / 3.0).ceil() as usize).max(16);
                for i in 0..=n {
                    let t = (i as f64) / (n as f64);
                    let angle_deg = start_angle + t * travel_angle;
                    let angle_rad = angle_deg * PI / 180.0;
                    points.push((cx + r_outer * angle_rad.cos(), cy + r_outer * angle_rad.sin()));
                    points.push((cx + r_inner * angle_rad.cos(), cy + r_inner * angle_rad.sin()));
                }
            }
        }
        ConnectorOutline::Rectangle { x, y, width, height } => {
            let rect_pts = if matches!(kind, ConnectorKind::Plug) {
                let rad = (width.min(*height) * 0.15).max(48.0);
                rect_to_rounded_polygon_plug(*x, *y, *width, *height, lines, rad)
            } else {
                let mut rect_pts = vec![
                    (*x, *y),
                    (x + width, *y),
                    (x + width, y + height),
                    (*x, y + height),
                ];
                if has_lines {
                    rect_pts = apply_line_cuts_to_rect(*x, *y, *width, *height, lines);
                }
                rect_pts
            };
            points.extend(rect_pts);
        }
        ConnectorOutline::Circle { x, y, radius } => {
            if radius.abs() >= 1e-9 {
                points.push((x - radius, y - radius));
                points.push((x + radius, y + radius));
                let n = 96usize;
                for i in 0..n {
                    let angle_rad = (i as f64 / n as f64) * 2.0 * PI;
                    points.push((x + radius * angle_rad.cos(), y + radius * angle_rad.sin()));
                }
            }
        }
        ConnectorOutline::Curve { points: pts } => {
            if pts.len() >= 2 {
                points.extend(catmull_rom_spline_points(pts, 8));
            }
        }
    }
}

/// Returns the same polygon points used for connector fill/background, in connector-local coords.
/// Mirrors connector_fill_polygon logic from schematic_view.
pub fn connector_fill_points_local(conn: &ConnectorSymbol) -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = Vec::new();
    let push_if_new = |pts: &mut Vec<(f64, f64)>, x: f64, y: f64| {
        let p = (x, y);
        if pts.last().map_or(true, |q| (q.0 - x).hypot(q.1 - y) > 0.01) {
            pts.push(p);
        }
    };

    if conn.outline.is_empty() {
        points.push((conn.extent_x, conn.extent_y));
        points.push((conn.extent_x + conn.width, conn.extent_y));
        points.push((conn.extent_x + conn.width, conn.extent_y + conn.height));
        points.push((conn.extent_x, conn.extent_y + conn.height));
        return points;
    }

    let has_rect = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Rectangle { .. }));
    let has_polygon = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Polygon { .. }));
    let has_polyline = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Polyline { .. }));
    let has_circle = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Circle { .. }));
    let has_lines = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Line { .. }));
    let has_arc = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Arc { .. }));
    let has_width_arc = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::WidthArc { .. }));

    let use_outline_path = has_arc || has_width_arc;

    if use_outline_path {
        let mut ordered: Vec<&ConnectorOutline> = conn.outline.iter().collect();
        let arc_y = |p: &ConnectorOutline| match p {
            ConnectorOutline::Arc { y, .. } | ConnectorOutline::WidthArc { y, .. } => Some(*y),
            _ => None,
        };
        let arc_ys: Vec<f64> = ordered.iter().filter_map(|p| arc_y(p)).collect();
        let (min_arc_y, max_arc_y) = arc_ys
            .iter()
            .fold((f64::MAX, f64::MIN), |(mn, mx), &y| (mn.min(y), mx.max(y)));
        ordered.sort_by(|a, b| {
            let key = |p: &ConnectorOutline| match p {
                ConnectorOutline::Polyline { .. } | ConnectorOutline::Polygon { .. } => (0u8, 0.0),
                ConnectorOutline::Arc { y, .. } | ConnectorOutline::WidthArc { y, .. } => {
                    let ord = if (*y - min_arc_y).abs() < (*y - max_arc_y).abs() {
                        1
                    } else {
                        3
                    };
                    (ord, *y)
                }
                ConnectorOutline::Line { .. } => (2, 0.0),
                ConnectorOutline::Rectangle { .. } => (4, 0.0),
                ConnectorOutline::Circle { .. } => (5, 0.0),
                ConnectorOutline::Curve { .. } => (6, 0.0),
            };
            let (ka, ya) = key(a);
            let (kb, yb) = key(b);
            ka.cmp(&kb).then_with(|| ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal))
        });
        for prim in ordered {
            match prim {
                ConnectorOutline::Line { x1, y1, x2, y2 } => {
                    push_if_new(&mut points, *x1, *y1);
                    push_if_new(&mut points, *x2, *y2);
                }
                ConnectorOutline::Polyline { points: pts } => {
                    for (px, py) in pts {
                        push_if_new(&mut points, *px, *py);
                    }
                }
                ConnectorOutline::Arc {
                    x: cx,
                    y: cy,
                    radius,
                    start_angle,
                    travel_angle,
                } => {
                    if radius.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                        let n = ((travel_angle.abs() / 3.0).ceil() as usize).max(16);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + radius * angle_rad.cos();
                            let py = cy + radius * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                    }
                }
                ConnectorOutline::WidthArc {
                    x: cx,
                    y: cy,
                    radius,
                    start_angle,
                    travel_angle,
                    width,
                } => {
                    let r_inner = *radius;
                    let r_outer = radius + width;
                    if r_outer.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                        let n = ((travel_angle.abs() / 3.0).ceil() as usize).max(16);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_outer * angle_rad.cos();
                            let py = cy + r_outer * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                        for i in (0..=n).rev() {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_inner * angle_rad.cos();
                            let py = cy + r_inner * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                    }
                }
                ConnectorOutline::Rectangle { x, y, width, height } => {
                    push_if_new(&mut points, *x, *y);
                    push_if_new(&mut points, x + width, *y);
                    push_if_new(&mut points, x + width, y + height);
                    push_if_new(&mut points, *x, y + height);
                }
                ConnectorOutline::Polygon { points: pts } => {
                    for (px, py) in pts {
                        push_if_new(&mut points, *px, *py);
                    }
                }
                ConnectorOutline::Circle { x, y, radius } => {
                    if radius.abs() >= 1e-9 {
                        let n = 96usize;
                        for i in 0..n {
                            let angle_rad = (i as f64 / n as f64) * 2.0 * PI;
                            let px = x + radius * angle_rad.cos();
                            let py = y + radius * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                    }
                }
                ConnectorOutline::Curve { points: pts } => {
                    if pts.len() >= 2 {
                        let sampled = catmull_rom_spline_points(pts, 8);
                        for (px, py) in sampled {
                            push_if_new(&mut points, px, py);
                        }
                    }
                }
            }
        }
    } else {
        let fill_prim = if has_rect {
            conn.outline.iter().find(|p| matches!(p, ConnectorOutline::Rectangle { .. }))
        } else if has_polygon {
            conn.outline.iter().find(|p| matches!(p, ConnectorOutline::Polygon { .. }))
        } else if has_polyline {
            conn.outline.iter().find(|p| matches!(p, ConnectorOutline::Polyline { .. }))
        } else if has_circle && !has_rect {
            conn.outline.iter().find(|p| matches!(p, ConnectorOutline::Circle { .. }))
        } else {
            None
        };

        if let Some(prim) = fill_prim {
            match prim {
                ConnectorOutline::Polygon { points: pts } => {
                    for (px, py) in pts {
                        push_if_new(&mut points, *px, *py);
                    }
                }
                ConnectorOutline::Rectangle { x, y, width, height } => {
                    let rect_pts = if matches!(conn.kind, ConnectorKind::Plug) {
                        let lines: Vec<_> = conn
                            .outline
                            .iter()
                            .filter_map(|p| match p {
                                ConnectorOutline::Line { x1, y1, x2, y2 } => Some((*x1, *y1, *x2, *y2)),
                                _ => None,
                            })
                            .collect();
                        let rad = (width.min(*height) * 0.15).max(48.0);
                        rect_to_rounded_polygon_plug(*x, *y, *width, *height, &lines, rad)
                    } else {
                        let mut rect_pts = vec![
                            (*x, *y),
                            (x + width, *y),
                            (x + width, y + height),
                            (*x, y + height),
                        ];
                        if has_lines {
                            let lines: Vec<_> = conn
                                .outline
                                .iter()
                                .filter_map(|p| match p {
                                    ConnectorOutline::Line { x1, y1, x2, y2 } => {
                                        Some((*x1, *y1, *x2, *y2))
                                    }
                                    _ => None,
                                })
                                .collect();
                            rect_pts = apply_line_cuts_to_rect(*x, *y, *width, *height, &lines);
                        }
                        rect_pts
                    };
                    for (px, py) in rect_pts {
                        push_if_new(&mut points, px, py);
                    }
                }
                ConnectorOutline::Polyline { points: pts } => {
                    for (px, py) in pts {
                        push_if_new(&mut points, *px, *py);
                    }
                }
                ConnectorOutline::Circle { x, y, radius } => {
                    if radius.abs() >= 1e-9 {
                        let n = 96usize;
                        for i in 0..n {
                            let angle_rad = (i as f64 / n as f64) * 2.0 * PI;
                            let px = x + radius * angle_rad.cos();
                            let py = y + radius * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                    }
                }
                _ => {}
            }
        } else {
            push_if_new(&mut points, conn.extent_x, conn.extent_y);
            push_if_new(&mut points, conn.extent_x + conn.width, conn.extent_y);
            push_if_new(&mut points, conn.extent_x + conn.width, conn.extent_y + conn.height);
            push_if_new(&mut points, conn.extent_x, conn.extent_y + conn.height);
        }
    }

    points
}
