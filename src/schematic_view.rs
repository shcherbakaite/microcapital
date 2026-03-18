//! Renders diagram content (border, wire segments, wire text) in schematic coordinates with scale-to-fit.
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

use eframe::egui;
use egui::text::TextFormat;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::Arc;
use crate::model::{
    ConnectorKind, ConnectorOutline, ConnectorPin, ConnectorSymbol, CrossRefKey,
    CrossRefMap, DevicePin, DiagramContent, HorizontalJust, Schemindicator, TextStyle, VerticalJust,
};

/// Brightens a color for dark mode contrast (does not change hue). Used for device colors only.
fn brighten_for_dark_mode(c: egui::Color32, factor: f32) -> egui::Color32 {
    let r = (c.r() as f32 / 255.0).powf(1.0 / factor);
    let g = (c.g() as f32 / 255.0).powf(1.0 / factor);
    let b = (c.b() as f32 / 255.0).powf(1.0 / factor);
    egui::Color32::from_rgb(
        (r * 255.0).min(255.0) as u8,
        (g * 255.0).min(255.0) as u8,
        (b * 255.0).min(255.0) as u8,
    )
}

/// Theme colors for schematic view. Adapts to light/dark mode.
struct SchematicTheme {
    dark_mode: bool,
    background: egui::Color32,
    grid: egui::Color32,
    label: egui::Color32,
    text: egui::Color32,
    wire_name: egui::Color32,
    default_wire: egui::Color32,
    splice_stroke: egui::Color32,
    splice_fill: egui::Color32,
    device_border: egui::Color32,
    device_fill: egui::Color32,
    connector_border_default: egui::Color32,
    connector_plug: egui::Color32,
    connector_jack: egui::Color32,
    connector_pin_name: egui::Color32,
    connector_pin_desc: egui::Color32,
    pin_name: egui::Color32,
    pin_desc: egui::Color32,
    overlay: egui::Color32,
    debug: egui::Color32,
    schemindicator_stroke: egui::Color32,
    highlight: egui::Color32,
}

impl SchematicTheme {
    fn from_dark_mode(dark: bool) -> Self {
        if dark {
            // Subdued dark palette: good contrast, low saturation, close to original
            Self {
                dark_mode: true,
                background: egui::Color32::from_rgb(71, 76, 92),   // original slate
                grid: egui::Color32::from_rgb(95, 100, 115),
                label: egui::Color32::from_gray(215),
                text: egui::Color32::from_gray(235),
                wire_name: egui::Color32::from_rgb(120, 200, 130), // muted green
                default_wire: egui::Color32::from_gray(200),
                splice_stroke: egui::Color32::from_rgb(150, 154, 160),
                splice_fill: egui::Color32::from_rgb(100, 104, 110),
                device_border: egui::Color32::from_rgb(165, 168, 175), // original
                device_fill: egui::Color32::from_rgb(78, 82, 90),
                connector_border_default: egui::Color32::from_rgb(200, 100, 100), // muted red
                connector_plug: egui::Color32::from_rgb(210, 165, 175), // soft pink
                connector_jack: egui::Color32::from_rgb(120, 175, 200), // soft blue
                connector_pin_name: egui::Color32::from_gray(220),
                connector_pin_desc: egui::Color32::from_gray(185),
                pin_name: egui::Color32::from_gray(175),
                pin_desc: egui::Color32::from_gray(155),
                overlay: egui::Color32::from_gray(200),
                debug: egui::Color32::from_gray(160),
                schemindicator_stroke: egui::Color32::from_rgb(155, 158, 165), // original
                highlight: egui::Color32::from_rgba_unmultiplied(71, 218, 92, 128), // muted gold
            }
        } else {
            Self {
                dark_mode: false,
                background: egui::Color32::from_gray(250),
                grid: egui::Color32::from_rgb(200, 200, 200),
                label: egui::Color32::from_gray(100),
                text: egui::Color32::BLACK,
                wire_name: egui::Color32::from_rgb(0, 128, 0),
                default_wire: egui::Color32::BLACK,
                splice_stroke: egui::Color32::from_rgb(100, 100, 100),
                splice_fill: egui::Color32::BLACK,
                device_border: egui::Color32::from_rgb(100, 100, 100),
                device_fill: egui::Color32::from_rgb(220, 220, 220),
                connector_border_default: egui::Color32::from_rgb(180, 0, 0),
                connector_plug: egui::Color32::from_rgb(255, 192, 203),
                connector_jack: egui::Color32::from_rgb(173, 216, 230),
                connector_pin_name: egui::Color32::from_rgb(100, 0, 0),
                connector_pin_desc: egui::Color32::from_rgb(80, 80, 80),
                pin_name: egui::Color32::from_rgb(80, 80, 80),
                pin_desc: egui::Color32::from_rgb(100, 100, 100),
                overlay: egui::Color32::from_rgb(40, 40, 40),
                debug: egui::Color32::from_gray(100),
                schemindicator_stroke: egui::Color32::from_rgb(80, 80, 80),
                highlight: egui::Color32::from_rgba_unmultiplied(71, 218, 92, 128),
            }
        }
    }
}

/// Computes the bounding rectangle of all content on the page (segments, devices, connectors,
/// splices, wire text). Returns (min_x, min_y, width, height) in logical units.
/// Uses precomputed element bounds when available for accurate grid extent.
fn compute_page_bounds(content: &DiagramContent) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let expand_rect = |min_x: &mut f64, min_y: &mut f64, max_x: &mut f64, max_y: &mut f64, (rx_min, ry_min, rx_max, ry_max): (f64, f64, f64, f64)| {
        *min_x = (*min_x).min(rx_min);
        *min_y = (*min_y).min(ry_min);
        *max_x = (*max_x).max(rx_max);
        *max_y = (*max_y).max(ry_max);
    };

    for wire in &content.wires {
        for seg in &wire.segments {
            if let Some(b) = seg.bounds {
                expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, b);
            } else {
                expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, (
                    seg.x1.min(seg.x2),
                    seg.y1.min(seg.y2),
                    seg.x1.max(seg.x2),
                    seg.y1.max(seg.y2),
                ));
                for ti in &seg.text_items {
                    expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, (ti.x, ti.y, ti.x, ti.y));
                }
            }
        }
    }

    for dev in &content.devices {
        if let Some(b) = dev.bounds {
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, b);
        } else {
            let m = &dev.transform;
            let (lx1, ly1) = (dev.extent_x, dev.extent_y);
            let (lx2, ly2) = (dev.extent_x + dev.width, dev.extent_y + dev.height);
            let corners = [
                (m[0] * lx1 + m[2] * ly1 + m[4], m[1] * lx1 + m[3] * ly1 + m[5]),
                (m[0] * lx2 + m[2] * ly1 + m[4], m[1] * lx2 + m[3] * ly1 + m[5]),
                (m[0] * lx2 + m[2] * ly2 + m[4], m[1] * lx2 + m[3] * ly2 + m[5]),
                (m[0] * lx1 + m[2] * ly2 + m[4], m[1] * lx1 + m[3] * ly2 + m[5]),
            ];
            let (gmin_x, gmax_x) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (x, _)| (mn.min(*x), mx.max(*x)));
            let (gmin_y, gmax_y) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (_, y)| (mn.min(*y), mx.max(*y)));
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, (gmin_x, gmin_y, gmax_x, gmax_y));
        }
    }

    for conn in &content.connectors {
        if let Some(b) = conn.bounds {
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, b);
        } else {
            let m = &conn.transform;
            let (lx1, ly1) = (conn.extent_x, conn.extent_y);
            let (lx2, ly2) = (conn.extent_x + conn.width, conn.extent_y + conn.height);
            let corners = [
                (m[0] * lx1 + m[2] * ly1 + m[4], m[1] * lx1 + m[3] * ly1 + m[5]),
                (m[0] * lx2 + m[2] * ly1 + m[4], m[1] * lx2 + m[3] * ly1 + m[5]),
                (m[0] * lx2 + m[2] * ly2 + m[4], m[1] * lx2 + m[3] * ly2 + m[5]),
                (m[0] * lx1 + m[2] * ly2 + m[4], m[1] * lx1 + m[3] * ly2 + m[5]),
            ];
            let (gmin_x, gmax_x) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (x, _)| (mn.min(*x), mx.max(*x)));
            let (gmin_y, gmax_y) = corners.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), (_, y)| (mn.min(*y), mx.max(*y)));
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, (gmin_x, gmin_y, gmax_x, gmax_y));
        }
    }

    for sp in &content.splices {
        if let Some(b) = sp.bounds {
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, b);
        } else {
            let m = &sp.transform;
            let (gx, gy) = (sp.x + m[4], sp.y + m[5]);
            let r = 50.0;
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, (gx - r, gy - r, gx + r, gy + r));
        }
    }

    for ind in &content.schemindicators {
        if let Some(b) = ind.bounds {
            expand_rect(&mut min_x, &mut min_y, &mut max_x, &mut max_y, b);
        }
    }

    if min_x == f64::INFINITY {
        return None;
    }

    let width = (max_x - min_x).max(1.0);
    let height = (max_y - min_y).max(1.0);
    Some((min_x, min_y, width, height))
}

/// Returns page extent (ox, oy, width, height) for scale-to-fit and coordinate mapping.
/// Uses border when present and valid; otherwise content bounds. Enables displaying diagrams without borders.
pub fn page_extent(content: &DiagramContent) -> Option<(f64, f64, f64, f64)> {
    match &content.border {
        Some(b) if b.width > 0.0 && b.height > 0.0 => Some((b.x, b.y, b.width, b.height)),
        _ => compute_page_bounds(content),
    }
}

