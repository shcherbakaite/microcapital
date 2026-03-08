use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::capital_xml::{load_diagram_content, parse_project};
use crate::model::{Design, DiagramContent, ElementRef, CrossRefKey, CrossRefMap, WireRef};
use crate::schematic_view;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Diagrams,
    Elements,
    Schematic,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Diagrams => "Diagrams",
            Tab::Elements => "Elements",
            Tab::Schematic => "Schematic",
        }
    }
}

/// Shared state used by TabViewer to read/write selection and load diagram content.
pub struct AppState {
    pub design: Option<Design>,
    pub xml_path: Option<PathBuf>,
    pub selected_diagram_id: Option<String>,
    pub selected_diagram_name: Option<String>,
    pub diagram_cache: HashMap<String, DiagramContent>,
    pub load_error: Option<String>,
    /// Schematic view: pan offset (pixels), zoom factor (1.0 = fit).
    pub schematic_pan: egui::Vec2,
    pub schematic_zoom: f32,
    /// Filter text for diagram list (space-separated words; schematic must contain at least one).
    pub diagram_filter: String,
    /// Filter text for element list (space-separated words; element name must contain at least one).
    pub element_filter: String,
    /// Pending navigation from right-click "go to reference": diagram name to open (persists across frames).
    pub pending_nav: std::rc::Rc<std::cell::RefCell<Option<String>>>,
    /// Empty cross-ref map used when no design is loaded.
    pub empty_cross_ref_map: CrossRefMap,
    /// Selected element from schematic click: (element_type, name) e.g. ("Device", "K7").
    pub selected_schematic_element: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>,
    /// Element headers expanded in Elements view (e.g. when selected from schematic).
    pub expanded_elements: std::rc::Rc<std::cell::RefCell<HashSet<(String, String)>>>,
    /// Last selection we scrolled to; when selection differs, we scroll once (avoids snapping back every frame).
    pub last_scrolled_to: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>,
    /// When set from Elements view selection, schematic will pan to center this element on next paint.
    pub pending_focus_element: std::rc::Rc<std::cell::RefCell<Option<(String, String)>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            design: None,
            xml_path: None,
            selected_diagram_id: None,
            selected_diagram_name: None,
            diagram_cache: HashMap::new(),
            load_error: None,
            schematic_pan: egui::Vec2::ZERO,
            schematic_zoom: 1.0,
            diagram_filter: String::new(),
            element_filter: String::new(),
            pending_nav: std::rc::Rc::new(std::cell::RefCell::new(None)),
            empty_cross_ref_map: CrossRefMap::default(),
            selected_schematic_element: std::rc::Rc::new(std::cell::RefCell::new(None)),
            expanded_elements: std::rc::Rc::new(std::cell::RefCell::new(HashSet::new())),
            last_scrolled_to: std::rc::Rc::new(std::cell::RefCell::new(None)),
            pending_focus_element: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }
}

impl AppState {
    pub fn ensure_diagram_loaded(&mut self) {
        let Some(path) = &self.xml_path else { return };
        let Some(id) = &self.selected_diagram_id else { return };
        if self.diagram_cache.contains_key(id) {
            return;
        }
        let path = path.clone();
        let id = id.clone();
        let empty_pin = HashMap::new();
        let empty_schempin = HashMap::new();
        let empty_device_pins: HashMap<String, HashMap<String, String>> = HashMap::new();
        let (device_pins, pin_sd, schempin_map) = self.design.as_ref()
            .map(|d| (&d.device_pins, &d.pin_shortdescription, &d.schempin_to_logicpin))
            .unwrap_or((&empty_device_pins, &empty_pin, &empty_schempin));
        match std::fs::File::open(&path) {
            Ok(f) => {
                let reader = std::io::BufReader::new(f);
                match load_diagram_content(reader, &id, device_pins, pin_sd, schempin_map) {
                    Ok(content) => {
                        self.diagram_cache.insert(id, content);
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Diagram load: {}", e));
                    }
                }
            }
            Err(e) => {
                self.load_error = Some(format!("Open file: {}", e));
            }
        }
    }
}

struct Viewer {
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
}

