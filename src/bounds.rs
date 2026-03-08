//! Precomputed bounding rects for diagram elements. Used for grid extent and hit tests.
//
// Copyright (C) 2025 microcapital contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::f64::consts::PI;

use crate::model::{
    ConnectorOutline, ConnectorSymbol, DeviceSymbol, DiagramContent, Schemindicator, SpliceSymbol,
    WireSegment,
};

/// Returns true if the given angle (in degrees, any value) lies within the arc from start_angle
/// to start_angle + travel_angle. Handles 360° wrap.
fn angle_in_arc(angle: f64, start_angle: f64, travel_angle: f64) -> bool {
    let a = ((angle % 360.0) + 360.0) % 360.0;
    let start_n = ((start_angle % 360.0) + 360.0) % 360.0;
    let end_n = ((start_angle + travel_angle) % 360.0 + 360.0) % 360.0;
    if travel_angle.abs() >= 360.0 {
        return true;
    }
    if (start_n - end_n).abs() < 1e-9 {
        return true; // full circle
    }
    if start_n <= end_n {
        a >= start_n - 1e-9 && a <= end_n + 1e-9
    } else {
        a >= start_n - 1e-9 || a <= end_n + 1e-9
    }
}

/// Add axis crossing points (0°, 90°, 180°, 270°) for an arc when they fall within the arc range.
/// These are the extrema of a circle and ensure the bounding box fully contains the arc.
fn add_arc_axis_crossings(
    points: &mut Vec<(f64, f64)>,
    x: f64,
    y: f64,
    radius: f64,
    start_angle: f64,
    travel_angle: f64,
) {
    if radius.abs() < 1e-9 {
        return;
    }
    let r = radius.abs();
    let axis_angles = [0.0, 90.0, 180.0, 270.0];
    for &axis in &axis_angles {
        if angle_in_arc(axis, start_angle, travel_angle) {
            let rad = axis * PI / 180.0;
            points.push((x + r * rad.cos(), y + r * rad.sin()));
        }
    }
}

/// Collects all points from outline geometry that affect the bounding box (in local coords).
fn outline_bounding_points(outline: &[ConnectorOutline]) -> Vec<(f64, f64)> {
    let mut points: Vec<(f64, f64)> = Vec::new();
    for prim in outline {
        match prim {
            ConnectorOutline::Line { x1, y1, x2, y2 } => {
                points.push((*x1, *y1));
                points.push((*x2, *y2));
            }
            ConnectorOutline::Rectangle { x, y, width, height } => {
                points.push((*x, *y));
                points.push((x + width, *y));
                points.push((x + width, y + height));
                points.push((*x, y + height));
            }
            ConnectorOutline::Circle { x, y, radius } => {
                if radius.abs() >= 1e-9 {
                    points.push((x - radius, y - radius));
                    points.push((x + radius, y + radius));
                }
            }
            ConnectorOutline::Arc {
                x,
                y,
                radius,
                start_angle,
                travel_angle,
            } => {
                if radius.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                    // Dense sampling (every ~10°) to capture full arc shape
                    let n = ((travel_angle.abs() / 10.0).ceil() as usize).max(8);
                    for i in 0..=n {
                        let t = (i as f64) / (n as f64);
                        let angle_deg = start_angle + t * travel_angle;
                        let angle_rad = angle_deg * PI / 180.0;
                        points.push((x + radius * angle_rad.cos(), y + radius * angle_rad.sin()));
                    }
                    add_arc_axis_crossings(&mut points, *x, *y, *radius, *start_angle, *travel_angle);
                }
            }
            ConnectorOutline::WidthArc {
                x,
                y,
                radius,
                start_angle,
                travel_angle,
                width,
            } => {
                let r_outer = radius + width;
                if r_outer.abs() >= 1e-9 && travel_angle.abs() >= 1e-9 {
                    // Dense sampling for both inner and outer arcs
                    let n = ((travel_angle.abs() / 10.0).ceil() as usize).max(8);
                    for i in 0..=n {
                        let t = (i as f64) / (n as f64);
                        let angle_deg = start_angle + t * travel_angle;
                        let angle_rad = angle_deg * PI / 180.0;
                        points.push((x + r_outer * angle_rad.cos(), y + r_outer * angle_rad.sin()));
                        points.push((x + radius * angle_rad.cos(), y + radius * angle_rad.sin()));
                    }
                    add_arc_axis_crossings(&mut points, *x, *y, *radius, *start_angle, *travel_angle);
                    add_arc_axis_crossings(&mut points, *x, *y, r_outer, *start_angle, *travel_angle);
                }
            }
            ConnectorOutline::Polyline { points: pts } | ConnectorOutline::Polygon { points: pts } => {
                points.extend(pts.iter().copied());
            }
            ConnectorOutline::Curve { points: pts } => {
                points.extend(pts.iter().copied());
            }
        }
    }
    points
}