/// Transforms logical (XML) coordinates to screen and draws the schematic.
/// `pan` is pixel offset; `zoom` multiplies the scale-to-fit factor (1.0 = fit).
/// `name_lookup`: id -> name from connectivity index; used when diagram content has empty names.
/// `cross_ref_map`: (Type Of, Device, Name) -> [(Diagram, Coord)] for right-click menu (built from content, not xrefs).
/// `current_diagram_name`: name of diagram being viewed (to exclude from "Other instances" menu).
/// `pending_nav`: when user picks a diagram from right-click menu, the diagram name is written here.
/// `selected_element`: (element_type, name) when user clicks an element in schematic; used for highlight and Elements view sync.
/// `expanded_elements`: when selection is set from schematic, add to this so Elements view expands the header.
/// `device_pins`: connref -> (pin_id -> shortdescription) for device short descriptions in overlay.
/// `element_shortdescription`: connref -> element short description from connectivity.
/// `wire_shortdescription`: wire id (harnwire) -> short description from sharedconductor.
pub fn paint_diagram(
    content: &DiagramContent,
    ui: &mut egui::Ui,
    rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
    name_lookup: &HashMap<String, String>,
    cross_ref_map: &CrossRefMap,
    current_diagram_name: Option<&str>,
    pending_nav: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    selected_element: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>,
    expanded_elements: std::rc::Rc<std::cell::RefCell<HashSet<(String, String)>>>,
    _device_pins: &HashMap<String, HashMap<String, String>>,
    element_shortdescription: &HashMap<String, String>,
    wire_shortdescription: &HashMap<String, String>,
) {
    let theme = SchematicTheme::from_dark_mode(ui.ctx().style().visuals.dark_mode);

    // Page extent: use border when present and valid; otherwise use content bounds.
    // Since we build the grid ourselves, diagrams without borders can be displayed.
    let (ox, oy, page_w, page_h) = match page_extent(content) {
        Some(ext) => ext,
        None => {
            let _ = ui.scope_builder(egui::UiBuilder::default().max_rect(rect), |ui| {
                ui.label("No content");
            });
            return;
        }
    };

    let (base_scale, offset_x, offset_y) = scale_to_fit(
        page_w,
        page_h,
        rect.width(),
        rect.height(),
    );
    let scale = base_scale * zoom;
    let offset_x = offset_x + pan.x;
    let offset_y = offset_y + pan.y;

    // Y flip: logical Y often increases downward; we want origin at top-left of rect.
    let to_screen = |x: f64, y: f64| {
        egui::Pos2::new(
            rect.min.x + offset_x + ((x - ox) as f32) * scale,
            rect.max.y - offset_y - ((y - oy) as f32) * scale, // flip Y
        )
    };

    // Background
    ui.painter_at(rect).rect_filled(
        rect,
        0.0,
        theme.background,
    );

    // Page bounds: union of border and content when border exists; else content bounds.
    let (grid_ox, grid_oy, grid_w, grid_h) = match (&content.border, compute_page_bounds(content)) {
        (Some(b), Some((cx, cy, cw, ch))) if b.width > 0.0 && b.height > 0.0 => {
            let border_min_x = b.x;
            let border_min_y = b.y;
            let border_max_x = b.x + b.width;
            let border_max_y = b.y + b.height;
            let content_max_x = cx + cw;
            let content_max_y = cy + ch;
            let min_x = border_min_x.min(cx);
            let min_y = border_min_y.min(cy);
            let max_x = border_max_x.max(content_max_x);
            let max_y = border_max_y.max(content_max_y);
            (min_x, min_y, (max_x - min_x).max(1.0), (max_y - min_y).max(1.0))
        }
        _ => (ox, oy, page_w, page_h),
    };

    // Border grid: alphanumeric on vertical axis (left), numeric on horizontal (bottom).
    // Grid extent is the page bounding rectangle. Equal spacing in both directions (square cells).
    const GRID_DIVISIONS: usize = 10;
    let grid_stroke = egui::Stroke::new((1.0 * scale).max(0.5), theme.grid);
    let label_font_size = 14.0; // fixed size for viewport-edge labels
    let label_inset = 8.0;
    let painter = ui.painter_at(rect);

    let cell_size = (grid_w.max(grid_h) / (GRID_DIVISIONS as f64)).max(1.0);
    // Round up so we never cut a cell in the middle; grid extent = whole cells only.
    let n_cols = (grid_w / cell_size).ceil().max(1.0) as usize;
    let n_rows = (grid_h / cell_size).ceil().max(1.0) as usize;
    let grid_extent_w = (n_cols as f64) * cell_size;
    let grid_extent_h = (n_rows as f64) * cell_size;
    let dx = cell_size;
    let dy = cell_size;

    // Grid lines (vertical and horizontal) — extent rounded up so all lines connect
    for i in 0..=n_cols {
        let lx = grid_ox + (i as f64) * dx;
        let from = to_screen(lx, grid_oy);
        let to = to_screen(lx, grid_oy + grid_extent_h);
        painter.line_segment([from, to], grid_stroke);
    }
    for j in 0..=n_rows {
        let ly = grid_oy + (j as f64) * dy;
        let from = to_screen(grid_ox, ly);
        let to = to_screen(grid_ox + grid_extent_w, ly);
        painter.line_segment([from, to], grid_stroke);
    }

    // Project grid coordinates onto viewport edges: labels always visible at screen edges
    // even when zoomed in and border edges are off-screen.
    let viewport_left = rect.min.x;
    let viewport_bottom = rect.max.y;

    // Vertical axis labels (alphanumeric) on left edge of viewport
    for j in 0..=n_rows {
        let ly = grid_oy + (j as f64) * dy;
        let screen_y = rect.max.y - offset_y - ((ly - oy) as f32) * scale;
        if screen_y >= rect.min.y && screen_y <= rect.max.y {
            let label = index_to_alphanumeric(j);
            let label_pos = egui::pos2(viewport_left + label_inset, screen_y);
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(label_font_size),
                theme.label,
            );
        }
    }

    // Horizontal axis labels (numeric) on bottom edge of viewport
    for i in 0..=n_cols {
        let lx = grid_ox + (i as f64) * dx;
        let screen_x = rect.min.x + offset_x + ((lx - ox) as f32) * scale;
        if screen_x >= rect.min.x && screen_x <= rect.max.x {
            let label = (i + 1).to_string();
            let label_pos = egui::pos2(screen_x, viewport_bottom - label_inset);
            painter.text(
                label_pos,
                egui::Align2::CENTER_BOTTOM,
                label,
                egui::FontId::proportional(label_font_size),
                theme.label,
            );
        }
    }

    // Build set of connected wire endpoints (splices, connector pins, device pins, segment junctions).
    // Use rounded logical coords for tolerance.
    let round_pt = |x: f64, y: f64| (x.round() as i64, y.round() as i64);
    let mut connected: HashSet<(i64, i64)> = HashSet::new();
    for splice in &content.splices {
        let m = &splice.transform;
        let (gx, gy) = (splice.x + m[4], splice.y + m[5]);
        connected.insert(round_pt(gx, gy));
    }
    for conn in &content.connectors {
        let m = &conn.transform;
        for pin in &conn.pins {
            let (gx, gy) = (conn.x + m[0] * pin.x + m[2] * pin.y + m[4], conn.y + m[1] * pin.x + m[3] * pin.y + m[5]);
            connected.insert(round_pt(gx, gy));
        }
    }
    for dev in &content.devices {
        let m = &dev.transform;
        for pin in &dev.pins {
            let (gx, gy) = (dev.x + m[0] * pin.x + m[2] * pin.y + m[4], dev.y + m[1] * pin.x + m[3] * pin.y + m[5]);
            connected.insert(round_pt(gx, gy));
        }
    }
    let mut endpoint_count: HashMap<(i64, i64), u32> = HashMap::new();
    for wire in &content.wires {
        for seg in &wire.segments {
            *endpoint_count.entry(round_pt(seg.x1, seg.y1)).or_insert(0) += 1;
            *endpoint_count.entry(round_pt(seg.x2, seg.y2)).or_insert(0) += 1;
        }
    }

    // Selected wire name/id for highlight (Elements panel uses both)
    let selected_wire_key = selected_element.borrow().as_ref().and_then(|(t, n)| {
        if t == "Wire" { Some(n.clone()) } else { None }
    });

    // Wire segments
    for wire in &content.wires {
        let is_selected = selected_wire_key.as_ref().map_or(false, |k| wire.name == *k || wire.id == *k);
        for seg in &wire.segments {
        let color = seg
            .attributeref
            .as_ref()
            .and_then(|id| content.datadictionary.get(id))
            .and_then(|dd| dd.color.as_ref())
            .and_then(|s| parse_color(s))
            .unwrap_or(theme.default_wire);
        let thickness = seg
            .attributeref
            .as_ref()
            .and_then(|id| content.datadictionary.get(id))
            .and_then(|dd| dd.thickness)
            .unwrap_or(1.0) as f32 * scale.max(0.5);

        let from = to_screen(seg.x1, seg.y1);
        let to = to_screen(seg.x2, seg.y2);
        ui.painter_at(rect).line_segment(
            [from, to],
            egui::Stroke::new(thickness, color),
        );

        // Right-click context menu: list other diagram instances from cross-ref map (not xrefs)
        let wire_names: Vec<String> = seg
            .text_items
            .iter()
            .filter(|ti| ti.is_wire_name && !ti.text.is_empty())
            .map(|ti| ti.text.clone())
            .collect();
        let mut other_targets: Vec<(String, (f64, f64))> = Vec::new();
        for name in &wire_names {
            let key = CrossRefKey("Wire".to_string(), String::new(), name.clone());
            if let Some(targets) = cross_ref_map.get(&key) {
                for t in targets {
                    if current_diagram_name != Some(t.0.as_str()) {
                        other_targets.push((t.0.clone(), t.1));
                    }
                }
            }
        }
        // Dedupe by diagram name (same wire can have multiple segments on one diagram)
        other_targets.sort_by(|a, b| a.0.cmp(&b.0));
        other_targets.dedup_by(|a, b| a.0 == b.0);
        // Always allocate rect for click/selection (even when no other instances)
        let seg_rect = egui::Rect::from_two_pos(from, to).expand((thickness * 2.0).max(8.0));
        let response = ui.allocate_rect(seg_rect, egui::Sense::click());
        // Use wire.id when set (matches WireRef.id in Elements view); else wire name for diagram-only wires.
        let sel_key: String = if !wire.id.is_empty() {
            wire.id.clone()
        } else if !wire.name.is_empty() {
            wire.name.clone()
        } else {
            wire_names.first().cloned().unwrap_or_default()
        };
        if !sel_key.is_empty() && response.clicked() {
            *selected_element.borrow_mut() = Some(("Wire".to_string(), sel_key.clone()));
            expanded_elements.borrow_mut().insert(("Wire".to_string(), sel_key));
        }
        if !other_targets.is_empty() {
            let targets = other_targets.clone();
            let pending = pending_nav.clone();
            response.context_menu(move |ui| {
                ui.label("Other instances:");
                for (diagram_name, _coord) in &targets {
                    if ui.button(format!("Diagram: {}", diagram_name)).clicked() {
                        *pending.borrow_mut() = Some(diagram_name.clone());
                        ui.close();
                    }
                }
            });
        }
        // Highlight selected wire: thick line following wire shape (all segments of the path)
        if is_selected {
            let highlight = egui::Stroke::new((thickness * 2.5).max(6.0), theme.highlight);
            ui.painter_at(rect).line_segment([from, to], highlight);
        }

        // Wire reference (backward double arrow) at unconnected ends
        let p1 = round_pt(seg.x1, seg.y1);
        let p2 = round_pt(seg.x2, seg.y2);
        let is_ref1 = !connected.contains(&p1) && endpoint_count.get(&p1) == Some(&1);
        let is_ref2 = !connected.contains(&p2) && endpoint_count.get(&p2) == Some(&1);
        // Match wire text size: use segment text height or typical Capital wire name height (1382)
        let wire_text_size = seg
            .text_items
            .iter()
            .map(|ti| ti.height as f32 * scale)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1382.0 * scale);
        let arrow_size = wire_text_size.max(8.0);
        if is_ref1 {
            paint_backward_double_arrow(
                &ui.painter_at(rect),
                from,
                to,
                arrow_size,
                thickness,
                color,
            );
        }
        if is_ref2 {
            paint_backward_double_arrow(
                &ui.painter_at(rect),
                to,
                from,
                arrow_size,
                thickness,
                color,
            );
        }

        // Wire text items (wire name, gauge, color). Cross-refs hidden; use right-click menu.
        // Capital compositetext x,y are segment-local. For vertical: x=perp, y=along (axes swapped).
        for ti in &seg.text_items {
            if ti.is_xref {
                continue; // Hide cross-ref text; instances shown in right-click menu
            }
            let (gx, gy) = text_pos_to_global(seg, ti.x, ti.y);
            let pos = to_screen(gx, gy);
            let font_size = (ti.height as f32 * scale).max(8.0);
            let align = text_align(ti.hjust, ti.vjust);
            let angle_rad = (ti.rotation as f32).to_radians();

            if ti.is_wire_name {
                paint_wire_text_with_border(
                    &ui.painter_at(rect),
                    pos,
                    align,
                    &ti.text,
                    font_size,
                    ti.style,
                    angle_rad,
                    theme.wire_name,
                );
            } else {
                let text_color = theme.text;
                if angle_rad.abs() < 0.001 {
                    paint_text_with_style(
                        &ui.painter_at(rect),
                        pos,
                        align,
                        &ti.text,
                        font_size,
                        text_color,
                        ti.style,
                    );
                } else {
                    let painter = ui.painter_at(rect);
                    let galley = if matches!(ti.style, TextStyle::Italic | TextStyle::BoldItalic) {
                        let format = TextFormat {
                            font_id: egui::FontId::proportional(font_size),
                            color: text_color,
                            italics: true,
                            ..Default::default()
                        };
                        let mut job = egui::text::LayoutJob::simple_format(ti.text.clone(), format);
                        job.wrap.max_width = f32::INFINITY;
                        painter.layout_job(job)
                    } else {
                        painter.layout_no_wrap(
                            ti.text.clone(),
                            egui::FontId::proportional(font_size),
                            text_color,
                        )
                    };
                    let text_rect = align.anchor_size(pos, galley.size());
                    let text_shape = egui::epaint::TextShape::new(
                        text_rect.min,
                        galley,
                        text_color,
                    )
                    .with_override_text_color(text_color)
                    .with_angle_and_anchor(angle_rad, align);
                    painter.add(egui::Shape::Text(text_shape));
                }
            }
        }
    }
    }

    // Collect connector selection highlights to draw on top (after splices/devices)
    let mut connector_selection_rects: Vec<egui::Rect> = Vec::new();

    // Connector symbols (plug = rounded pink, jack = blue rectangle)
    for conn in &content.connectors {
        let m = &conn.transform;
        // Transform maps local coords to diagram; conn.x, conn.y are the diagram anchor
        let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
            (
                conn.x + m[0] * lx + m[2] * ly + m[4],
                conn.y + m[1] * lx + m[3] * ly + m[5],
            )
        };

        // Body bounds: use paramextent for fill polygon (empty outline case)
        let (d0x, d0y) = transform_pt(conn.extent_x, conn.extent_y);
        let (d1x, d1y) = transform_pt(conn.extent_x + conn.width, conn.extent_y);
        let (d2x, d2y) = transform_pt(conn.extent_x + conn.width, conn.extent_y + conn.height);
        let (d3x, d3y) = transform_pt(conn.extent_x, conn.extent_y + conn.height);
        let extent_min_x = d0x.min(d1x).min(d2x).min(d3x);
        let extent_max_x = d0x.max(d1x).max(d2x).max(d3x);
        let extent_min_y = d0y.min(d1y).min(d2y).min(d3y);
        let extent_max_y = d0y.max(d1y).max(d2y).max(d3y);

        let actual_body_rect = egui::Rect::from_min_max(
            to_screen(extent_min_x, extent_max_y),
            to_screen(extent_max_x, extent_min_y),
        );
        let mut body_rect = actual_body_rect;

        // Bounds rect for hit test and selection: use precomputed bounds when available
        let (bounds_min_x, bounds_min_y, bounds_max_x, bounds_max_y) = conn.bounds
            .unwrap_or((extent_min_x, extent_min_y, extent_max_x, extent_max_y));
        let mut bounds_rect = egui::Rect::from_min_max(
            to_screen(bounds_min_x, bounds_max_y),
            to_screen(bounds_max_x, bounds_min_y),
        );
        let min_size = 20.0;
        if bounds_rect.width() < min_size || bounds_rect.height() < min_size {
            let c = bounds_rect.center();
            bounds_rect = egui::Rect::from_center_size(
                c,
                egui::vec2(
                    bounds_rect.width().max(min_size),
                    bounds_rect.height().max(min_size),
                ),
            );
        }
        if body_rect.width() < min_size || body_rect.height() < min_size {
            let c = body_rect.center();
            body_rect = egui::Rect::from_center_size(
                c,
                egui::vec2(
                    body_rect.width().max(min_size),
                    body_rect.height().max(min_size),
                ),
            );
        }
        let border_color = conn.attributeref.as_ref()
            .and_then(|id| content.datadictionary.get(id))
            .and_then(|dd| dd.color.as_ref())
            .and_then(|s| parse_color(s))
            .unwrap_or(theme.connector_border_default);
        let body_color = match conn.kind {
            ConnectorKind::Plug => theme.connector_plug,
            ConnectorKind::Jack => theme.connector_jack,
        };
        // Single stroke width for all outline primitives so lines don't appear variable thickness
        let stroke = egui::Stroke::new((2.0 * scale).max(1.0), border_color);

        let fill_polygon = connector_fill_polygon(conn, body_rect, &transform_pt, &to_screen);

        if conn.outline.is_empty() {
            // Fallback: draw extent rectangle as filled polygon
            if fill_polygon.len() >= 3 {
                paint_polygon_fill(&ui.painter_at(rect), &fill_polygon, body_color, stroke);
            } else {
                ui.painter_at(rect).rect_filled(body_rect, egui::CornerRadius::ZERO, body_color);
                ui.painter_at(rect).rect_stroke(
                    body_rect,
                    egui::CornerRadius::ZERO,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        } else {
            // Draw background fill as polygon following exact connector shape
            if fill_polygon.len() >= 3 {
                // Plug: solid light pink; Jack: solid light blue (full opacity for proper fill)
                let fill_color = body_color;
                paint_polygon_fill(
                    &ui.painter_at(rect),
                    &fill_polygon,
                    fill_color,
                    egui::Stroke::NONE,
                );
            }

            // Draw geometry primitives (polyline, arc, line) — stroke only for outline shapes.
            // Variable thickness can appear when the same edge is drawn more than once: (1) two
            // Rectangle primitives that share an edge each stroke that edge (StrokeKind::Inside),
            // or (2) a Rectangle and a Line/Polyline coincide on the same edge.
            let painter = ui.painter_at(rect);
            for prim in &conn.outline {
                match prim {
                    ConnectorOutline::Line { x1, y1, x2, y2 } => {
                        let dx = x2 - x1;
                        let dy = y2 - y1;
                        if dx * dx + dy * dy < 1e-12 {
                            continue; // skip degenerate line
                        }
                        let (gx1, gy1) = transform_pt(*x1, *y1);
                        let (gx2, gy2) = transform_pt(*x2, *y2);
                        painter.line_segment(
                            [to_screen(gx1, gy1), to_screen(gx2, gy2)],
                            stroke,
                        );
                    }
                    ConnectorOutline::Arc {
                        x: cx,
                        y: cy,
                        radius,
                        start_angle,
                        travel_angle,
                    } => {
                        if radius.abs() < 1e-9 || travel_angle.abs() < 1e-9 {
                            continue;
                        }
                        // Capital: 0°=3 o'clock, travelangle sweeps toward 90° (down).
                        let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                        let mut points = Vec::with_capacity(n + 1);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + radius * angle_rad.cos();
                            let py = cy + radius * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        if points.len() >= 2 {
                            painter.add(egui::Shape::line(points, stroke));
                        }
                    }
                    ConnectorOutline::Polyline { points: pts } => {
                        if pts.len() >= 2 {
                            let screen_pts: Vec<egui::Pos2> = pts
                                .iter()
                                .map(|(px, py)| {
                                    let (gx, gy) = transform_pt(*px, *py);
                                    to_screen(gx, gy)
                                })
                                .collect();
                            painter.add(egui::Shape::line(screen_pts, stroke));
                        }
                    }
                    ConnectorOutline::Rectangle { x, y, width, height } => {
                        let (g0x, g0y) = transform_pt(*x, *y);
                        let (g1x, g1y) = transform_pt(x + width, *y);
                        let (g2x, g2y) = transform_pt(x + width, y + height);
                        let (g3x, g3y) = transform_pt(*x, y + height);
                        let min_x = g0x.min(g1x).min(g2x).min(g3x);
                        let max_x = g0x.max(g1x).max(g2x).max(g3x);
                        let min_y = g0y.min(g1y).min(g2y).min(g3y);
                        let max_y = g0y.max(g1y).max(g2y).max(g3y);
                        let r = egui::Rect::from_min_max(
                            to_screen(min_x, max_y),
                            to_screen(max_x, min_y),
                        );
                        let rounding = match conn.kind {
                            ConnectorKind::Plug => {
                                let rad = r.width().min(r.height()) * 0.15;
                                egui::CornerRadius::same((rad.max(3.0) as u8).max(3))
                            }
                            ConnectorKind::Jack => egui::CornerRadius::ZERO,
                        };
                        // Outline only (no fill) so symbol rectangles are not filled
                        painter.rect_stroke(r, rounding, stroke, egui::StrokeKind::Inside);
                    }
                    ConnectorOutline::Circle { x, y, radius } => {
                        if radius.abs() < 1e-9 {
                            continue;
                        }
                        let (gx, gy) = transform_pt(*x, *y);
                        let center = to_screen(gx, gy);
                        let r_screen = (radius * scale as f64).max(2.0) as f32;
                        // Outline only (no fill) so symbol circles are not filled
                        painter.circle_stroke(center, r_screen, stroke);
                    }
                    ConnectorOutline::WidthArc {
                        x: cx,
                        y: cy,
                        radius,
                        start_angle,
                        travel_angle,
                        width,
                    } => {
                        let r_inner = radius;
                        let r_outer = radius + width;
                        let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                        let mut points = Vec::with_capacity(2 * n + 2);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_outer * angle_rad.cos();
                            let py = cy + r_outer * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        for i in (0..=n).rev() {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_inner * angle_rad.cos();
                            let py = cy + r_inner * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        if points.len() >= 3 {
                            let fill = egui::Color32::from_rgba_unmultiplied(
                                body_color.r(), body_color.g(), body_color.b(), 180,
                            );
                            painter.add(egui::Shape::convex_polygon(
                                points.clone(),
                                fill,
                                stroke,
                            ));
                        }
                    }
                    ConnectorOutline::Polygon { points: pts } => {
                        if pts.len() >= 3 {
                            let screen_pts: Vec<egui::Pos2> = pts
                                .iter()
                                .map(|(px, py)| {
                                    let (gx, gy) = transform_pt(*px, *py);
                                    to_screen(gx, gy)
                                })
                                .collect();
                            let fill = egui::Color32::from_rgba_unmultiplied(
                                body_color.r(), body_color.g(), body_color.b(), 180,
                            );
                            painter.add(egui::Shape::convex_polygon(
                                screen_pts.clone(),
                                fill,
                                stroke,
                            ));
                        }
                    }
                    ConnectorOutline::Curve { points: pts } => {
                        if pts.len() >= 2 {
                            let screen_pts = catmull_rom_spline(pts, 8, |px, py| {
                                let (gx, gy) = transform_pt(px, py);
                                to_screen(gx, gy)
                            });
                            if screen_pts.len() >= 2 {
                                painter.add(egui::Shape::line(screen_pts, stroke));
                            }
                        }
                    }
                }
            }
        }

        // Connector name label: use attributetext placement when available, else center above body
        let label = if !conn.connref.is_empty() {
            name_lookup.get(&conn.connref).cloned()
        } else {
            None
        }.or_else(|| if !conn.name.is_empty() { Some(conn.name.clone()) } else { None })
            .unwrap_or_else(|| format!("{} pins", conn.pins.len()));
        let (name_pos, align, font_size, rotation) = if let Some(ref pl) = conn.name_placement {
            let (gx, gy) = transform_pt(pl.x, pl.y);
            let pos = to_screen(gx, gy);
            let fs = (pl.height as f32 * scale).max(8.0);
            // Text rotation is in symbol-local (logical, Y-up) coords; add symbol transform for diagram.
            // Negate for screen: we Y-flip to screen, so vertical text would otherwise read top-to-bottom.
            let (rotation_deg, flipped) = normalize_label_rotation_deg(
                -(pl.rotation + transform_rotation_deg(&conn.transform)),
            );
            let hjust = if flipped { swap_hjust(pl.hjust) } else { pl.hjust };
            let align = text_align(hjust, pl.vjust);
            (pos, align, fs, rotation_deg)
        } else {
            let font_size = (actual_body_rect.height() * 0.4).max(8.0);
            let name_pos = egui::pos2(body_rect.center().x, body_rect.min.y - font_size * 0.3);
            (name_pos, egui::Align2::CENTER_BOTTOM, font_size, 0.0)
        };
        paint_text_with_style_and_rotation(
            &ui.painter_at(rect),
            name_pos,
            align,
            &label,
            font_size,
            border_color,
            conn.name_style,
            rotation,
        );

        // Pins: transform position, draw diamond (unfilled) and text
        for pin in &conn.pins {
            let (dx, dy) = transform_pt(pin.x, pin.y);
            let pos = to_screen(dx, dy);
            let pin_size = (6.0 * scale).max(3.0);
            let diamond: [egui::Pos2; 4] = [
                pos + egui::vec2(pin_size, 0.0),
                pos + egui::vec2(0.0, pin_size),
                pos + egui::vec2(-pin_size, 0.0),
                pos + egui::vec2(0.0, -pin_size),
            ];
            ui.painter_at(rect).add(egui::Shape::closed_line(
                diamond.to_vec(),
                egui::Stroke::new(1.5, border_color),
            ));
            let conn_rotation = transform_rotation_deg(&conn.transform);
            let (text_pos, text_align, font_size, rotation) =
                pin_text_pos(pin, &transform_pt, to_screen, scale, conn_rotation);
            if pin.name_visible {
                paint_text_with_style_and_rotation(
                    &ui.painter_at(rect),
                    text_pos,
                    text_align,
                    &pin.text,
                    font_size,
                    theme.connector_pin_name,
                    crate::model::TextStyle::Plain,
                    rotation,
                );
            }
            // Connector pin short descriptions
            if let Some(ref desc) = pin.short_description {
                if !desc.is_empty() && desc != &pin.text {
                    let (desc_pos, desc_align, desc_font_size, desc_rotation) =
                        pin_short_desc_pos(pin, &transform_pt, to_screen, scale, conn_rotation);
                    paint_text_with_style_and_rotation(
                        &ui.painter_at(rect),
                        desc_pos,
                        desc_align,
                        desc,
                        desc_font_size,
                        theme.connector_pin_desc,
                        crate::model::TextStyle::Plain,
                        desc_rotation,
                    );
                }
            }
        }

        // Right-click context menu and click-to-select
        let conn_name = if conn.name.is_empty() {
            conn.connref.clone()
        } else {
            conn.name.clone()
        };
        let key = CrossRefKey("Connector".to_string(), String::new(), conn_name.clone());
        let mut other_targets: Vec<(String, (f64, f64))> = cross_ref_map
            .get(&key)
            .map(|v| v.iter().map(|t| (t.0.clone(), t.1)).collect())
            .unwrap_or_default();
        other_targets.retain(|(dn, _)| current_diagram_name != Some(dn.as_str()));
        other_targets.sort_by(|a, b| a.0.cmp(&b.0));
        other_targets.dedup_by(|a, b| a.0 == b.0);
        let hit_rect = bounds_rect.expand(4.0);
        let response = ui.allocate_rect(hit_rect, egui::Sense::click());
        if response.clicked() {
            *selected_element.borrow_mut() = Some(("Connector".to_string(), conn_name.clone()));
            expanded_elements.borrow_mut().insert(("Connector".to_string(), conn_name.clone()));
        }
        if !other_targets.is_empty() {
            let targets = other_targets.clone();
            let pending = pending_nav.clone();
            response.context_menu(move |ui| {
                ui.label("Other instances:");
                for (diagram_name, _) in &targets {
                    if ui.button(format!("Diagram: {}", diagram_name)).clicked() {
                        *pending.borrow_mut() = Some(diagram_name.clone());
                        ui.close();
                    }
                }
            });
        }
        if let Some(ref sel) = *selected_element.borrow() {
            if sel.0 == "Connector" && sel.1 == conn_name {
                connector_selection_rects.push(hit_rect);
            }
        }
    }

    // Splice symbols (wire junction points, typically circles) — outline only, no fill
    for splice in &content.splices {
        let m = &splice.transform;
        let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
            (
                splice.x + m[0] * lx + m[2] * ly + m[4],
                splice.y + m[1] * lx + m[3] * ly + m[5],
            )
        };
        // Splice outline: gray stroke only, never filled; thin stroke so circles stay hollow
        let stroke_color = theme.splice_stroke;
        let base_stroke_w = (2.0 * scale).max(1.0);
        let painter = ui.painter_at(rect);
        for prim in &splice.outline {
            match prim {
                crate::model::ConnectorOutline::Circle { x, y, radius } => {
                    if radius.abs() < 1e-9 {
                        continue;
                    }
                    let (gx, gy) = transform_pt(*x, *y);
                    let center = to_screen(gx, gy);
                    let r_screen = (radius * scale as f64).max(2.0) as f32;
                    painter.circle_filled(center, r_screen, theme.splice_fill);
                }
                crate::model::ConnectorOutline::Line { x1, y1, x2, y2 } => {
                    let (gx1, gy1) = transform_pt(*x1, *y1);
                    let (gx2, gy2) = transform_pt(*x2, *y2);
                    let stroke = egui::Stroke::new(base_stroke_w, stroke_color);
                    painter.line_segment(
                        [to_screen(gx1, gy1), to_screen(gx2, gy2)],
                        stroke,
                    );
                }
                crate::model::ConnectorOutline::Arc {
                    x: cx,
                    y: cy,
                    radius,
                    start_angle,
                    travel_angle,
                } => {
                    if radius.abs() < 1e-9 || travel_angle.abs() < 1e-9 {
                        continue;
                    }
                    let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                    let mut points = Vec::with_capacity(n + 1);
                    for i in 0..=n {
                        let t = (i as f64) / (n as f64);
                        let angle_deg = start_angle + t * travel_angle;
                        let angle_rad = angle_deg * PI / 180.0;
                        let px = cx + radius * angle_rad.cos();
                        let py = cy + radius * angle_rad.sin();
                        let (gx, gy) = transform_pt(px, py);
                        points.push(to_screen(gx, gy));
                    }
                    if points.len() >= 2 {
                        let stroke = egui::Stroke::new(base_stroke_w, stroke_color);
                        painter.add(egui::Shape::line(points, stroke));
                    }
                }
                crate::model::ConnectorOutline::Rectangle { x, y, width, height } => {
                    let (g0x, g0y) = transform_pt(*x, *y);
                    let (g1x, g1y) = transform_pt(x + width, *y);
                    let (g2x, g2y) = transform_pt(x + width, y + height);
                    let (g3x, g3y) = transform_pt(*x, y + height);
                    let min_x = g0x.min(g1x).min(g2x).min(g3x);
                    let max_x = g0x.max(g1x).max(g2x).max(g3x);
                    let min_y = g0y.min(g1y).min(g2y).min(g3y);
                    let max_y = g0y.max(g1y).max(g2y).max(g3y);
                    let r = egui::Rect::from_min_max(
                        to_screen(min_x, max_y),
                        to_screen(max_x, min_y),
                    );
                    let stroke = egui::Stroke::new(base_stroke_w, stroke_color);
                    painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
                }
                _ => {}
            }
        }
        let label = name_lookup
            .get(&splice.connref)
            .cloned()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| splice.name.clone());
        let (name_pos, align, font_size, rotation) = if let Some(ref pl) = splice.name_placement {
            let (gx, gy) = transform_pt(pl.x, pl.y);
            let pos = to_screen(gx, gy);
            let fs = (pl.height as f32 * scale).max(8.0);
            let (rotation_deg, flipped) =
                normalize_label_rotation_deg(-(pl.rotation + transform_rotation_deg(&splice.transform)));
            let hjust = if flipped { swap_hjust(pl.hjust) } else { pl.hjust };
            let align = text_align(hjust, pl.vjust);
            (pos, align, fs, rotation_deg)
        } else {
            let (gx, gy) = transform_pt(0.0, 0.0);
            let center = to_screen(gx, gy);
            let font_size = (12.0 * scale).max(8.0);
            let name_pos = egui::pos2(center.x, center.y - font_size * 1.5);
            (name_pos, egui::Align2::CENTER_BOTTOM, font_size, 0.0)
        };
        paint_text_with_style_and_rotation(
            &ui.painter_at(rect),
            name_pos,
            align,
            &label,
            font_size,
            stroke_color,
            splice.name_style,
            rotation,
        );

        // Right-click context menu and click-to-select
        let splice_name = if splice.name.is_empty() {
            splice.connref.clone()
        } else {
            splice.name.clone()
        };
        let key = CrossRefKey("Splice".to_string(), String::new(), splice_name.clone());
        let mut other_targets: Vec<(String, (f64, f64))> = cross_ref_map
            .get(&key)
            .map(|v| v.iter().map(|t| (t.0.clone(), t.1)).collect())
            .unwrap_or_default();
        other_targets.retain(|(dn, _)| current_diagram_name != Some(dn.as_str()));
        other_targets.sort_by(|a, b| a.0.cmp(&b.0));
        other_targets.dedup_by(|a, b| a.0 == b.0);
        let (cx, cy) = transform_pt(0.0, 0.0);
        let center = to_screen(cx, cy);
        let hit_size = (base_stroke_w * 8.0).max(16.0);
        let hit_rect = egui::Rect::from_center_size(center, egui::vec2(hit_size, hit_size));
        let response = ui.allocate_rect(hit_rect, egui::Sense::click());
        if response.clicked() {
            *selected_element.borrow_mut() = Some(("Splice".to_string(), splice_name.clone()));
            expanded_elements.borrow_mut().insert(("Splice".to_string(), splice_name.clone()));
        }
        if !other_targets.is_empty() {
            let targets = other_targets.clone();
            let pending = pending_nav.clone();
            response.context_menu(move |ui| {
                ui.label("Other instances:");
                for (diagram_name, _) in &targets {
                    if ui.button(format!("Diagram: {}", diagram_name)).clicked() {
                        *pending.borrow_mut() = Some(diagram_name.clone());
                        ui.close();
                    }
                }
            });
        }
        if let Some(ref sel) = *selected_element.borrow() {
            if sel.0 == "Splice" && sel.1 == splice_name {
                ui.painter_at(rect).rect_stroke(
                    hit_rect,
                    0.0,
                    egui::Stroke::new(3.0, theme.highlight),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }

    // Schemindicators (oval, twist-commercial, shield) — stroke outline on wires
    paint_schemindicators(
        ui,
        &content.schemindicators,
        content,
        rect,
        scale,
        &to_screen,
        &theme,
    );

    // Device symbols (generic rectangles or complex outlines)
    for dev in &content.devices {
        let dd_color = dev.attributeref.as_ref()
            .and_then(|id| content.datadictionary.get(id))
            .and_then(|dd| dd.color.as_ref())
            .and_then(|s| parse_color(s));
        let (device_border, device_fill) = if let Some(c) = dd_color {
            let border = if theme.dark_mode {
                theme.device_border // Override: always use high-contrast theme color for lines
            } else {
                c
            };
            let fill = egui::Color32::from_rgb(
                ((c.r() as u32 * 2 + 255) / 3).min(255) as u8,
                ((c.g() as u32 * 2 + 255) / 3).min(255) as u8,
                ((c.b() as u32 * 2 + 255) / 3).min(255) as u8,
            );
            let fill = if theme.dark_mode {
                brighten_for_dark_mode(fill, 1.6)
            } else {
                fill
            };
            (border, fill)
        } else {
            (theme.device_border, theme.device_fill)
        };
        let m = &dev.transform;
        let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
            (
                dev.x + m[0] * lx + m[2] * ly + m[4],
                dev.y + m[1] * lx + m[3] * ly + m[5],
            )
        };

        let (d0x, d0y) = transform_pt(dev.extent_x, dev.extent_y);
        let (d1x, d1y) = transform_pt(dev.extent_x + dev.width, dev.extent_y);
        let (d2x, d2y) = transform_pt(dev.extent_x + dev.width, dev.extent_y + dev.height);
        let (d3x, d3y) = transform_pt(dev.extent_x, dev.extent_y + dev.height);
        let min_x = d0x.min(d1x).min(d2x).min(d3x);
        let max_x = d0x.max(d1x).max(d2x).max(d3x);
        let min_y = d0y.min(d1y).min(d2y).min(d3y);
        let max_y = d0y.max(d1y).max(d2y).max(d3y);

        let actual_body_rect = egui::Rect::from_min_max(
            to_screen(min_x, max_y),
            to_screen(max_x, min_y),
        );
        let mut body_rect = actual_body_rect;
        let min_size = 20.0;
        if body_rect.width() < min_size || body_rect.height() < min_size {
            let c = body_rect.center();
            body_rect = egui::Rect::from_center_size(
                c,
                egui::vec2(
                    body_rect.width().max(min_size),
                    body_rect.height().max(min_size),
                ),
            );
        }
        // Single stroke width for all outline primitives so lines don't appear variable thickness
        let stroke = egui::Stroke::new((2.0 * scale).max(1.0), device_border);

        if dev.outline.is_empty() {
            ui.painter_at(rect).rect_filled(body_rect, 0.0, device_fill);
            ui.painter_at(rect).rect_stroke(
                body_rect,
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
        } else {
            // Same overlap note as connector outline: shared edges (e.g. two rectangles, or rect + line) stroke twice and look thicker.
            let painter = ui.painter_at(rect);
            for prim in &dev.outline {
                match prim {
                    ConnectorOutline::Line { x1, y1, x2, y2 } => {
                        let dx = x2 - x1;
                        let dy = y2 - y1;
                        if dx * dx + dy * dy < 1e-12 {
                            continue;
                        }
                        let (gx1, gy1) = transform_pt(*x1, *y1);
                        let (gx2, gy2) = transform_pt(*x2, *y2);
                        painter.line_segment(
                            [to_screen(gx1, gy1), to_screen(gx2, gy2)],
                            stroke,
                        );
                    }
                    ConnectorOutline::Arc {
                        x: cx,
                        y: cy,
                        radius,
                        start_angle,
                        travel_angle,
                    } => {
                        if radius.abs() < 1e-9 || travel_angle.abs() < 1e-9 {
                            continue;
                        }
                        let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                        let mut points = Vec::with_capacity(n + 1);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + radius * angle_rad.cos();
                            let py = cy + radius * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        if points.len() >= 2 {
                            painter.add(egui::Shape::line(points, stroke));
                        }
                    }
                    ConnectorOutline::Polyline { points: pts } => {
                        if pts.len() >= 2 {
                            let screen_pts: Vec<egui::Pos2> = pts
                                .iter()
                                .map(|(px, py)| {
                                    let (gx, gy) = transform_pt(*px, *py);
                                    to_screen(gx, gy)
                                })
                                .collect();
                            painter.add(egui::Shape::line(screen_pts, stroke));
                        }
                    }
                    ConnectorOutline::Rectangle { x, y, width, height } => {
                        let (g0x, g0y) = transform_pt(*x, *y);
                        let (g1x, g1y) = transform_pt(x + width, *y);
                        let (g2x, g2y) = transform_pt(x + width, y + height);
                        let (g3x, g3y) = transform_pt(*x, y + height);
                        let min_x = g0x.min(g1x).min(g2x).min(g3x);
                        let max_x = g0x.max(g1x).max(g2x).max(g3x);
                        let min_y = g0y.min(g1y).min(g2y).min(g3y);
                        let max_y = g0y.max(g1y).max(g2y).max(g3y);
                        let r = egui::Rect::from_min_max(
                            to_screen(min_x, max_y),
                            to_screen(max_x, min_y),
                        );
                        // Outline only (no fill) so symbol rectangles are not filled
                        painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
                    }
                    ConnectorOutline::Circle { x, y, radius } => {
                        if radius.abs() < 1e-9 {
                            continue;
                        }
                        let (gx, gy) = transform_pt(*x, *y);
                        let center = to_screen(gx, gy);
                        let r_screen = (radius * scale as f64).max(2.0) as f32;
                        // Outline only (no fill) so symbol circles are not filled
                        painter.circle_stroke(center, r_screen, stroke);
                    }
                    ConnectorOutline::WidthArc {
                        x: cx,
                        y: cy,
                        radius,
                        start_angle,
                        travel_angle,
                        width,
                    } => {
                        let r_inner = radius;
                        let r_outer = radius + width;
                        let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                        let mut points = Vec::with_capacity(2 * n + 2);
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_outer * angle_rad.cos();
                            let py = cy + r_outer * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        for i in (0..=n).rev() {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_inner * angle_rad.cos();
                            let py = cy + r_inner * angle_rad.sin();
                            let (gx, gy) = transform_pt(px, py);
                            points.push(to_screen(gx, gy));
                        }
                        if points.len() >= 3 {
                            painter.add(egui::Shape::convex_polygon(
                                points.clone(),
                                device_fill,
                                stroke,
                            ));
                        }
                    }
                    ConnectorOutline::Polygon { points: pts } => {
                        if pts.len() >= 3 {
                            let screen_pts: Vec<egui::Pos2> = pts
                                .iter()
                                .map(|(px, py)| {
                                    let (gx, gy) = transform_pt(*px, *py);
                                    to_screen(gx, gy)
                                })
                                .collect();
                            painter.add(egui::Shape::convex_polygon(
                                screen_pts.clone(),
                                device_fill,
                                stroke,
                            ));
                        }
                    }
                    ConnectorOutline::Curve { points: pts } => {
                        if pts.len() >= 2 {
                            let screen_pts = catmull_rom_spline(pts, 8, |px, py| {
                                let (gx, gy) = transform_pt(px, py);
                                to_screen(gx, gy)
                            });
                            if screen_pts.len() >= 2 {
                                painter.add(egui::Shape::line(screen_pts, stroke));
                            }
                        }
                    }
                }
            }
        }

        // Device name label: use attributetext placement when available, else center above body
        let label = if !dev.connref.is_empty() {
            name_lookup.get(&dev.connref).cloned()
        } else {
            None
        }.or_else(|| if !dev.name.is_empty() { Some(dev.name.clone()) } else { None })
            .unwrap_or_else(|| format!("{} pins", dev.pins.len()));
        let (name_pos, align, font_size, rotation) = if let Some(ref pl) = dev.name_placement {
            let (gx, gy) = transform_pt(pl.x, pl.y);
            let pos = to_screen(gx, gy);
            let fs = (pl.height as f32 * scale).max(8.0);
            // Text rotation is in symbol-local (logical, Y-up) coords; negate for screen (Y-flip).
            let (rotation_deg, flipped) = normalize_label_rotation_deg(
                -(pl.rotation + transform_rotation_deg(&dev.transform)),
            );
            let hjust = if flipped { swap_hjust(pl.hjust) } else { pl.hjust };
            let align = text_align(hjust, pl.vjust);
            (pos, align, fs, rotation_deg)
        } else {
            let font_size = (actual_body_rect.height() * 0.4).max(8.0);
            let name_pos = egui::pos2(body_rect.center().x, body_rect.min.y - font_size * 0.3);
            (name_pos, egui::Align2::CENTER_BOTTOM, font_size, 0.0)
        };
        paint_text_with_style_and_rotation(
            &ui.painter_at(rect),
            name_pos,
            align,
            &label,
            font_size,
            device_border,
            dev.name_style,
            rotation,
        );

        for pin in &dev.pins {
            let (dx, dy) = transform_pt(pin.x, pin.y);
            let pos = to_screen(dx, dy);
            let pin_size = (6.0 * scale).max(3.0);
            let diamond: [egui::Pos2; 4] = [
                pos + egui::vec2(pin_size, 0.0),
                pos + egui::vec2(0.0, pin_size),
                pos + egui::vec2(-pin_size, 0.0),
                pos + egui::vec2(0.0, -pin_size),
            ];
            ui.painter_at(rect).add(egui::Shape::closed_line(
                diamond.to_vec(),
                egui::Stroke::new(1.5, device_border),
            ));
            let dev_rotation = transform_rotation_deg(&dev.transform);
            let (text_pos, text_align, font_size, rotation) =
                pin_text_pos(pin, &transform_pt, to_screen, scale, dev_rotation);
            // Pin names (only when attributetext visibility is not false)
            if pin.name_visible {
                paint_text_with_style_and_rotation(
                    &ui.painter_at(rect),
                    text_pos,
                    text_align,
                    &pin.text,
                    font_size,
                    theme.pin_name,
                    crate::model::TextStyle::Plain,
                    rotation,
                );
            }
            // Pin Short Descriptions
            if let Some(ref desc) = pin.short_description {
                if !desc.is_empty() && desc != &pin.text {
                    let (desc_pos, desc_align, desc_font_size, desc_rotation) =
                        pin_short_desc_pos(pin, &transform_pt, to_screen, scale, dev_rotation);
                    paint_text_with_style_and_rotation(
                        &ui.painter_at(rect),
                        desc_pos,
                        desc_align,
                        desc,
                        desc_font_size,
                        theme.pin_desc,
                        crate::model::TextStyle::Plain,
                        desc_rotation,
                    );
                }
            }
        }

        // Right-click context menu and click-to-select
        let dev_name = if dev.name.is_empty() {
            dev.connref.clone()
        } else {
            dev.name.clone()
        };
        let key = CrossRefKey("Device".to_string(), String::new(), dev_name.clone());
        let mut other_targets: Vec<(String, (f64, f64))> = cross_ref_map
            .get(&key)
            .map(|v| v.iter().map(|t| (t.0.clone(), t.1)).collect())
            .unwrap_or_default();
        other_targets.retain(|(dn, _)| current_diagram_name != Some(dn.as_str()));
        other_targets.sort_by(|a, b| a.0.cmp(&b.0));
        other_targets.dedup_by(|a, b| a.0 == b.0);
        let hit_rect = body_rect.expand(4.0);
        let response = ui.allocate_rect(hit_rect, egui::Sense::click());
        if response.clicked() {
            *selected_element.borrow_mut() = Some(("Device".to_string(), dev_name.clone()));
            expanded_elements.borrow_mut().insert(("Device".to_string(), dev_name.clone()));
        }
        if !other_targets.is_empty() {
            let targets = other_targets.clone();
            let pending = pending_nav.clone();
            response.context_menu(move |ui| {
                ui.label("Other instances:");
                for (diagram_name, _) in &targets {
                    if ui.button(format!("Diagram: {}", diagram_name)).clicked() {
                        *pending.borrow_mut() = Some(diagram_name.clone());
                        ui.close();
                    }
                }
            });
        }
        if let Some(ref sel) = *selected_element.borrow() {
            if sel.0 == "Device" && sel.1 == dev_name {
                ui.painter_at(rect).rect_stroke(
                    hit_rect,
                    0.0,
                    egui::Stroke::new(3.0, theme.highlight),
                    egui::StrokeKind::Outside,
                );
            }
        }
    }

    // Draw connector selection highlights on top (so they're not behind splices/devices)
    let highlight = egui::Stroke::new(3.0, theme.highlight);
    for r in &connector_selection_rects {
        ui.painter_at(rect).rect_stroke(*r, 0.0, highlight, egui::StrokeKind::Outside);
    }

    // Overlay text at top of schematic view: name, description, grid location of selected element
    let (overlay_name, overlay_desc, overlay_loc) = if let Some(ref sel) = *selected_element.borrow() {
        let (el_type, el_name) = sel;
        let logical_to_grid = |gx: f64, gy: f64| -> String {
            let col_idx = ((gx - grid_ox) / dx).floor().max(0.0) as usize;
            let row_idx = ((gy - grid_oy) / dy).floor().max(0.0) as usize;
            let col_num = (col_idx + 1).min(n_cols);
            format!("{}{}", index_to_alphanumeric(row_idx.min(n_rows)), col_num)
        };
        let (name, desc, loc) = match el_type.as_str() {
            "Device" => {
                let dev = content.devices.iter().find(|d| {
                    let n = if d.name.is_empty() { &d.connref } else { &d.name };
                    n == el_name
                });
                if let Some(dev) = dev {
                    let (gx, gy) = dev.bounds.map(|(a, b, c, d)| ((a + c) / 2.0, (b + d) / 2.0))
                        .unwrap_or_else(|| {
                            let m = &dev.transform;
                            let cx = dev.extent_x + dev.width / 2.0;
                            let cy = dev.extent_y + dev.height / 2.0;
                            (dev.x + m[0] * cx + m[2] * cy + m[4], dev.y + m[1] * cx + m[3] * cy + m[5])
                        });
                    let display_name = name_lookup.get(&dev.connref).cloned()
                        .filter(|s| !s.is_empty())
                        .or_else(|| if !dev.name.is_empty() { Some(dev.name.clone()) } else { None })
                        .unwrap_or_else(|| dev.connref.clone());
                    let desc = element_shortdescription.get(&dev.connref).cloned().unwrap_or_else(|| "—".to_string());
                    (display_name, desc, logical_to_grid(gx, gy))
                } else {
                    (el_name.clone(), "—".to_string(), "—".to_string())
                }
            }
            "Connector" => {
                let conn = content.connectors.iter().find(|c| {
                    let n = if c.name.is_empty() { &c.connref } else { &c.name };
                    n == el_name
                });
                if let Some(conn) = conn {
                    let (gx, gy) = conn.bounds.map(|(a, b, c, d)| ((a + c) / 2.0, (b + d) / 2.0))
                        .unwrap_or_else(|| {
                            let m = &conn.transform;
                            let cx = conn.extent_x + conn.width / 2.0;
                            let cy = conn.extent_y + conn.height / 2.0;
                            (conn.x + m[0] * cx + m[2] * cy + m[4], conn.y + m[1] * cx + m[3] * cy + m[5])
                        });
                    let display_name = name_lookup.get(&conn.connref).cloned()
                        .filter(|s| !s.is_empty())
                        .or_else(|| if !conn.name.is_empty() { Some(conn.name.clone()) } else { None })
                        .unwrap_or_else(|| conn.connref.clone());
                    let desc = element_shortdescription.get(&conn.connref).cloned().unwrap_or_else(|| "—".to_string());
                    (display_name, desc, logical_to_grid(gx, gy))
                } else {
                    (el_name.clone(), "—".to_string(), "—".to_string())
                }
            }
            "Splice" => {
                let splice = content.splices.iter().find(|s| {
                    let n = if s.name.is_empty() { &s.connref } else { &s.name };
                    n == el_name
                });
                if let Some(splice) = splice {
                    let m = &splice.transform;
                    let (gx, gy) = (splice.x + m[4], splice.y + m[5]);
                    let display_name = name_lookup.get(&splice.connref).cloned()
                        .filter(|s| !s.is_empty())
                        .or_else(|| if !splice.name.is_empty() { Some(splice.name.clone()) } else { None })
                        .unwrap_or_else(|| splice.connref.clone());
                    let desc = element_shortdescription.get(&splice.connref).cloned().unwrap_or_else(|| "—".to_string());
                    (display_name, desc, logical_to_grid(gx, gy))
                } else {
                    (el_name.clone(), "—".to_string(), "—".to_string())
                }
            }
            "Wire" => {
                let wire = content.wires.iter().find(|w| w.name == *el_name || w.id == *el_name);
                if let Some(wire) = wire {
                    let (gx, gy) = wire.segments.first()
                        .map(|s| ((s.x1 + s.x2) / 2.0, (s.y1 + s.y2) / 2.0))
                        .unwrap_or((0.0, 0.0));
                    let display_name = if !wire.name.is_empty() { wire.name.clone() } else { wire.id.clone() };
                    let desc = wire_shortdescription.get(&wire.id).cloned().unwrap_or_else(|| "—".to_string());
                    (display_name, desc, logical_to_grid(gx, gy))
                } else {
                    (el_name.clone(), "—".to_string(), "—".to_string())
                }
            }
            _ => (el_name.clone(), "—".to_string(), "—".to_string()),
        };
        (name, desc, loc)
    } else {
        ("—".to_string(), "—".to_string(), "—".to_string())
    };
    let overlay_lines = [
        format!("Name: {}", overlay_name),
        format!("Description: {}", overlay_desc),
        format!("Coordinates: {}", overlay_loc),
    ];
    let overlay_font = egui::FontId::proportional(14.0);
    let line_height = 18.0;
    let overlay_y = rect.min.y + 10.0;
    let overlay_x = rect.center().x;
    for (i, line) in overlay_lines.iter().enumerate() {
        ui.painter_at(rect).text(
            egui::pos2(overlay_x, overlay_y + (i as f32) * line_height),
            egui::Align2::CENTER_TOP,
            line.clone(),
            overlay_font.clone(),
            theme.overlay,
        );
    }

    // Debug overlay: connector, splice, and device count (top left corner)
    let nc = content.connectors.len();
    let ns = content.splices.len();
    let nd = content.devices.len();
    if nc > 0 || ns > 0 || nd > 0 {
        ui.painter_at(rect).text(
            rect.min + egui::vec2(8.0, 10.0),
            egui::Align2::LEFT_TOP,
            format!("Connectors: {}  Splices: {}  Devices: {}", nc, ns, nd),
            egui::FontId::proportional(12.0),
            theme.debug,
        );
    }
}

/// Computes position, alignment, font size, and rotation for pin name text.
/// `symbol_rotation_deg`: rotation of the symbol (device/connector) so pin text follows orientation.
fn pin_text_pos<F, G>(
    pin: &impl PinWithPlacement,
    transform_pt: &F,
    to_screen: G,
    scale: f32,
    symbol_rotation_deg: f64,
) -> (egui::Pos2, egui::Align2, f32, f64)
where
    F: Fn(f64, f64) -> (f64, f64),
    G: Fn(f64, f64) -> egui::Pos2,
{
    if let Some(ref pl) = pin.name_placement() {
        // When attributetext position is (0,0) it often means "unspecified" (e.g. multiple instances);
        // use default offset so text does not sit on top of the pin.
        let use_placement_pos = pl.x != 0.0 || pl.y != 0.0;
        if use_placement_pos {
            let (sx, sy) = pin_local_to_symbol_local(
                pin.pin_x(),
                pin.pin_y(),
                pin.pin_transform(),
                pl.x,
                pl.y,
            );
            let (gx, gy) = transform_pt(sx, sy);
            let pos = to_screen(gx, gy);
            let (rotation_deg, flipped) = normalize_label_rotation_deg(
                -(pl.rotation + symbol_rotation_deg),
            );
            let hjust = if flipped { swap_hjust(pl.hjust) } else { pl.hjust };
            let align = text_align(hjust, pl.vjust);
            let font_size = (pl.height as f32 * scale).max(8.0);
            (pos, align, font_size, rotation_deg)
        } else {
            default_pin_text_pos(pin, transform_pt, to_screen, scale)
        }
    } else {
        default_pin_text_pos(pin, transform_pt, to_screen, scale)
    }
}

fn default_pin_text_pos<F, G>(
    pin: &impl PinWithPlacement,
    transform_pt: &F,
    to_screen: G,
    scale: f32,
) -> (egui::Pos2, egui::Align2, f32, f64)
where
    F: Fn(f64, f64) -> (f64, f64),
    G: Fn(f64, f64) -> egui::Pos2,
{
    let (dx, dy) = transform_pt(pin.pin_x(), pin.pin_y());
    let pos_screen = to_screen(dx, dy);
    let pin_size = (6.0 * scale).max(3.0);
    let font_size = (1152.0_f64 * scale as f64).max(8.0) as f32;
    let text_pos = pos_screen + egui::vec2(pin_size + 2.0, -font_size * 0.5);
    (text_pos, egui::Align2::LEFT_CENTER, font_size, 0.0)
}

/// Computes position, alignment, font size, and rotation for pin short description.
/// `symbol_rotation_deg`: rotation of the symbol so short-desc text follows orientation.
fn pin_short_desc_pos<F, G>(
    pin: &impl PinWithPlacement,
    transform_pt: &F,
    to_screen: G,
    scale: f32,
    symbol_rotation_deg: f64,
) -> (egui::Pos2, egui::Align2, f32, f64)
where
    F: Fn(f64, f64) -> (f64, f64),
    G: Fn(f64, f64) -> egui::Pos2,
{
    if let Some(ref pl) = pin.short_description_placement() {
        let use_placement_pos = pl.x != 0.0 || pl.y != 0.0;
        if use_placement_pos {
            let (sx, sy) = pin_local_to_symbol_local(
                pin.pin_x(),
                pin.pin_y(),
                pin.pin_transform(),
                pl.x,
                pl.y,
            );
            let (gx, gy) = transform_pt(sx, sy);
            let pos = to_screen(gx, gy);
            let (rotation_deg, flipped) = normalize_label_rotation_deg(
                -(pl.rotation + symbol_rotation_deg),
            );
            let hjust = if flipped { swap_hjust(pl.hjust) } else { pl.hjust };
            let align = text_align(hjust, pl.vjust);
            let font_size = (pl.height as f32 * scale).max(6.0);
            (pos, align, font_size, rotation_deg)
        } else {
            default_pin_short_desc_pos(pin, transform_pt, to_screen, scale)
        }
    } else {
        default_pin_short_desc_pos(pin, transform_pt, to_screen, scale)
    }
}

fn default_pin_short_desc_pos<F, G>(
    pin: &impl PinWithPlacement,
    transform_pt: &F,
    to_screen: G,
    scale: f32,
) -> (egui::Pos2, egui::Align2, f32, f64)
where
    F: Fn(f64, f64) -> (f64, f64),
    G: Fn(f64, f64) -> egui::Pos2,
{
    let (dx, dy) = transform_pt(pin.pin_x(), pin.pin_y());
    let pos_screen = to_screen(dx, dy);
    let pin_size = (6.0 * scale).max(3.0);
    let font_size = (1152.0_f64 * scale as f64).max(8.0) as f32;
    let desc_font_size = (font_size * 0.85).max(6.0);
    let desc_pos = pos_screen + egui::vec2(pin_size + 2.0, font_size * 0.6);
    (desc_pos, egui::Align2::LEFT_CENTER, desc_font_size, 0.0)
}

trait PinWithPlacement {
    fn pin_x(&self) -> f64;
    fn pin_y(&self) -> f64;
    fn pin_transform(&self) -> &[f64; 6];
    fn name_placement(&self) -> Option<&crate::model::AttributeTextPlacement>;
    fn short_description_placement(&self) -> Option<&crate::model::AttributeTextPlacement>;
}

impl PinWithPlacement for ConnectorPin {
    fn pin_x(&self) -> f64 {
        self.x
    }
    fn pin_y(&self) -> f64 {
        self.y
    }
    fn pin_transform(&self) -> &[f64; 6] {
        &self.transform
    }
    fn name_placement(&self) -> Option<&crate::model::AttributeTextPlacement> {
        self.name_placement.as_ref()
    }
    fn short_description_placement(&self) -> Option<&crate::model::AttributeTextPlacement> {
        self.short_description_placement.as_ref()
    }
}

impl PinWithPlacement for DevicePin {
    fn pin_x(&self) -> f64 {
        self.x
    }
    fn pin_y(&self) -> f64 {
        self.y
    }
    fn pin_transform(&self) -> &[f64; 6] {
        &self.transform
    }
    fn name_placement(&self) -> Option<&crate::model::AttributeTextPlacement> {
        self.name_placement.as_ref()
    }
    fn short_description_placement(&self) -> Option<&crate::model::AttributeTextPlacement> {
        self.short_description_placement.as_ref()
    }
}

/// Converts pin-local (attributetext x,y) to symbol-local coordinates.
fn pin_local_to_symbol_local(
    pin_x: f64,
    pin_y: f64,
    pin_transform: &[f64; 6],
    pl_x: f64,
    pl_y: f64,
) -> (f64, f64) {
    let m = pin_transform;
    (
        pin_x + m[0] * pl_x + m[2] * pl_y + m[4],
        pin_y + m[1] * pl_x + m[3] * pl_y + m[5],
    )
}

/// Converts segment-local text position to global diagram coordinates.
/// Capital: horizontal (|dx|>=|dy|): x=along, y=perp. Vertical (|dy|>|dx|): x=perp, y=along (axes swapped).
/// Uses perpendicular (uy, -ux) = 90° clockwise from segment direction.
fn text_pos_to_global(seg: &crate::model::WireSegment, tx: f64, ty: f64) -> (f64, f64) {
    let dx = seg.x2 - seg.x1;
    let dy = seg.y2 - seg.y1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return (seg.x1 + tx, seg.y1 + ty);
    }
    let ux = dx / len;
    let uy = dy / len;
    let perp_x = uy;
    let perp_y = -ux;

    // Capital swaps axes when segment is more vertical than horizontal (|dy| > |dx|).
    // Previously used dx.abs() < 0.001*len which was too strict for near-vertical segments.
    let (along, perp) = if dy.abs() > dx.abs() {
        // Vertical-ish: Capital uses x=perp, y=along (axes swapped)
        (ty, tx)
    } else {
        // Horizontal-ish: x=along, y=perp
        (tx, ty)
    };

    let gx = seg.x1 + along * ux + perp * perp_x;
    let gy = seg.y1 + along * uy + perp * perp_y;
    (gx, gy)
}

/// Collects bounding points from outline primitives (for center computation).
fn outline_center_local(outline: &[ConnectorOutline]) -> (f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for prim in outline {
        match prim {
            ConnectorOutline::Line { x1, y1, x2, y2 } => {
                min_x = min_x.min(*x1).min(*x2);
                max_x = max_x.max(*x1).max(*x2);
                min_y = min_y.min(*y1).min(*y2);
                max_y = max_y.max(*y1).max(*y2);
            }
            ConnectorOutline::Arc { x, y, radius, .. } => {
                let r = radius.abs();
                min_x = min_x.min(x - r);
                max_x = max_x.max(x + r);
                min_y = min_y.min(y - r);
                max_y = max_y.max(y + r);
            }
            ConnectorOutline::Polyline { points } => {
                for (px, py) in points {
                    min_x = min_x.min(*px);
                    max_x = max_x.max(*px);
                    min_y = min_y.min(*py);
                    max_y = max_y.max(*py);
                }
            }
            _ => {}
        }
    }
    let cx = if min_x.is_finite() && max_x.is_finite() {
        (min_x + max_x) / 2.0
    } else {
        0.0
    };
    let cy = if min_y.is_finite() && max_y.is_finite() {
        (min_y + max_y) / 2.0
    } else {
        0.0
    };
    (cx, cy)
}

/// Paints schemindicators (oval, twist-commercial, shield) — stroke outline only.
fn paint_schemindicators(
    ui: &mut egui::Ui,
    indicators: &[Schemindicator],
    content: &DiagramContent,
    rect: egui::Rect,
    scale: f32,
    to_screen: &impl Fn(f64, f64) -> egui::Pos2,
    theme: &SchematicTheme,
) {
    let stroke_color = theme.schemindicator_stroke;
    let painter = ui.painter_at(rect);

    for ind in indicators {
        let m = &ind.transform;
        let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
            (
                ind.x + m[0] * lx + m[2] * ly + m[4],
                ind.y + m[1] * lx + m[3] * ly + m[5],
            )
        };

        let border_color = ind.attributeref.as_ref()
            .and_then(|id| content.datadictionary.get(id))
            .and_then(|dd| dd.color.as_ref())
            .and_then(|s| parse_color(s))
            .unwrap_or(stroke_color);
        let stroke = egui::Stroke::new((2.0 * scale).max(1.0), border_color);

        for prim in &ind.outline {
            match prim {
                ConnectorOutline::Line { x1, y1, x2, y2 } => {
                    let (gx1, gy1) = transform_pt(*x1, *y1);
                    let (gx2, gy2) = transform_pt(*x2, *y2);
                    painter.line_segment(
                        [to_screen(gx1, gy1), to_screen(gx2, gy2)],
                        stroke,
                    );
                }
                ConnectorOutline::Arc {
                    x: cx,
                    y: cy,
                    radius,
                    start_angle,
                    travel_angle,
                } => {
                    if radius.abs() < 1e-9 || travel_angle.abs() < 1e-9 {
                        continue;
                    }
                    let n = ((travel_angle.abs() / 5.0).ceil() as usize).max(2);
                    let mut points = Vec::with_capacity(n + 1);
                    for i in 0..=n {
                        let t = (i as f64) / (n as f64);
                        let angle_deg = start_angle + t * travel_angle;
                        let angle_rad = angle_deg * PI / 180.0;
                        let px = cx + radius * angle_rad.cos();
                        let py = cy + radius * angle_rad.sin();
                        let (gx, gy) = transform_pt(px, py);
                        points.push(to_screen(gx, gy));
                    }
                    if points.len() >= 2 {
                        painter.add(egui::Shape::line(points, stroke));
                    }
                }
                ConnectorOutline::Polyline { points: pts } => {
                    if pts.len() >= 2 {
                        let screen_pts: Vec<egui::Pos2> = pts
                            .iter()
                            .map(|(px, py)| {
                                let (gx, gy) = transform_pt(*px, *py);
                                to_screen(gx, gy)
                            })
                            .collect();
                        painter.add(egui::Shape::line(screen_pts, stroke));
                    }
                }
                _ => {}
            }
        }

        
    }
}