impl TabViewer for Viewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    /// Schematic tab needs pan/zoom, so disable dock scrollbars ([false, false]).
    /// List tabs use their own ScrollArea (scrollbar on the right); disable dock scrollbars
    /// to avoid scrollbar appearing in the middle.
    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let mut state = self.state.borrow_mut();
        match tab {
            Tab::Diagrams => {
                let filter_words: Vec<String> = state.diagram_filter
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                let diagram_list: Vec<_> = state.design.as_ref()
                    .map(|d| {
                        d.diagrams
                            .iter()
                            .filter(|x| x.visible)
                            .filter(|x| {
                                filter_words.is_empty()
                                    || filter_words.iter().any(|w| x.name.to_lowercase().contains(w))
                            })
                            .map(|x| (x.id.clone(), x.name.clone()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let mut selected_id = state.selected_diagram_id.clone();
                let mut selected_name = state.selected_diagram_name.clone();
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.diagram_filter)
                            .desired_width(ui.available_width()),
                    );
                });
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false]) // Keep full width, don't shrink
                    .show(ui, |ui| {
                    if diagram_list.is_empty() {
                        ui.label("Open an XML file to see diagrams.");
                        return;
                    }
                    for (id, name) in &diagram_list {
                        let is_sel = selected_id.as_deref() == Some(id.as_str());
                        if ui.selectable_label(is_sel, name).clicked() {
                            selected_id = Some(id.clone());
                            selected_name = Some(name.clone());
                        }
                    }
                });
                let selection_changed = selected_id.as_deref() != state.selected_diagram_id.as_deref();
                state.selected_diagram_id = selected_id.clone();
                state.selected_diagram_name = selected_name;
                if selection_changed && selected_id.is_some() {
                    state.load_error = None;
                    state.schematic_pan = egui::Vec2::ZERO;
                    state.schematic_zoom = 1.0;
                }
            }
            Tab::Elements => {
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.element_filter)
                            .desired_width(ui.available_width()),
                    );
                });
                let (devices, splices, connectors, wires, diagram_lookup, cross_ref_map): (
                    Vec<_>, Vec<_>, Vec<_>, Vec<_>, HashMap<String, String>, &CrossRefMap,
                ) = state.design.as_ref()
                    .map(|d| {
                        let lookup: HashMap<_, _> =
                            d.diagrams.iter().map(|x| (x.name.clone(), x.id.clone())).collect();
                        (
                            d.connectivity.devices.clone(),
                            d.connectivity.splices.clone(),
                            d.connectivity.connectors.clone(),
                            d.connectivity.wires.clone(),
                            lookup,
                            &d.cross_ref_map,
                        )
                    })
                    .unwrap_or_else(|| {
                        (vec![], vec![], vec![], vec![], HashMap::new(), &state.empty_cross_ref_map)
                    });
                let element_filter_words: Vec<String> = state.element_filter
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                let matches_filter = |name: &str, id: &str| {
                    element_filter_words.is_empty()
                        || element_filter_words.iter().any(|w| {
                            name.to_lowercase().contains(w) || id.to_lowercase().contains(w)
                        })
                };
                let filtered_devices: Vec<_> = devices.iter()
                    .filter(|el| matches_filter(&el.name, &el.id))
                    .collect();
                let filtered_splices: Vec<_> = splices.iter()
                    .filter(|el| matches_filter(&el.name, &el.id))
                    .collect();
                let filtered_connectors: Vec<_> = connectors.iter()
                    .filter(|el| matches_filter(&el.name, &el.id))
                    .collect();
                let filtered_wires: Vec<_> = wires.iter()
                    .filter(|w| matches_filter(&w.name, &w.id))
                    .collect();
                let mut pending_diagram_name: Option<String> = None;
                egui::ScrollArea::vertical()
                    .id_salt("elements_list")
                    .auto_shrink([false, true]) // Keep full width so scrollbar stays on the right
                    .show(ui, |ui| {
                    if devices.is_empty() && splices.is_empty() && connectors.is_empty() && wires.is_empty() {
                        ui.label("Open an XML file to see elements.");
                        return;
                    }
                    if filtered_devices.is_empty() && filtered_splices.is_empty()
                        && filtered_connectors.is_empty() && filtered_wires.is_empty()
                    {
                        ui.label("No elements match the filter.");
                        return;
                    }
                    /// Look up diagram instances for an element. Tries all name alternatives (name, id)
                    /// since cross_ref_map may be keyed by either; merges and deduplicates results.
                    fn diagram_instances(
                        cross_ref_map: &CrossRefMap,
                        element_type: &str,
                        name_alternatives: &[String],
                        fallback_diagram: &str,
                    ) -> Vec<String> {
                        let mut diagrams: Vec<String> = Vec::new();
                        for name in name_alternatives {
                            if name.is_empty() {
                                continue;
                            }
                            let key = CrossRefKey(element_type.to_string(), String::new(), name.clone());
                            if let Some(targets) = cross_ref_map.get(&key) {
                                for t in targets {
                                    diagrams.push(t.0.clone());
                                }
                            }
                        }
                        diagrams.sort();
                        diagrams.dedup();
                        if diagrams.is_empty() && !fallback_diagram.is_empty() {
                            diagrams.push(fallback_diagram.to_string());
                        }
                        diagrams
                    }
                    let sel = state.selected_schematic_element.borrow().clone();
                    let need_scroll = sel.as_ref().map(|s| *state.last_scrolled_to.borrow() != Some(s.clone())).unwrap_or_else(|| state.last_scrolled_to.borrow().is_some());
                    // When selected element is in this category, force open. Otherwise None so user can toggle.
                    let category_open = |t: &str| {
                        if sel.as_ref().map(|(ty, _)| ty == t).unwrap_or(false) {
                            Some(true)
                        } else {
                            None
                        }
                    };
                    // Selection key: use id when available (matches schematic connref); else name.
                    let sel_key = |el: &ElementRef| {
                        if !el.id.is_empty() { el.id.clone() } else { el.name.clone() }
                    };
                    let wire_sel_key = |w: &WireRef| {
                        if !w.id.is_empty() { w.id.clone() } else { w.name.clone() }
                    };
                    if egui::CollapsingHeader::new(format!("Devices ({})", filtered_devices.len()))
                        .open(category_open("Device"))
                        .show(ui, |ui| {
                        for el in &filtered_devices {
                            let key = sel_key(el);
                            let name_alt = [el.name.clone(), el.id.clone()];
                            let diagrams = diagram_instances(cross_ref_map, "Device", &name_alt, &el.diagram_name);
                            let is_selected = sel.as_ref().map(|(t, n)| t == "Device" && (n == &el.name || n == &el.id)).unwrap_or(false);
                            let default_open = state.expanded_elements.borrow().contains(&("Device".to_string(), key.clone())) || is_selected;
                            let header_id = ("element", "Device", el.name.as_str(), default_open);
                            let resp = egui::CollapsingHeader::new(&el.name)
                                .id_salt(header_id)
                                .default_open(default_open)
                                .show(ui, |ui| {
                                for dn in &diagrams {
                                    let row_selected = is_selected
                                        && state.selected_diagram_name.as_deref() == Some(dn.as_str());
                                    let row_resp = ui.selectable_label(row_selected, format!("  {}", dn));
                                    if row_selected && need_scroll {
                                        row_resp.scroll_to_me(Some(egui::Align::Center));
                                        *state.last_scrolled_to.borrow_mut() = sel.clone();
                                    }
                                    if row_resp.clicked() {
                                        pending_diagram_name = Some(dn.clone());
                                        *state.selected_schematic_element.borrow_mut() =
                                            Some(("Device".to_string(), key.clone()));
                                        state.expanded_elements.borrow_mut().insert(("Device".to_string(), key.clone()));
                                        *state.pending_focus_element.borrow_mut() =
                                            Some(("Device".to_string(), key.clone()));
                                    }
                                }
                            });
                            if resp.header_response.clicked() {
                                *state.selected_schematic_element.borrow_mut() =
                                    Some(("Device".to_string(), key.clone()));
                                state.expanded_elements.borrow_mut().insert(("Device".to_string(), key.clone()));
                                *state.pending_focus_element.borrow_mut() =
                                    Some(("Device".to_string(), key.clone()));
                                if diagrams.len() == 1 {
                                    if let Some(dn) = diagrams.first() {
                                        pending_diagram_name = Some(dn.clone());
                                    }
                                }
                            }
                            if is_selected && need_scroll {
                                resp.header_response.scroll_to_me(Some(egui::Align::Center));
                                *state.last_scrolled_to.borrow_mut() = sel.clone();
                            }
                        }
                    }).body_response.is_none() {}
                    if egui::CollapsingHeader::new(format!("Splices ({})", filtered_splices.len()))
                        .open(category_open("Splice"))
                        .show(ui, |ui| {
                        for el in &filtered_splices {
                            let key = sel_key(el);
                            let name_alt = [el.name.clone(), el.id.clone()];
                            let diagrams = diagram_instances(cross_ref_map, "Splice", &name_alt, &el.diagram_name);
                            let is_selected = sel.as_ref().map(|(t, n)| t == "Splice" && (n == &el.name || n == &el.id)).unwrap_or(false);
                            let default_open = state.expanded_elements.borrow().contains(&("Splice".to_string(), key.clone())) || is_selected;
                            let header_id = ("element", "Splice", el.name.as_str(), default_open);
                            let resp = egui::CollapsingHeader::new(&el.name)
                                .id_salt(header_id)
                                .default_open(default_open)
                                .show(ui, |ui| {
                                for dn in &diagrams {
                                    let row_selected = is_selected
                                        && state.selected_diagram_name.as_deref() == Some(dn.as_str());
                                    let row_resp = ui.selectable_label(row_selected, format!("  {}", dn));
                                    if row_selected && need_scroll {
                                        row_resp.scroll_to_me(Some(egui::Align::Center));
                                        *state.last_scrolled_to.borrow_mut() = sel.clone();
                                    }
                                    if row_resp.clicked() {
                                        pending_diagram_name = Some(dn.clone());
                                        *state.selected_schematic_element.borrow_mut() =
                                            Some(("Splice".to_string(), key.clone()));
                                        state.expanded_elements.borrow_mut().insert(("Splice".to_string(), key.clone()));
                                        *state.pending_focus_element.borrow_mut() =
                                            Some(("Splice".to_string(), key.clone()));
                                    }
                                }
                            });
                            if resp.header_response.clicked() {
                                *state.selected_schematic_element.borrow_mut() =
                                    Some(("Splice".to_string(), key.clone()));
                                state.expanded_elements.borrow_mut().insert(("Splice".to_string(), key.clone()));
                                *state.pending_focus_element.borrow_mut() =
                                    Some(("Splice".to_string(), key.clone()));
                                if diagrams.len() == 1 {
                                    if let Some(dn) = diagrams.first() {
                                        pending_diagram_name = Some(dn.clone());
                                    }
                                }
                            }
                            if is_selected && need_scroll {
                                resp.header_response.scroll_to_me(Some(egui::Align::Center));
                                *state.last_scrolled_to.borrow_mut() = sel.clone();
                            }
                        }
                    }).body_response.is_none() {}
                    if egui::CollapsingHeader::new(format!("Connectors ({})", filtered_connectors.len()))
                        .open(category_open("Connector"))
                        .show(ui, |ui| {
                        for el in &filtered_connectors {
                            let key = sel_key(el);
                            let name_alt = [el.name.clone(), el.id.clone()];
                            let diagrams = diagram_instances(cross_ref_map, "Connector", &name_alt, &el.diagram_name);
                            let is_selected = sel.as_ref().map(|(t, n)| t == "Connector" && (n == &el.name || n == &el.id)).unwrap_or(false);
                            let default_open = state.expanded_elements.borrow().contains(&("Connector".to_string(), key.clone())) || is_selected;
                            let header_id = ("element", "Connector", el.name.as_str(), default_open);
                            let resp = egui::CollapsingHeader::new(&el.name)
                                .id_salt(header_id)
                                .default_open(default_open)
                                .show(ui, |ui| {
                                for dn in &diagrams {
                                    let row_selected = is_selected
                                        && state.selected_diagram_name.as_deref() == Some(dn.as_str());
                                    let row_resp = ui.selectable_label(row_selected, format!("  {}", dn));
                                    if row_selected && need_scroll {
                                        row_resp.scroll_to_me(Some(egui::Align::Center));
                                        *state.last_scrolled_to.borrow_mut() = sel.clone();
                                    }
                                    if row_resp.clicked() {
                                        pending_diagram_name = Some(dn.clone());
                                        *state.selected_schematic_element.borrow_mut() =
                                            Some(("Connector".to_string(), key.clone()));
                                        state.expanded_elements.borrow_mut().insert(("Connector".to_string(), key.clone()));
                                        *state.pending_focus_element.borrow_mut() =
                                            Some(("Connector".to_string(), key.clone()));
                                    }
                                }
                            });
                            if resp.header_response.clicked() {
                                *state.selected_schematic_element.borrow_mut() =
                                    Some(("Connector".to_string(), key.clone()));
                                state.expanded_elements.borrow_mut().insert(("Connector".to_string(), key.clone()));
                                *state.pending_focus_element.borrow_mut() =
                                    Some(("Connector".to_string(), key.clone()));
                                if diagrams.len() == 1 {
                                    if let Some(dn) = diagrams.first() {
                                        pending_diagram_name = Some(dn.clone());
                                    }
                                }
                            }
                            if is_selected && need_scroll {
                                resp.header_response.scroll_to_me(Some(egui::Align::Center));
                                *state.last_scrolled_to.borrow_mut() = sel.clone();
                            }
                        }
                    }).body_response.is_none() {}
                    if egui::CollapsingHeader::new(format!("Wires ({})", filtered_wires.len()))
                        .open(category_open("Wire"))
                        .show(ui, |ui| {
                        for w in &filtered_wires {
                            let key = wire_sel_key(w);
                            let name_alt = [w.name.clone(), w.id.clone()];
                            let diagrams = diagram_instances(cross_ref_map, "Wire", &name_alt, &w.diagram_name);
                            let is_selected = sel.as_ref().map(|(t, n)| t == "Wire" && (n == &w.name || n == &w.id)).unwrap_or(false);
                            let default_open = {
                                let exp = state.expanded_elements.borrow();
                                exp.contains(&("Wire".to_string(), key.clone()))
                                    || is_selected
                            };
                            let header_id = ("element", "Wire", w.name.as_str(), default_open);
                            let resp = egui::CollapsingHeader::new(&w.name)
                                .id_salt(header_id)
                                .default_open(default_open)
                                .show(ui, |ui| {
                                for dn in &diagrams {
                                    let row_selected = is_selected
                                        && state.selected_diagram_name.as_deref() == Some(dn.as_str());
                                    let row_resp = ui.selectable_label(row_selected, format!("  {}", dn));
                                    if row_selected && need_scroll {
                                        row_resp.scroll_to_me(Some(egui::Align::Center));
                                        *state.last_scrolled_to.borrow_mut() = sel.clone();
                                    }
                                    if row_resp.clicked() {
                                        pending_diagram_name = Some(dn.clone());
                                        *state.selected_schematic_element.borrow_mut() =
                                            Some(("Wire".to_string(), key.clone()));
                                        state.expanded_elements.borrow_mut().insert(("Wire".to_string(), key.clone()));
                                        *state.pending_focus_element.borrow_mut() =
                                            Some(("Wire".to_string(), key.clone()));
                                    }
                                }
                            });
                            if resp.header_response.clicked() {
                                *state.selected_schematic_element.borrow_mut() =
                                    Some(("Wire".to_string(), key.clone()));
                                state.expanded_elements.borrow_mut().insert(("Wire".to_string(), key.clone()));
                                *state.pending_focus_element.borrow_mut() =
                                    Some(("Wire".to_string(), key.clone()));
                                if diagrams.len() == 1 {
                                    if let Some(dn) = diagrams.first() {
                                        pending_diagram_name = Some(dn.clone());
                                    }
                                }
                            }
                            if is_selected && need_scroll {
                                resp.header_response.scroll_to_me(Some(egui::Align::Center));
                                *state.last_scrolled_to.borrow_mut() = sel.clone();
                            }
                        }
                    }).body_response.is_none() {}
                });
                if state.selected_schematic_element.borrow().is_none() {
                    *state.last_scrolled_to.borrow_mut() = None;
                }
                if let Some(name) = pending_diagram_name {
                    state.selected_diagram_name = Some(name.clone());
                    state.selected_diagram_id = diagram_lookup.get(&name).cloned();
                    state.load_error = None;
                    state.schematic_pan = egui::Vec2::ZERO;
                    state.schematic_zoom = 1.0;
                }
            }
            Tab::Schematic => {
                state.ensure_diagram_loaded();
                if let Some(err) = &state.load_error {
                    ui.colored_label(egui::Color32::RED, err);
                }
                let rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

                if let Some(id) = &state.selected_diagram_id {
                    if let Some(content) = state.diagram_cache.get(id).cloned() {
                        let name_lookup: HashMap<String, String> = state.design.as_ref()
                            .map(|d| {
                                let mut m = HashMap::new();
                                for el in &d.connectivity.devices {
                                    m.insert(el.id.clone(), el.name.clone());
                                }
                                for el in &d.connectivity.splices {
                                    m.insert(el.id.clone(), el.name.clone());
                                }
                                for el in &d.connectivity.connectors {
                                    m.insert(el.id.clone(), el.name.clone());
                                }
                                m
                            })
                            .unwrap_or_default();
                        // Focus on selected element (from Elements view or right-click "go to reference")
                        let zoom = state.schematic_zoom;
                        let focus_pan = state.pending_focus_element.borrow_mut().take().and_then(
                            |(el_type, el_name)| {
                                schematic_view::compute_pan_to_center_element(
                                    &content,
                                    rect,
                                    zoom,
                                    &el_type,
                                    &el_name,
                                )
                            },
                        );
                        if let Some(pan) = focus_pan {
                            state.schematic_pan = pan;
                        }
                        // Pan: drag (flip Y so drag-up moves content up)
                        if response.dragged() {
                            let delta = ui.input(|i| i.pointer.delta());
                            state.schematic_pan += egui::vec2(delta.x, -delta.y);
                            ui.ctx().request_repaint();
                        }
                        // Cursor feedback for pan
                        let response = response.on_hover_cursor(egui::CursorIcon::Grab);
                        // Zoom: scroll wheel (centered on cursor)
                        if response.hovered() {
                            let scroll = ui.input(|i| i.raw_scroll_delta.y);
                            if scroll != 0.0 {
                                let factor = 1.0 + scroll * 0.005;
                                let zoom_new = (state.schematic_zoom * factor).clamp(0.1, 20.0);
                                if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                                    if let Some((_ox, _oy, page_w, page_h)) =
                                        schematic_view::page_extent(&content)
                                    {
                                        if page_w > 0.0 && page_h > 0.0 {
                                            let (base_scale, center_x, center_y) =
                                                schematic_view::scale_to_fit(
                                                    page_w,
                                                    page_h,
                                                    rect.width(),
                                                    rect.height(),
                                                );
                                            let cursor_local_x = cursor.x - rect.min.x;
                                            let cursor_local_y = rect.max.y - cursor.y;
                                            let scale_old = base_scale * state.schematic_zoom;
                                            let scale_new = base_scale * zoom_new;
                                            let zoom_factor = scale_new / scale_old;
                                            state.schematic_pan.x = cursor_local_x - center_x
                                                - (cursor_local_x - center_x - state.schematic_pan.x)
                                                    * zoom_factor;
                                            state.schematic_pan.y = cursor_local_y - center_y
                                                - (cursor_local_y - center_y - state.schematic_pan.y)
                                                    * zoom_factor;
                                        }
                                    }
                                }
                                state.schematic_zoom = zoom_new;
                                ui.ctx().request_repaint();
                            }
                        }
                        // Zoom: Ctrl+Plus / Ctrl+Minus (centered on cursor when schematic is hovered)
                        if response.hovered() && ui.input(|i| i.modifiers.ctrl) {
                            let zoom_in = ui.input(|i| i.key_pressed(egui::Key::Plus))
                                || ui.input(|i| i.key_pressed(egui::Key::Equals));
                            let zoom_out = ui.input(|i| i.key_pressed(egui::Key::Minus));
                            let zoom_factor = if zoom_in {
                                1.2
                            } else if zoom_out {
                                1.0 / 1.2
                            } else {
                                1.0
                            };
                            if zoom_factor != 1.0 {
                                let zoom_new = if zoom_in {
                                    (state.schematic_zoom * 1.2).min(20.0)
                                } else {
                                    (state.schematic_zoom / 1.2).max(0.1)
                                };
                                if let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) {
                                    if let Some((_ox, _oy, page_w, page_h)) =
                                        schematic_view::page_extent(&content)
                                    {
                                        if page_w > 0.0 && page_h > 0.0 {
                                            let (base_scale, center_x, center_y) =
                                                schematic_view::scale_to_fit(
                                                    page_w,
                                                    page_h,
                                                    rect.width(),
                                                    rect.height(),
                                                );
                                            let cursor_local_x = cursor.x - rect.min.x;
                                            let cursor_local_y = rect.max.y - cursor.y;
                                            let scale_old = base_scale * state.schematic_zoom;
                                            let scale_new = base_scale * zoom_new;
                                            let factor = scale_new / scale_old;
                                            state.schematic_pan.x = cursor_local_x - center_x
                                                - (cursor_local_x - center_x - state.schematic_pan.x)
                                                    * factor;
                                            state.schematic_pan.y = cursor_local_y - center_y
                                                - (cursor_local_y - center_y - state.schematic_pan.y)
                                                    * factor;
                                        }
                                    }
                                }
                                state.schematic_zoom = zoom_new;
                                ui.ctx().request_repaint();
                            }
                        }

                        let cross_ref_map = state
                            .design
                            .as_ref()
                            .map(|d| &d.cross_ref_map)
                            .unwrap_or(&state.empty_cross_ref_map);
                        let empty_device_pins: HashMap<String, HashMap<String, String>> = HashMap::new();
                        let empty_element_sd: HashMap<String, String> = HashMap::new();
                        let empty_wire_sd: HashMap<String, String> = HashMap::new();
                        let device_pins = state.design.as_ref()
                            .map(|d| &d.device_pins)
                            .unwrap_or(&empty_device_pins);
                        let element_shortdescription = state.design.as_ref()
                            .map(|d| &d.element_shortdescription)
                            .unwrap_or(&empty_element_sd);
                        let wire_shortdescription = state.design.as_ref()
                            .map(|d| &d.wire_shortdescription)
                            .unwrap_or(&empty_wire_sd);
                        schematic_view::paint_diagram(
                            &content,
                            ui,
                            rect,
                            state.schematic_pan,
                            state.schematic_zoom,
                            &name_lookup,
                            cross_ref_map,
                            state.selected_diagram_name.as_deref(),
                            state.pending_nav.clone(),
                            state.selected_schematic_element.clone(),
                            state.expanded_elements.clone(),
                            device_pins,
                            element_shortdescription,
                            wire_shortdescription,
                        );
                        // Manual hit-test on click: select element or clear (bypasses egui allocation order)
                        if response.clicked() && !response.dragged() {
                            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                                match schematic_view::hit_test(
                                    &content,
                                    rect,
                                    state.schematic_pan,
                                    state.schematic_zoom,
                                    &name_lookup,
                                    pointer_pos,
                                ) {
                                    Some((el_type, el_name)) => {
                                        // Use hit_test result directly; Elements list matches both name and id
                                        *state.selected_schematic_element.borrow_mut() =
                                            Some((el_type.clone(), el_name.clone()));
                                        state.expanded_elements.borrow_mut().insert((el_type, el_name));
                                    }
                                    None => {
                                        *state.selected_schematic_element.borrow_mut() = None;
                                    }
                                }
                            }
                        }
                        let diagram_name = state.pending_nav.borrow_mut().take();
                        if let Some(diagram_name) = diagram_name {
                            state.selected_diagram_name = Some(diagram_name.clone());
                            state.selected_diagram_id = state
                                .design
                                .as_ref()
                                .and_then(|d| {
                                    d.diagrams
                                        .iter()
                                        .find(|x| x.name == diagram_name)
                                        .map(|x| x.id.clone())
                                });
                            state.load_error = None;
                            state.schematic_pan = egui::Vec2::ZERO;
                            state.schematic_zoom = 1.0;
                            // Focus on selected element when diagram loads (same as Elements view)
                            if let Some(sel) = state.selected_schematic_element.borrow().clone() {
                                *state.pending_focus_element.borrow_mut() = Some(sel);
                            }
                        }
                    } else {
                        ui.label("Loading diagram…");
                    }
                } else {
                    ui.label("Select a diagram from the list.");
                }
            }
        }
    }
}