/// Compute bounding rect from points; returns (min_x, min_y, max_x, max_y).
fn points_to_bounds(points: &[(f64, f64)]) -> Option<(f64, f64, f64, f64)> {
    if points.is_empty() {
        return None;
    }
    let (min_x, max_x) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (x, _)| {
            (mn.min(*x), mx.max(*x))
        });
    let (min_y, max_y) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (_, y)| {
            (mn.min(*y), mx.max(*y))
        });
    Some((min_x, min_y, max_x, max_y))
}

fn transform_pt(x: f64, y: f64, ox: f64, oy: f64, m: &[f64; 6]) -> (f64, f64) {
    (
        ox + m[0] * x + m[2] * y + m[4],
        oy + m[1] * x + m[3] * y + m[5],
    )
}

/// Padding in logical units for pin graphic (diamond) extent. Do not include name/attribute text.
const PIN_GRAPHIC_PADDING: f64 = 500.0;

/// Transform symbol-local point to global diagram coords.
fn to_global(x: f64, y: f64, ox: f64, oy: f64, m: &[f64; 6]) -> (f64, f64) {
    transform_pt(x, y, ox, oy, m)
}

/// Compute device bounds in global logical coords. Includes extent rect, outline, pins, and all
/// symbol graphics (lines, arcs, circles). Excludes name text and attribute text.
/// Transforms each point to global before computing AABB for correct bounds with rotation/scale.
pub fn compute_device_bounds(dev: &DeviceSymbol) -> (f64, f64, f64, f64) {
    let m = &dev.transform;
    let mut global_points: Vec<(f64, f64)> = Vec::new();
    let to_global_pt = |x: f64, y: f64| to_global(x, y, dev.x, dev.y, m);

    // Extent rect corners
    global_points.push(to_global_pt(dev.extent_x, dev.extent_y));
    global_points.push(to_global_pt(dev.extent_x + dev.width, dev.extent_y));
    global_points.push(to_global_pt(dev.extent_x + dev.width, dev.extent_y + dev.height));
    global_points.push(to_global_pt(dev.extent_x, dev.extent_y + dev.height));

    // Symbol graphics (lines, arcs, circles, rectangles, polylines, etc.) — each point transformed
    for (lx, ly) in outline_bounding_points(&dev.outline) {
        global_points.push(to_global_pt(lx, ly));
    }

    // All device pins including graphic extent (diamond)
    for pin in &dev.pins {
        global_points.push(to_global_pt(pin.x, pin.y));
        global_points.push(to_global_pt(pin.x - PIN_GRAPHIC_PADDING, pin.y));
        global_points.push(to_global_pt(pin.x + PIN_GRAPHIC_PADDING, pin.y));
        global_points.push(to_global_pt(pin.x, pin.y - PIN_GRAPHIC_PADDING));
        global_points.push(to_global_pt(pin.x, pin.y + PIN_GRAPHIC_PADDING));
    }

    points_to_bounds(&global_points).unwrap_or_else(|| {
        let (g0, g1, g2, g3) = (
            to_global_pt(dev.extent_x, dev.extent_y),
            to_global_pt(dev.extent_x + dev.width, dev.extent_y),
            to_global_pt(dev.extent_x + dev.width, dev.extent_y + dev.height),
            to_global_pt(dev.extent_x, dev.extent_y + dev.height),
        );
        let min_x = g0.0.min(g1.0).min(g2.0).min(g3.0);
        let max_x = g0.0.max(g1.0).max(g2.0).max(g3.0);
        let min_y = g0.1.min(g1.1).min(g2.1).min(g3.1);
        let max_y = g0.1.max(g1.1).max(g2.1).max(g3.1);
        (min_x, min_y, max_x, max_y)
    })
}