/// Paints text at pos with alignment, optionally applying TextStyle and rotation.
fn paint_text_with_style_and_rotation(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font_size: f32,
    color: egui::Color32,
    style: TextStyle,
    rotation_deg: f64,
) {
    let angle_rad = (rotation_deg as f32).to_radians();
    if angle_rad.abs() < 0.001 {
        paint_text_with_style(painter, pos, align, text, font_size, color, style);
        return;
    }
    let font_id = egui::FontId::proportional(font_size);
    let use_italics = matches!(style, TextStyle::Italic | TextStyle::BoldItalic);
    let galley = if use_italics {
        let format = egui::text::TextFormat {
            font_id: font_id.clone(),
            color,
            italics: true,
            ..Default::default()
        };
        let mut job = egui::text::LayoutJob::simple_format(text.to_owned(), format);
        job.wrap.max_width = f32::INFINITY;
        painter.layout_job(job)
    } else {
        painter.layout_no_wrap(text.to_owned(), font_id, color)
    };
    let text_rect = align.anchor_size(pos, galley.size());
    let text_shape = egui::epaint::TextShape::new(text_rect.min, galley, color)
        .with_override_text_color(color)
        .with_angle_and_anchor(angle_rad, align);
    painter.add(egui::Shape::Text(text_shape));
}