/// Base egui application state and UI.
pub struct MicroCapitalApp {
    dock_state: DockState<Tab>,
    state: std::rc::Rc<std::cell::RefCell<AppState>>,
    /// true = dark mode, false = light mode
    dark_mode: bool,
    /// About dialog visible
    about_open: bool,
}

impl Default for MicroCapitalApp {
    fn default() -> Self {
        Self::new(false)
    }
}

impl MicroCapitalApp {
    /// Create app with optional dark mode. Use `from_storage` when loading from eframe.
    pub fn new(dark_mode: bool) -> Self {
        // Layout: left = Elements (top) + Diagrams (bottom), right = Schematic.
        // Left panel gets ~25% width so schematic has most of the space.
        let mut dock_state = DockState::new(vec![Tab::Schematic]);
        let tree = dock_state.main_surface_mut();
        let [_, diagrams_node] = tree.split_left(NodeIndex::root(), 0.25, vec![Tab::Diagrams]);
        let [_, _] = tree.split_below(diagrams_node, 0.33, vec![Tab::Elements]);

        Self {
            dock_state,
            state: std::rc::Rc::new(std::cell::RefCell::new(AppState::default())),
            dark_mode,
            about_open: false,
        }
    }

    /// Create app, loading dark mode preference from storage if available.
    pub fn from_storage(storage: Option<&dyn eframe::Storage>) -> Self {
        let dark_mode = storage
            .and_then(|s| s.get_string("microcapital.dark_mode"))
            .map(|s| s == "true")
            .unwrap_or(false);
        Self::new(dark_mode)
    }
}