/// Compute connector bounds in global logical coords. Uses the same points as the fill polygon
/// (connector_fill_points_local) so bounds match the visible connector shape exactly.
/// Also includes pins. Excludes name text and attribute text.
pub fn compute_connector_bounds(conn: &ConnectorSymbol) -> (f64, f64, f64, f64) {
    let m = &conn.transform;
    let mut global_points: Vec<(f64, f64)> = Vec::new();
    let to_global_pt = |x: f64, y: f64| to_global(x, y, conn.x, conn.y, m);

    // All outline primitives (fill + circles, lines, etc.) — full bounds
    for (lx, ly) in crate::outline::connector_bounds_points_local(conn) {
        global_points.push(to_global_pt(lx, ly));
    }

    // All connector pins including graphic extent (diamond)
    for pin in &conn.pins {
        global_points.push(to_global_pt(pin.x, pin.y));
        global_points.push(to_global_pt(pin.x - PIN_GRAPHIC_PADDING, pin.y));
        global_points.push(to_global_pt(pin.x + PIN_GRAPHIC_PADDING, pin.y));
        global_points.push(to_global_pt(pin.x, pin.y - PIN_GRAPHIC_PADDING));
        global_points.push(to_global_pt(pin.x, pin.y + PIN_GRAPHIC_PADDING));
    }

    points_to_bounds(&global_points).unwrap_or_else(|| {
        let (g0, g1, g2, g3) = (
            to_global_pt(conn.extent_x, conn.extent_y),
            to_global_pt(conn.extent_x + conn.width, conn.extent_y),
            to_global_pt(conn.extent_x + conn.width, conn.extent_y + conn.height),
            to_global_pt(conn.extent_x, conn.extent_y + conn.height),
        );
        let min_x = g0.0.min(g1.0).min(g2.0).min(g3.0);
        let max_x = g0.0.max(g1.0).max(g2.0).max(g3.0);
        let min_y = g0.1.min(g1.1).min(g2.1).min(g3.1);
        let max_y = g0.1.max(g1.1).max(g2.1).max(g3.1);
        (min_x, min_y, max_x, max_y)
    })
}

/// Compute splice bounds. Splice is typically a circle at (0,0) with radius ~50.
pub fn compute_splice_bounds(sp: &SpliceSymbol) -> (f64, f64, f64, f64) {
    let m = &sp.transform;
    let mut points: Vec<(f64, f64)> = vec![(0.0, 0.0)];

    for prim in &sp.outline {
        match prim {
            ConnectorOutline::Circle { x, y, radius } => {
                if radius.abs() >= 1e-9 {
                    points.push((x - radius, y - radius));
                    points.push((x + radius, y + radius));
                }
            }
            _ => {
                points.extend(outline_bounding_points(&[prim.clone()]));
            }
        }
    }

    let local_bounds = points_to_bounds(&points).unwrap_or((-50.0, -50.0, 50.0, 50.0));
    let (min_x, min_y, max_x, max_y) = local_bounds;
    let corners = [
        transform_pt(min_x, min_y, sp.x, sp.y, m),
        transform_pt(max_x, min_y, sp.x, sp.y, m),
        transform_pt(max_x, max_y, sp.x, sp.y, m),
        transform_pt(min_x, max_y, sp.x, sp.y, m),
    ];
    let (gmin_x, gmax_x) = corners
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (x, _)| {
            (mn.min(*x), mx.max(*x))
        });
    let (gmin_y, gmax_y) = corners
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (_, y)| {
            (mn.min(*y), mx.max(*y))
        });
    (gmin_x, gmin_y, gmax_x, gmax_y)
}

/// Compute schemindicator bounds in global logical coords.
pub fn compute_schemindicator_bounds(ind: &Schemindicator) -> (f64, f64, f64, f64) {
    let m = &ind.transform;
    let mut global_points: Vec<(f64, f64)> = Vec::new();
    let to_global_pt = |x: f64, y: f64| to_global(x, y, ind.x, ind.y, m);

    for (lx, ly) in outline_bounding_points(&ind.outline) {
        global_points.push(to_global_pt(lx, ly));
    }

    points_to_bounds(&global_points).unwrap_or_else(|| {
        let center = to_global_pt(0.0, 0.0);
        let pad = 500.0;
        (
            center.0 - pad,
            center.1 - pad,
            center.0 + pad,
            center.1 + pad,
        )
    })
}

/// Compute wire segment bounds. Includes endpoints and text items.
pub fn compute_segment_bounds(seg: &WireSegment) -> (f64, f64, f64, f64) {
    let mut points = vec![(seg.x1, seg.y1), (seg.x2, seg.y2)];
    for ti in &seg.text_items {
        points.push((ti.x, ti.y));
    }
    points_to_bounds(&points).unwrap_or((seg.x1.min(seg.x2), seg.y1.min(seg.y2), seg.x1.max(seg.x2), seg.y1.max(seg.y2)))
}

/// Compute and store bounds for all elements in the diagram content.
pub fn compute_all_bounds(content: &mut DiagramContent) {
    for dev in &mut content.devices {
        dev.bounds = Some(compute_device_bounds(dev));
    }
    for conn in &mut content.connectors {
        conn.bounds = Some(compute_connector_bounds(conn));
    }
    for sp in &mut content.splices {
        sp.bounds = Some(compute_splice_bounds(sp));
    }
    for ind in &mut content.schemindicators {
        ind.bounds = Some(compute_schemindicator_bounds(ind));
    }
    for wire in &mut content.wires {
        for seg in &mut wire.segments {
            seg.bounds = Some(compute_segment_bounds(seg));
        }
    }
}