/// Paints wire segment cross-reference text as a link; clickable to jump to diagram.
/// Paints a backward double arrow at wire end (wire reference indicator).
/// Matches harness style: two chevrons side-by-side, pointing back along the wire.
/// `tip` = screen position of the wire end; `along_wire` = other end of segment.
fn paint_backward_double_arrow(
    painter: &egui::Painter,
    tip: egui::Pos2,
    along_wire: egui::Pos2,
    size: f32,
    stroke_width: f32,
    color: egui::Color32,
) {
    let dx = along_wire.x - tip.x;
    let dy = along_wire.y - tip.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-6 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let perp_x = -uy;
    let perp_y = ux;

    let wing = size * 0.5;
    let spacing = size * 0.35;
    let base_offset = -spacing; // push chevrons down from wire tip
    let stroke = egui::Stroke::new(stroke_width, color);

    // Two chevrons along the wire, offset down from the wire tip.
    // Chevron point at c; wings spread along the wire. Arrow points outward from wire end.
    for offset in [0.0, spacing] {
        let c = egui::Pos2::new(tip.x + ux * (base_offset + offset), tip.y + uy * (base_offset + offset));
        let left = egui::Pos2::new(c.x + ux * wing + perp_x * wing, c.y + uy * wing + perp_y * wing);
        let right = egui::Pos2::new(c.x + ux * wing - perp_x * wing, c.y + uy * wing - perp_y * wing);
        painter.line_segment([c, left], stroke);
        painter.line_segment([c, right], stroke);
    }
}