impl eframe::App for MicroCapitalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme before any UI is drawn
        ctx.set_visuals(if self.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // Top menu: File -> Open
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open XML…").clicked() {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Capital XML", &["xml"])
                            .pick_file()
                        {
                            match std::fs::File::open(&path) {
                                Ok(f) => {
                                    let reader = std::io::BufReader::new(f);
                                    match parse_project(reader) {
                                        Ok((design, diagram_contents)) => {
                                            let mut state = self.state.borrow_mut();
                                            state.design = Some(design);
                                            state.diagram_cache = diagram_contents;
                                            state.xml_path = Some(path);
                                            state.selected_diagram_id = None;
                                            state.selected_diagram_name = None;
                                            state.load_error = None;
                                            *state.pending_focus_element.borrow_mut() = None;
                                        }
                                        Err(e) => {
                                            self.state.borrow_mut().load_error = Some(e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.state.borrow_mut().load_error = Some(e.to_string());
                                }
                            }
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.selectable_label(!self.dark_mode, "Light Mode").clicked() {
                        self.dark_mode = false;
                        ui.close();
                    }
                    if ui.selectable_label(self.dark_mode, "Dark Mode").clicked() {
                        self.dark_mode = true;
                        ui.close();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.about_open = true;
                        ui.close();
                    }
                });
            });
        });

        // About dialog
        let mut about_open = self.about_open;
        let mut request_close = false;
        egui::Window::new("About MicroCapital")
            .open(&mut about_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("MicroCapital");
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(8.0);
                ui.label("Open-source viewer for Capital XML files.");
                ui.label("https://github.com/vshcherbakov/microcapital");
                ui.add_space(8.0);
                ui.label("Copyright © 2025 Vladislav Shcherbakov");
                ui.add_space(8.0);
                ui.label("This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.");
                ui.add_space(4.0);
                ui.label("This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.");
                ui.add_space(8.0);
                if ui.button("Close").clicked() {
                    request_close = true;
                }
            });
        if request_close {
            about_open = false;
        }
        self.about_open = about_open;

        // Dock area fills the rest
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = Viewer {
                state: self.state.clone(),
            };
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut viewer);
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            "microcapital.dark_mode",
            if self.dark_mode { "true" } else { "false" }.to_string(),
        );
    }
}