/// Paints wire text (wire name, gauge, color) with green color and rounded green border.
/// Uses standard rect_stroke with rounded corners for both horizontal and vertical text.
fn paint_wire_text_with_border(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font_size: f32,
    style: TextStyle,
    angle_rad: f32,
    color: egui::Color32,
) {
    let font_id = egui::FontId::proportional(font_size);
    let use_italics = matches!(style, TextStyle::Italic | TextStyle::BoldItalic);
    let galley = if use_italics {
        let format = TextFormat {
            font_id: font_id.clone(),
            color,
            italics: true,
            ..Default::default()
        };
        let mut job = egui::text::LayoutJob::simple_format(text.to_owned(), format);
        job.wrap.max_width = f32::INFINITY;
        painter.layout_job(job)
    } else {
        painter.layout_no_wrap(text.to_owned(), font_id, color)
    };
    let text_rect = align.anchor_size(pos, galley.size());
    let padding = (font_size * 0.15).max(2.0);
    let border_rect = text_rect.expand(padding);
    let corner_radius = egui::CornerRadius::same((font_size * 0.2).max(2.0).round() as u8);
    let stroke = egui::Stroke::new(1.5, color);

    let rect_to_draw = if angle_rad.abs() < 0.001 {
        border_rect
    } else {
        // AABB of rotated border_rect around anchor (pos)
        let cx = pos.x;
        let cy = pos.y;
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        let rotate = |p: egui::Pos2| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            egui::Pos2::new(cx + dx * c - dy * s, cy + dx * s + dy * c)
        };
        let corners = [
            rotate(border_rect.left_top()),
            rotate(border_rect.right_top()),
            rotate(border_rect.right_bottom()),
            rotate(border_rect.left_bottom()),
        ];
        let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        egui::Rect::from_min_max(egui::Pos2::new(min_x, min_y), egui::Pos2::new(max_x, max_y))
    };

    painter.rect_stroke(
        rect_to_draw,
        corner_radius,
        stroke,
        egui::StrokeKind::Outside,
    );

    if angle_rad.abs() < 0.001 {
        painter.galley(text_rect.min, galley, color);
    } else {
        let text_shape = egui::epaint::TextShape::new(text_rect.min, galley, color)
            .with_override_text_color(color)
            .with_angle_and_anchor(angle_rad, align);
        painter.add(egui::Shape::Text(text_shape));
    }
}

/// Paints text at pos with alignment, optionally applying TextStyle (italic).
fn paint_text_with_style(
    painter: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font_size: f32,
    color: egui::Color32,
    style: TextStyle,
) {
    let font_id = egui::FontId::proportional(font_size);
    let use_italics = matches!(style, TextStyle::Italic | TextStyle::BoldItalic);
    if use_italics {
        let format = TextFormat {
            font_id: font_id.clone(),
            color,
            italics: true,
            ..Default::default()
        };
        let mut job = egui::text::LayoutJob::simple_format(text.to_owned(), format);
        job.wrap.max_width = f32::INFINITY;
        let galley = painter.layout_job(job);
        let rect = align.anchor_size(pos, galley.size());
        painter.galley(rect.min, galley, color);
    } else {
        painter.text(pos, align, text.to_owned(), font_id, color);
    }
}

/// Rotation angle in degrees from a 2D affine transform matrix [m0, m1, m2, m3, m4, m5].
/// For a rotation, (m0, m1) is the image of (1,0), so angle = atan2(m1, m0).
fn transform_rotation_deg(m: &[f64; 6]) -> f64 {
    (m[1].atan2(m[0])).to_degrees()
}

/// Normalize label rotation so text is never upside down (avoid 180°). Keeps angle in (-90, 90].
/// Returns (normalized_angle, did_flip_180). When we flip by 180°, horizontal justification must be
/// swapped (left<->right) so the pivot stays correct (Capital anchor is in reading direction).
fn normalize_label_rotation_deg(mut r: f64) -> (f64, bool) {
    let mut flipped = false;
    while r > 90.0 {
        r -= 180.0;
        flipped = !flipped;
    }
    while r <= -90.0 {
        r += 180.0;
        flipped = !flipped;
    }
    (r, flipped)
}

fn swap_hjust(h: HorizontalJust) -> HorizontalJust {
    match h {
        HorizontalJust::Left => HorizontalJust::Right,
        HorizontalJust::Right => HorizontalJust::Left,
        HorizontalJust::Center => HorizontalJust::Center,
    }
}

/// Converts grid row index to alphanumeric label: 0 -> "A", 1 -> "B", ..., 25 -> "Z", 26 -> "AA", ...
fn index_to_alphanumeric(i: usize) -> String {
    let mut n = i;
    let mut s = String::new();
    loop {
        s.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    s
}

fn text_align(hjust: HorizontalJust, vjust: VerticalJust) -> egui::Align2 {
    let x = match hjust {
        HorizontalJust::Left => egui::Align::LEFT,
        HorizontalJust::Right => egui::Align::RIGHT,
        HorizontalJust::Center => egui::Align::Center,
    };
    let y = match vjust {
        VerticalJust::Top => egui::Align::TOP,
        VerticalJust::Bottom => egui::Align::BOTTOM,
        VerticalJust::Center => egui::Align::Center,
    };
    egui::Align2([x, y])
}

/// Returns (base_scale, offset_x, offset_y) for scale-to-fit. Used by paint_diagram and zoom-to-cursor.
pub fn scale_to_fit(
    logic_w: f64,
    logic_h: f64,
    screen_w: f32,
    screen_h: f32,
) -> (f32, f32, f32) {
    if logic_w <= 0.0 || logic_h <= 0.0 {
        return (1.0, 0.0, 0.0);
    }
    let scale_x = screen_w / (logic_w as f32);
    let scale_y = screen_h / (logic_h as f32);
    let scale = scale_x.min(scale_y);
    let offset_x = (screen_w - (logic_w as f32) * scale) * 0.5;
    let offset_y = (screen_h - (logic_h as f32) * scale) * 0.5;
    (scale, offset_x, offset_y)
}

/// Returns the logical (x, y) center of an element in diagram content, or None if not found.
/// Used to pan the schematic view to focus on a selected element.
pub fn element_center_in_diagram(
    content: &DiagramContent,
    element_type: &str,
    element_name: &str,
) -> Option<(f64, f64)> {
    match element_type {
        "Device" => {
            for dev in &content.devices {
                let name = if !dev.connref.is_empty() { &dev.connref } else { &dev.name };
                if name == element_name {
                    return dev.bounds
                        .map(|(a, b, c, d)| ((a + c) / 2.0, (b + d) / 2.0))
                        .or_else(|| {
                            let m = &dev.transform;
                            let cx = dev.extent_x + dev.width / 2.0;
                            let cy = dev.extent_y + dev.height / 2.0;
                            Some((dev.x + m[0] * cx + m[2] * cy + m[4], dev.y + m[1] * cx + m[3] * cy + m[5]))
                        });
                }
            }
        }
        "Connector" => {
            for conn in &content.connectors {
                let name = if !conn.connref.is_empty() { &conn.connref } else { &conn.name };
                if name == element_name {
                    return conn.bounds
                        .map(|(a, b, c, d)| ((a + c) / 2.0, (b + d) / 2.0))
                        .or_else(|| {
                            let m = &conn.transform;
                            let cx = conn.extent_x + conn.width / 2.0;
                            let cy = conn.extent_y + conn.height / 2.0;
                            Some((conn.x + m[0] * cx + m[2] * cy + m[4], conn.y + m[1] * cx + m[3] * cy + m[5]))
                        });
                }
            }
        }
        "Splice" => {
            for splice in &content.splices {
                let name = if !splice.connref.is_empty() { &splice.connref } else { &splice.name };
                if name == element_name {
                    let m = &splice.transform;
                    let (cx, cy) = (splice.x + m[4], splice.y + m[5]);
                    return Some((cx, cy));
                }
            }
        }
        "Wire" => {
            for wire in &content.wires {
                let matches = wire.name == element_name || wire.id == element_name;
                if matches && !wire.segments.is_empty() {
                    let mut sum_x = 0.0;
                    let mut sum_y = 0.0;
                    let mut n = 0;
                    for seg in &wire.segments {
                        sum_x += (seg.x1 + seg.x2) / 2.0;
                        sum_y += (seg.y1 + seg.y2) / 2.0;
                        n += 1;
                    }
                    if n > 0 {
                        return Some((sum_x / n as f64, sum_y / n as f64));
                    }
                }
            }
        }
        _ => {}
    }
    None
}

/// Computes pan offset to center the given element in the viewport.
/// Returns None if the element is not found in the diagram.
pub fn compute_pan_to_center_element(
    content: &DiagramContent,
    rect: egui::Rect,
    zoom: f32,
    element_type: &str,
    element_name: &str,
) -> Option<egui::Vec2> {
    element_center_in_diagram(content, element_type, element_name).and_then(|(gx, gy)| {
        page_extent(content).and_then(|(ox, oy, page_w, page_h)| {
            if page_w > 0.0 && page_h > 0.0 {
                let (base_scale, center_x, center_y) =
                    scale_to_fit(page_w, page_h, rect.width(), rect.height());
                let scale = base_scale * zoom;
                Some(egui::vec2(
                    rect.width() / 2.0 - (gx - ox) as f32 * scale - center_x,
                    rect.height() / 2.0 - (gy - oy) as f32 * scale - center_y,
                ))
            } else {
                None
            }
        })
    })
}

/// Hit-test: returns (element_type, name) if click_pos hits an element. Check order matches draw order (devices on top).
pub fn hit_test(
    content: &DiagramContent,
    rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
    _name_lookup: &HashMap<String, String>,
    click_pos: egui::Pos2,
) -> Option<(String, String)> {
    let (ox, oy, page_w, page_h) = match page_extent(content) {
        Some(ext) => ext,
        None => return None,
    };
    let (base_scale, offset_x, offset_y) = scale_to_fit(
        page_w,
        page_h,
        rect.width(),
        rect.height(),
    );
    let scale = base_scale * zoom;
    let offset_x = offset_x + pan.x;
    let offset_y = offset_y + pan.y;
    let to_screen = |x: f64, y: f64| {
        egui::Pos2::new(
            rect.min.x + offset_x + ((x - ox) as f32) * scale,
            rect.max.y - offset_y - ((y - oy) as f32) * scale,
        )
    };
    if !rect.contains(click_pos) {
        return None;
    }
    // Check in reverse draw order (devices on top, then splices, connectors, wires)
    for dev in &content.devices {
        let (min_x, min_y, max_x, max_y) = dev.bounds.unwrap_or_else(|| {
            let m = &dev.transform;
            let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
                (
                    dev.x + m[0] * lx + m[2] * ly + m[4],
                    dev.y + m[1] * lx + m[3] * ly + m[5],
                )
            };
            let (d0x, d0y) = transform_pt(dev.extent_x, dev.extent_y);
            let (d1x, d1y) = transform_pt(dev.extent_x + dev.width, dev.extent_y);
            let (d2x, d2y) = transform_pt(dev.extent_x + dev.width, dev.extent_y + dev.height);
            let (d3x, d3y) = transform_pt(dev.extent_x, dev.extent_y + dev.height);
            (
                d0x.min(d1x).min(d2x).min(d3x),
                d0y.min(d1y).min(d2y).min(d3y),
                d0x.max(d1x).max(d2x).max(d3x),
                d0y.max(d1y).max(d2y).max(d3y),
            )
        });
        let mut body_rect = egui::Rect::from_min_max(
            to_screen(min_x, max_y),
            to_screen(max_x, min_y),
        );
        let min_size = 20.0;
        if body_rect.width() < min_size || body_rect.height() < min_size {
            let c = body_rect.center();
            body_rect = egui::Rect::from_center_size(
                c,
                egui::vec2(body_rect.width().max(min_size), body_rect.height().max(min_size)),
            );
        }
        let body_rect = body_rect.expand(4.0);
        // Use connref for selection: it matches ElementRef.id in connectivity (name can differ).
        let dev_name = if !dev.connref.is_empty() {
            dev.connref.clone()
        } else {
            dev.name.clone()
        };
        if body_rect.contains(click_pos) {
            return Some(("Device".to_string(), dev_name));
        }
    }
    for splice in &content.splices {
        let m = &splice.transform;
        let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
            (
                splice.x + m[0] * lx + m[2] * ly + m[4],
                splice.y + m[1] * lx + m[3] * ly + m[5],
            )
        };
        let base_stroke_w = (2.0 * scale).max(1.0);
        let (cx, cy) = transform_pt(0.0, 0.0);
        let center = to_screen(cx, cy);
        let hit_size = (base_stroke_w * 8.0).max(16.0);
        let hit_rect = egui::Rect::from_center_size(center, egui::vec2(hit_size, hit_size));
        // Use connref for selection: it matches ElementRef.id in connectivity.
        let splice_name = if !splice.connref.is_empty() {
            splice.connref.clone()
        } else {
            splice.name.clone()
        };
        if hit_rect.contains(click_pos) {
            return Some(("Splice".to_string(), splice_name));
        }
    }
    for conn in &content.connectors {
        let (min_x, min_y, max_x, max_y) = conn.bounds.unwrap_or_else(|| {
            let m = &conn.transform;
            let transform_pt = |lx: f64, ly: f64| -> (f64, f64) {
                (
                    conn.x + m[0] * lx + m[2] * ly + m[4],
                    conn.y + m[1] * lx + m[3] * ly + m[5],
                )
            };
            let (d0x, d0y) = transform_pt(conn.extent_x, conn.extent_y);
            let (d1x, d1y) = transform_pt(conn.extent_x + conn.width, conn.extent_y);
            let (d2x, d2y) = transform_pt(conn.extent_x + conn.width, conn.extent_y + conn.height);
            let (d3x, d3y) = transform_pt(conn.extent_x, conn.extent_y + conn.height);
            (
                d0x.min(d1x).min(d2x).min(d3x),
                d0y.min(d1y).min(d2y).min(d3y),
                d0x.max(d1x).max(d2x).max(d3x),
                d0y.max(d1y).max(d2y).max(d3y),
            )
        });
        let mut body_rect = egui::Rect::from_min_max(
            to_screen(min_x, max_y),
            to_screen(max_x, min_y),
        );
        let min_size = 20.0;
        if body_rect.width() < min_size || body_rect.height() < min_size {
            let c = body_rect.center();
            body_rect = egui::Rect::from_center_size(
                c,
                egui::vec2(body_rect.width().max(min_size), body_rect.height().max(min_size)),
            );
        }
        let hit_rect = body_rect.expand(4.0);
        // Use connref for selection: it matches ElementRef.id in connectivity.
        let conn_name = if !conn.connref.is_empty() {
            conn.connref.clone()
        } else {
            conn.name.clone()
        };
        if hit_rect.contains(click_pos) {
            return Some(("Connector".to_string(), conn_name));
        }
    }
    for wire in &content.wires {
        for seg in &wire.segments {
            let thickness = seg
                .attributeref
                .as_ref()
                .and_then(|id| content.datadictionary.get(id))
                .and_then(|dd| dd.thickness)
                .unwrap_or(1.0) as f32 * scale.max(0.5);
            let from = to_screen(seg.x1, seg.y1);
            let to = to_screen(seg.x2, seg.y2);
            let seg_rect = egui::Rect::from_two_pos(from, to).expand((thickness * 2.0).max(8.0));
            if seg_rect.contains(click_pos) {
                // Use wire.id when set (matches WireRef.id in Elements view); else wire name for diagram-only wires.
                let sel_key = if !wire.id.is_empty() {
                    wire.id.clone()
                } else if !wire.name.is_empty() {
                    wire.name.clone()
                } else {
                    seg.text_items
                        .iter()
                        .find(|ti| ti.is_wire_name && !ti.text.is_empty())
                        .map(|ti| ti.text.clone())
                        .unwrap_or_default()
                };
                if !sel_key.is_empty() {
                    return Some(("Wire".to_string(), sel_key));
                }
            }
        }
    }
    None
}

/// Paints a filled polygon using triangulation. Handles concave and simple polygons correctly.
fn paint_polygon_fill(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    fill: egui::Color32,
    stroke: egui::Stroke,
) {
    if points.len() < 3 {
        return;
    }
    let flat: Vec<f64> = points
        .iter()
        .flat_map(|p| [p.x as f64, p.y as f64])
        .collect();
    let indices = match earcutr::earcut(&flat, &[], 2) {
        Ok(idx) => idx,
        Err(_) => return,
    };
    if indices.is_empty() {
        return;
    }
    let mut mesh = egui::Mesh::default();
    for p in points {
        mesh.colored_vertex(*p, fill);
    }
    for tri in indices.chunks_exact(3) {
        mesh.add_triangle(tri[0] as u32, tri[1] as u32, tri[2] as u32);
    }
    painter.add(egui::Shape::mesh(Arc::new(mesh)));
    if stroke != egui::Stroke::NONE {
        painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
    }
}

/// Builds polygon points for connector fill, following the exact outline shape.
/// Returns points in screen space. For empty outline: jack = 4 corners, plug = rounded rect sampled.
fn connector_fill_polygon<F1, F2>(
    conn: &ConnectorSymbol,
    body_rect: egui::Rect,
    transform_pt: &F1,
    to_screen: &F2,
) -> Vec<egui::Pos2>
where
    F1: Fn(f64, f64) -> (f64, f64),
    F2: Fn(f64, f64) -> egui::Pos2,
{
    let mut points: Vec<egui::Pos2> = Vec::new();
    let push_if_new = |pts: &mut Vec<egui::Pos2>, x: f64, y: f64| {
        let (gx, gy) = transform_pt(x, y);
        let p = to_screen(gx, gy);
        if pts.last().map_or(true, |q| q.distance(p) > 0.01) {
            pts.push(p);
        }
    };

    if conn.outline.is_empty() {
        // Fallback: extent rectangle. Plug = rounded corners; Jack = sharp corners.
        let r = body_rect.width().min(body_rect.height()) * 0.15;
        let rad = r.max(3.0);
        let left = body_rect.min.x;
        let right = body_rect.max.x;
        let top = body_rect.min.y;
        let bottom = body_rect.max.y;

        if matches!(conn.kind, ConnectorKind::Jack) || rad <= 0.5 {
            // Jack: simple rectangle (4 corners)
            points.push(egui::pos2(left, bottom));
            points.push(egui::pos2(right, bottom));
            points.push(egui::pos2(right, top));
            points.push(egui::pos2(left, top));
        } else {
            // Plug: rounded rectangle (sample quarter-circles at each corner)
            let n = 64usize; // segments per quarter circle for smooth, fully filled curves
            let rad_f = rad as f32;
            // BL corner: center (left+rad, bottom+rad), arc 180° -> 270°
            for i in 0..=n {
                let a = 180.0 + (i as f64 / n as f64) * 90.0;
                let ar = a * PI / 180.0;
                let (c, s) = (ar.cos() as f32, ar.sin() as f32);
                points.push(egui::pos2(left + rad_f - rad_f * c, bottom + rad_f + rad_f * s));
            }
            // BR corner
            for i in 1..=n {
                let a = 270.0 + (i as f64 / n as f64) * 90.0;
                let ar = a * PI / 180.0;
                let (c, s) = (ar.cos() as f32, ar.sin() as f32);
                points.push(egui::pos2(right - rad_f + rad_f * c, bottom + rad_f + rad_f * s));
            }
            // TR corner
            for i in 1..=n {
                let a = (i as f64 / n as f64) * 90.0;
                let ar = a * PI / 180.0;
                let (c, s) = (ar.cos() as f32, ar.sin() as f32);
                points.push(egui::pos2(right - rad_f + rad_f * c, top - rad_f - rad_f * s));
            }
            // TL corner
            for i in 1..=n {
                let a = 90.0 + (i as f64 / n as f64) * 90.0;
                let ar = a * PI / 180.0;
                let (c, s) = (ar.cos() as f32, ar.sin() as f32);
                points.push(egui::pos2(left + rad_f - rad_f * c, top - rad_f - rad_f * s));
            }
        }
        return points;
    }

    // Use only the primary fill-defining primitive to avoid overlapping segments and
    // self-intersecting polygons (e.g. Rectangle + Circle for pin holes).
    // When outline contains Arc or WidthArc, traverse ALL primitives so rounded arcs are included.
    // Priority: Arc/WidthArc path > Rectangle (+ Line corner cuts) > Polygon > Polyline > Circle.
    let has_rect = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Rectangle { .. }));
    let has_polygon = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Polygon { .. }));
    let has_polyline = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Polyline { .. }));
    let has_circle = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Circle { .. }));
    let has_lines = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Line { .. }));
    let has_arc = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::Arc { .. }));
    let has_width_arc = conn.outline.iter().any(|p| matches!(p, ConnectorOutline::WidthArc { .. }));

    let use_outline_path = has_arc || has_width_arc;

    if use_outline_path {
        // Build fill polygon by traversing outline primitives in boundary order.
        // XML groups by type (Line, Arc, Polyline) but the path must follow the closed boundary.
        // For shapes with polyline + line + 2 arcs (rounded corners): Polyline -> Arc(bottom) -> Line -> Arc(top).
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
                        // Outer arc
                        for i in 0..=n {
                            let t = (i as f64) / (n as f64);
                            let angle_deg = start_angle + t * travel_angle;
                            let angle_rad = angle_deg * PI / 180.0;
                            let px = cx + r_outer * angle_rad.cos();
                            let py = cy + r_outer * angle_rad.sin();
                            push_if_new(&mut points, px, py);
                        }
                        // Inner arc (reverse direction to close the pie wedge)
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
                        let sampled = catmull_rom_spline(pts, 8, |px, py| {
                            let (gx, gy) = transform_pt(px, py);
                            to_screen(gx, gy)
                        });
                        for p in sampled {
                            if points.last().map_or(true, |q| q.distance(p) > 0.01) {
                                points.push(p);
                            }
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
                        // Plug: rounded polygon with arc points at corners (and at cut corners)
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
                        // Jack: sharp corners, optional line cuts
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
            // Outline has only Lines/Curves (no arcs) — use extent rect
            push_if_new(&mut points, conn.extent_x, conn.extent_y);
            push_if_new(&mut points, conn.extent_x + conn.width, conn.extent_y);
            push_if_new(&mut points, conn.extent_x + conn.width, conn.extent_y + conn.height);
            push_if_new(&mut points, conn.extent_x, conn.extent_y + conn.height);
        }
    }

    // Close polygon if not closed
    if points.len() >= 3 {
        let first = points[0];
        let last = points[points.len() - 1];
        if first.distance(last) > 0.5 {
            points.push(first);
        }
    }

    // egui convex_polygon prefers clockwise winding for best rendering
    ensure_clockwise_winding(&mut points);

    points
}

/// Builds a rounded polygon for Plug connectors. Rounds all corners; at cut corners,
/// inserts arc points between the two line endpoints. Uses more points for smooth curves.
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
        (x + round_rad, y + round_rad),       // TL
        (x2 - round_rad, y + round_rad),      // TR
        (x2 - round_rad, y2 - round_rad),     // BR
        (x + round_rad, y2 - round_rad),     // BL
    ];
    // Arc angle ranges (degrees): start and sweep for each corner, CW order
    let arc_ranges: [(f64, f64); 4] = [
        (180.0, 90.0),   // TL: 180° -> 270°
        (270.0, 90.0),   // TR: 270° -> 0°
        (0.0, 90.0),     // BR: 0° -> 90°
        (90.0, 90.0),    // BL: 90° -> 180°
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
            // Cut corner: arc from first to second around the corner vertex
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
            // Full quarter circle
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

/// Applies Line corner cuts to a rectangle. Each line with both endpoints on adjacent
/// rect edges replaces that corner with the two line endpoints. Returns the polygon vertices.
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
        (Edge::Top, Edge::Left),     // TL
        (Edge::Top, Edge::Right),    // TR
        (Edge::Right, Edge::Bottom), // BR
        (Edge::Bottom, Edge::Left),  // BL
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
        // Find which corner this line cuts
        for (i, (c1, c2)) in corners.iter().enumerate() {
            if (e1 == *c1 && e2 == *c2) || (e1 == *c2 && e2 == *c1) {
                // Replace corner i with the two line endpoints in CW order
                let (p1, p2) = ((*x1, *y1), (*x2l, *y2l));
                // Order: going CW, the point on the first edge of the corner comes first
                let (first, second) = match i {
                    0 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (y1 - y).abs() < (y2l - y).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else {
                            if (x1 - x).abs() < (x2l - x).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        }
                    }
                    1 => {
                        if e1 == Edge::Top || e2 == Edge::Top {
                            if (x1 - x2).abs() < (x2l - x2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else {
                            if (y1 - y).abs() < (y2l - y).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        }
                    }
                    2 => {
                        if e1 == Edge::Right || e2 == Edge::Right {
                            if (y1 - y2).abs() < (y2l - y2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else {
                            if (x1 - x2).abs() < (x2l - x2).abs() {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        }
                    }
                    3 => {
                        // Bottom-left: bottom edge point (larger x) comes first, then left edge
                        if e1 == Edge::Bottom || e2 == Edge::Bottom {
                            if x1 > x2l {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
                        } else {
                            if y1 > y2l {
                                (p1, p2)
                            } else {
                                (p2, p1)
                            }
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

/// Ensures polygon vertices are in clockwise order (egui's preferred winding).
/// Uses signed area: in Y-down screen coords, CW = positive area.
fn ensure_clockwise_winding(points: &mut [egui::Pos2]) {
    if points.len() < 3 {
        return;
    }
    let mut area = 0.0;
    let n = points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    if area < 0.0 {
        points.reverse();
    }
}

/// Samples a Catmull-Rom spline through points, producing smooth curve segments.
fn catmull_rom_spline<F>(
    points: &[(f64, f64)],
    segments_per_edge: usize,
    mut to_screen: F,
) -> Vec<egui::Pos2>
where
    F: FnMut(f64, f64) -> egui::Pos2,
{
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
                continue; // avoid duplicate at segment boundary
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
            result.push(to_screen(x, y));
        }
    }
    result
}

fn parse_color(s: &str) -> Option<egui::Color32> {
    let s = s.trim();
    if s.starts_with('#') && s.len() >= 7 {
        let r = u8::from_str_radix(&s[1..3], 16).ok()?;
        let g = u8::from_str_radix(&s[3..5], 16).ok()?;
        let b = u8::from_str_radix(&s[5..7], 16).ok()?;
        return Some(egui::Color32::from_rgb(r, g, b));
    }
    match s.to_lowercase().as_str() {
        "black" => Some(egui::Color32::BLACK),
        "red" => Some(egui::Color32::from_rgb(255, 0, 0)),
        "blue" => Some(egui::Color32::from_rgb(0, 0, 255)),
        "green" => Some(egui::Color32::from_rgb(0, 128, 0)),
        "white" => Some(egui::Color32::WHITE),
        "yellow" => Some(egui::Color32::from_rgb(255, 255, 0)),
        "orange" => Some(egui::Color32::from_rgb(255, 165, 0)),
        "gray" | "grey" => Some(egui::Color32::from_rgb(128, 128, 128)),
        "purple" | "violet" => Some(egui::Color32::from_rgb(128, 0, 128)),
        "brown" => Some(egui::Color32::from_rgb(139, 69, 19)),
        "cyan" => Some(egui::Color32::from_rgb(0, 255, 255)),
        "magenta" => Some(egui::Color32::from_rgb(255, 0, 255)),
        "grid" => Some(egui::Color32::from_rgb(200, 200, 200)),
        "net" => Some(egui::Color32::from_rgb(100, 100, 100)),
        _ => None,
    }
}
