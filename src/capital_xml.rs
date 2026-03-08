//! Parser for Capital Essentials project XML using hard_xml.
//! Parses design metadata, connectivity index, and diagram content in one pass.

mod diagram_content;

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Read};

use hard_xml::XmlRead;

use crate::model::{
    BorderBounds, ConnectorKind, CrossRefKey, CrossRefTarget, Design, DiagramContent, DiagramInfo,
    ElementRef, WireRef,
};

pub use diagram_content::convert_diagram_content;

// ---------------------------------------------------------------------------
// hard_xml struct definitions for Capital Essentials XML schema
// ---------------------------------------------------------------------------

#[derive(Debug, XmlRead)]
#[xml(tag = "project")]
struct XmlProject {
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(child = "designmgr")]
    designmgr: Option<XmlDesignmgr>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "designmgr")]
struct XmlDesignmgr {
    #[xml(child = "logicaldesign")]
    logicaldesign: Option<XmlLogicaldesign>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "logicaldesign")]
struct XmlLogicaldesign {
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(child = "designsharedusagemgr")]
    designsharedusagemgr: Vec<XmlDesignsharedusagemgr>,
    #[xml(child = "designwideusagemgr")]
    designwideusagemgr: Vec<XmlDesignwideusagemgr>,
    #[xml(child = "connectivity")]
    connectivity: Vec<XmlConnectivity>,
    #[xml(child = "designsharedpinusage")]
    designsharedpinusage: Vec<XmlDesignsharedpinusage>,
    #[xml(child = "diagram")]
    diagram: Vec<XmlDiagram>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "designsharedusagemgr")]
struct XmlDesignsharedusagemgr {
    #[xml(child = "diagramsharedusagemgr")]
    diagramsharedusagemgr: Vec<XmlDiagramsharedusagemgr>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "designwideusagemgr")]
struct XmlDesignwideusagemgr {
    #[xml(child = "diagramsharedusagemgr")]
    diagramsharedusagemgr: Vec<XmlDiagramsharedusagemgr>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "diagramsharedusagemgr")]
struct XmlDiagramsharedusagemgr {
    #[xml(attr = "diagramid")]
    diagramid: Option<String>,
    #[xml(attr = "diagramname")]
    diagramname: Option<String>,
    #[xml(child = "designsharedconductorusage")]
    designsharedconductorusage: Vec<XmlDesignsharedconductorusage>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "designsharedconductorusage")]
struct XmlDesignsharedconductorusage {
    #[xml(attr = "harnwire")]
    harnwire: Option<String>,
    #[xml(attr = "diagramname")]
    diagramname: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "connectivity")]
struct XmlConnectivity {
    #[xml(child = "device")]
    device: Vec<XmlDevice>,
    #[xml(child = "grounddevice")]
    grounddevice: Vec<XmlGrounddevice>,
    #[xml(child = "splice")]
    splice: Vec<XmlSplice>,
    #[xml(child = "connector")]
    connector: Vec<XmlConnector>,
    #[xml(child = "deviceconnector")]
    deviceconnector: Vec<XmlDeviceconnector>,
    #[xml(child = "pin")]
    pin: Vec<XmlPin>,
    #[xml(child = "wire")]
    wire: Vec<XmlWire>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "wire")]
struct XmlWire {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "internalpin")]
struct XmlInternalpin {
    #[xml(attr = "id")]
    id: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "device")]
struct XmlDevice {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
    #[xml(attr = "diagram")]
    diagram: Option<String>,
    #[xml(attr = "typecode")]
    typecode: Option<String>,
    #[xml(child = "pin")]
    pin: Vec<XmlPin>,
    #[xml(child = "internalpin")]
    internalpin: Vec<XmlInternalpin>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "grounddevice")]
struct XmlGrounddevice {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
    #[xml(attr = "diagram")]
    diagram: Option<String>,
    #[xml(attr = "typecode")]
    typecode: Option<String>,
    #[xml(child = "pin")]
    pin: Vec<XmlPin>,
    #[xml(child = "internalpin")]
    internalpin: Vec<XmlInternalpin>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "splice")]
struct XmlSplice {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
    #[xml(attr = "diagram")]
    diagram: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "connector")]
struct XmlConnector {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
    #[xml(attr = "diagram")]
    diagram: Option<String>,
    #[xml(attr = "type")]
    type_: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "deviceconnector")]
struct XmlDeviceconnector {
    #[xml(child = "pin")]
    pin: Vec<XmlPin>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "pin")]
struct XmlPin {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "shortdescription")]
    shortdescription: Option<String>,
    #[xml(attr = "connectedpin")]
    connectedpin: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "designsharedpinusage")]
struct XmlDesignsharedpinusage {
    #[xml(attr = "diagram")]
    diagram: Option<String>,
    #[xml(attr = "schempin")]
    schempin: Option<String>,
    #[xml(attr = "logicpin")]
    logicpin: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "diagram")]
struct XmlDiagram {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(child = "property")]
    property: Vec<XmlProperty>,
    #[xml(child = "diagramcontent")]
    diagramcontent: Vec<XmlDiagramcontent>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "property")]
struct XmlProperty {
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "val")]
    val: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "diagramcontent")]
pub(crate) struct XmlDiagramcontent {
    #[xml(attr = "diagramid")]
    diagramid: Option<String>,
    #[xml(child = "border")]
    border: Vec<XmlBorder>,
    #[xml(child = "datadictionary")]
    datadictionary: Vec<XmlDatadictionary>,
    #[xml(child = "schemwire")]
    schemwire: Vec<diagram_content::XmlSchemwire>,
    #[xml(child = "schemdevice")]
    schemdevice: Vec<diagram_content::XmlSchemdevice>,
    #[xml(child = "schemgrounddevice")]
    schemgrounddevice: Vec<diagram_content::XmlSchemgrounddevice>,
    #[xml(child = "schemconnector")]
    schemconnector: Vec<diagram_content::XmlSchemconnector>,
    #[xml(child = "schemsplice")]
    schemsplice: Vec<diagram_content::XmlSchemsplice>,
    #[xml(child = "schemindicator")]
    schemindicator: Vec<diagram_content::XmlSchemindicator>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "border")]
struct XmlBorder {
    #[xml(attr = "paperx")]
    paperx: Option<f64>,
    #[xml(attr = "papery")]
    papery: Option<f64>,
    #[xml(attr = "paperwidth")]
    paperwidth: Option<f64>,
    #[xml(attr = "paperheight")]
    paperheight: Option<f64>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "width")]
    width: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
    #[xml(attr = "units")]
    units: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "datadictionary")]
struct XmlDatadictionary {
    #[xml(child = "ddattribute")]
    ddattribute: Vec<XmlDdattribute>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "ddattribute")]
struct XmlDdattribute {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "thickness")]
    thickness: Option<String>,
    #[xml(attr = "linestyle")]
    linestyle: Option<String>,
    #[xml(child = "color")]
    color: Vec<XmlColor>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "color")]
struct XmlColor {
    #[xml(attr = "value")]
    value: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn decode_entity(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Parses the project XML and returns design metadata: diagram list and connectivity index.
/// Does not load per-diagram graphics; use `parse_project` for full parse including diagram content.
pub fn parse_design<R: BufRead>(reader: R) -> Result<Design, String> {
    let (design, _) = parse_project(reader)?;
    Ok(design)
}

/// Parses the full project XML once and returns both Design and all diagram content.
pub fn parse_project<R: BufRead>(reader: R) -> Result<(Design, HashMap<String, DiagramContent>), String> {
    let mut xml = String::new();
    reader
        .take(100 * 1024 * 1024) // 100MB limit
        .read_to_string(&mut xml)
        .map_err(|e| format!("Read error: {}", e))?;

    let project: XmlProject = XmlProject::from_str(&xml)
        .map_err(|e| format!("XML parse error: {}", e))?;

    let designmgr = project.designmgr.as_ref().ok_or("No designmgr")?;
    let logicaldesign = designmgr
        .logicaldesign
        .as_ref()
        .ok_or("No logicaldesign")?;

    let mut design = Design::default();

    // Project name from project or logicaldesign
    if let Some(name) = project.name.as_ref().or(logicaldesign.name.as_ref()) {
        design.project_name = decode_entity(name);
    }

    // Build wire_to_diagram from designsharedconductorusage (harnwire -> diagramname)
    let mut wire_to_diagram: HashMap<String, String> = HashMap::new();

    // Build diagram list and wire_to_diagram from designsharedusagemgr and designwideusagemgr
    let mut diagram_visible: HashMap<String, bool> = HashMap::new();
    let mut seen_diagram_ids: HashSet<String> = HashSet::new();

    for dsu in logicaldesign
        .designsharedusagemgr
        .iter()
        .flat_map(|m| m.diagramsharedusagemgr.iter())
        .chain(
            logicaldesign
                .designwideusagemgr
                .iter()
                .flat_map(|m| m.diagramsharedusagemgr.iter()),
        )
    {
        let diagramid = dsu.diagramid.clone().unwrap_or_default();
        let diagramname = dsu
            .diagramname
            .as_ref()
            .map(|n| decode_entity(n))
            .unwrap_or_default();
        if !diagramid.is_empty() && seen_diagram_ids.insert(diagramid.clone()) {
            design.diagrams.push(DiagramInfo {
                id: diagramid.clone(),
                name: diagramname.clone(),
                visible: true,
            });
        }
        // Always collect wires from designsharedconductorusage; diagramsharedusagemgr
        // in designwideusagemgr have the conductor data (designsharedusagemgr ones are often empty).
        for c in &dsu.designsharedconductorusage {
            if let Some(hw) = &c.harnwire {
                let diagram_name = c
                    .diagramname
                    .as_ref()
                    .map(|n| decode_entity(n))
                    .unwrap_or_else(|| diagramname.clone());
                wire_to_diagram.insert(hw.clone(), diagram_name);
            }
        }
    }

    // Apply diagram VISIBLE property from diagram elements
    for diag in &logicaldesign.diagram {
        if let Some(id) = &diag.id {
            let mut visible = true;
            for prop in &diag.property {
                if prop
                    .name
                    .as_ref()
                    .map(|n| n.eq_ignore_ascii_case("VISIBLE"))
                    == Some(true)
                {
                    if let Some(v) = &prop.val {
                        visible = v.trim().eq_ignore_ascii_case("true");
                    }
                    break;
                }
            }
            diagram_visible.insert(id.clone(), visible);
        }
    }

    for diag in design.diagrams.iter_mut() {
        if let Some(&visible) = diagram_visible.get(&diag.id) {
            diag.visible = visible;
        }
    }

    // Build wire list from wire_to_diagram (will enrich names from diagram content later)
    for (id, diagram_name) in wire_to_diagram {
        design.connectivity.wires.push(WireRef {
            id: id.clone(),
            name: format!("Wire {}", id),
            diagram_name,
        });
    }

    // Build connectivity from device, splice, connector, pin
    for conn in &logicaldesign.connectivity {
        for dev in &conn.device {
            if let (Some(id), Some(name), Some(diagram)) =
                (&dev.id, &dev.name, &dev.diagram)
            {
                if let Some(sd) = &dev.shortdescription {
                    if !sd.is_empty() {
                        design.element_shortdescription.insert(id.clone(), decode_entity(sd));
                    }
                }
                design.connectivity.devices.push(ElementRef {
                    id: id.clone(),
                    name: decode_entity(name),
                    diagram_name: decode_entity(diagram),
                    type_label: dev.typecode.as_ref().map(|t| decode_entity(t)),
                });
            }
        }
        for gd in &conn.grounddevice {
            if let Some(id) = &gd.id {
                let name = gd.name.as_ref().map(|n| decode_entity(n)).unwrap_or_else(|| id.clone());
                let diagram = gd.diagram.as_ref().map(|d| decode_entity(d)).unwrap_or_default();
                if let Some(sd) = &gd.shortdescription {
                    if !sd.is_empty() {
                        design.element_shortdescription.insert(id.clone(), decode_entity(sd));
                    }
                }
                design.connectivity.devices.push(ElementRef {
                    id: id.clone(),
                    name,
                    diagram_name: diagram,
                    type_label: gd.typecode.as_ref().map(|t| decode_entity(t)),
                });
            }
        }
        for sp in &conn.splice {
            if let (Some(id), Some(name)) = (&sp.id, &sp.name) {
                if let Some(sd) = &sp.shortdescription {
                    if !sd.is_empty() {
                        design.element_shortdescription.insert(id.clone(), decode_entity(sd));
                    }
                }
                design.connectivity.splices.push(ElementRef {
                    id: id.clone(),
                    name: decode_entity(name),
                    diagram_name: sp.diagram.as_ref().map(|d| decode_entity(d)).unwrap_or_default(),
                    type_label: None,
                });
            }
        }
        for c in &conn.connector {
            if let (Some(id), Some(name)) = (&c.id, &c.name) {
                if let Some(sd) = &c.shortdescription {
                    if !sd.is_empty() {
                        design.element_shortdescription.insert(id.clone(), decode_entity(sd));
                    }
                }
                design.connectivity.connectors.push(ElementRef {
                    id: id.clone(),
                    name: decode_entity(name),
                    diagram_name: c.diagram.as_ref().map(|d| decode_entity(d)).unwrap_or_default(),
                    type_label: None,
                });
                // type="plug" | type="jack" from <connector> tag
                if let Some(t) = &c.type_ {
                    let kind = if t.eq_ignore_ascii_case("plug") {
                        ConnectorKind::Plug
                    } else {
                        ConnectorKind::Jack
                    };
                    design.connector_type.insert(id.clone(), kind);
                }
            }
        }
        for w in &conn.wire {
            if let (Some(id), Some(sd)) = (&w.id, &w.shortdescription) {
                if !sd.is_empty() {
                    design.wire_shortdescription.insert(id.clone(), decode_entity(sd));
                }
            }
        }

        // Process device pins (pins inside device) and deviceconnector pins
        for dev in &conn.device {
            for ip in &dev.internalpin {
                if let Some(pid) = &ip.id {
                    design.internal_pin_ids.insert(pid.clone());
                }
            }
            if let Some(dev_id) = &dev.id {
                for pin in &dev.pin {
                    if let (Some(pid), Some(sd)) = (&pin.id, &pin.shortdescription) {
                        if !sd.is_empty() {
                            let decoded = decode_entity(sd);
                            design.pin_shortdescription.insert(pid.clone(), decoded.clone());
                            design.device_pins
                                .entry(dev_id.clone())
                                .or_default()
                                .insert(pid.clone(), decoded);
                        }
                    }
                    if let (Some(pid), Some(cp)) = (&pin.id, &pin.connectedpin) {
                        design.pin_connections.push((pid.clone(), cp.clone()));
                    }
                }
            }
        }
        for gd in &conn.grounddevice {
            for ip in &gd.internalpin {
                if let Some(pid) = &ip.id {
                    design.internal_pin_ids.insert(pid.clone());
                }
            }
            if let Some(gd_id) = &gd.id {
                for pin in &gd.pin {
                    if let (Some(pid), Some(sd)) = (&pin.id, &pin.shortdescription) {
                        if !sd.is_empty() {
                            let decoded = decode_entity(sd);
                            design.pin_shortdescription.insert(pid.clone(), decoded.clone());
                            design.device_pins
                                .entry(gd_id.clone())
                                .or_default()
                                .insert(pid.clone(), decoded);
                        }
                    }
                    if let (Some(pid), Some(cp)) = (&pin.id, &pin.connectedpin) {
                        design.pin_connections.push((pid.clone(), cp.clone()));
                    }
                }
            }
        }
        for dc in &conn.deviceconnector {
            for pin in &dc.pin {
                if let (Some(pid), Some(sd)) = (&pin.id, &pin.shortdescription) {
                    if !sd.is_empty() {
                        let decoded = decode_entity(sd);
                        design.pin_shortdescription.insert(pid.clone(), decoded.clone());
                    }
                }
                if let (Some(pid), Some(cp)) = (&pin.id, &pin.connectedpin) {
                    design.pin_connections.push((pid.clone(), cp.clone()));
                }
            }
        }
        for pin in &conn.pin {
            if let (Some(pid), Some(sd)) = (&pin.id, &pin.shortdescription) {
                if !sd.is_empty() {
                    let decoded = decode_entity(sd);
                    design.pin_shortdescription.insert(pid.clone(), decoded.clone());
                }
            }
            if let (Some(pid), Some(cp)) = (&pin.id, &pin.connectedpin) {
                design.pin_connections.push((pid.clone(), cp.clone()));
            }
        }
    }

    // designsharedpinusage: (diagram, schempin) -> logicpin
    for psu in &logicaldesign.designsharedpinusage {
        if let (Some(d), Some(s), Some(l)) = (&psu.diagram, &psu.schempin, &psu.logicpin) {
            design
                .schempin_to_logicpin
                .insert((d.clone(), s.clone()), l.clone());
        }
    }

    // Propagate pin shortdescription through connected pins
    let mut changed = true;
    while changed {
        changed = false;
        for (a, b) in &design.pin_connections {
            if let Some(desc) = design.pin_shortdescription.get(a) {
                if !desc.is_empty() && design.pin_shortdescription.get(b) != Some(desc) {
                    design.pin_shortdescription.insert(b.clone(), desc.clone());
                    changed = true;
                }
            }
            if let Some(desc) = design.pin_shortdescription.get(b) {
                if !desc.is_empty() && design.pin_shortdescription.get(a) != Some(desc) {
                    design.pin_shortdescription.insert(a.clone(), desc.clone());
                    changed = true;
                }
            }
        }
    }

    // Build diagram content map
    let mut diagram_contents: HashMap<String, DiagramContent> = HashMap::new();
    let diagram_id_to_name: HashMap<String, String> = design
        .diagrams
        .iter()
        .map(|d| (d.id.clone(), d.name.clone()))
        .collect();

    for diag in &logicaldesign.diagram {
        let diagram_id = match &diag.id {
            Some(id) => id.clone(),
            None => continue,
        };
        for dc in &diag.diagramcontent {
            let content_id = dc.diagramid.as_ref().unwrap_or(&diagram_id).clone();
            let content = convert_diagram_content(
                dc,
                &diagram_id,
                &diagram_id_to_name,
                &design.device_pins,
                &design.pin_shortdescription,
                &design.pin_name,
                &design.schempin_to_logicpin,
                &design.internal_pin_ids,
                &design.connector_type,
            );
            diagram_contents.insert(content_id, content);
        }
    }

    // Enrich wire names from diagram content (real displayed names) and add diagram-only wires
    let mut wire_id_to_display: HashMap<String, String> = HashMap::new();
    let mut wire_id_to_diagram: HashMap<String, String> = HashMap::new();
    for (content_id, content) in &diagram_contents {
        let diagram_name = diagram_id_to_name
            .get(content_id)
            .cloned()
            .unwrap_or_else(|| content_id.clone());
        for w in &content.wires {
            if !w.id.is_empty() && !w.name.is_empty() {
                wire_id_to_display
                    .entry(w.id.clone())
                    .or_insert_with(|| w.name.clone());
                wire_id_to_diagram
                    .entry(w.id.clone())
                    .or_insert_with(|| diagram_name.clone());
            }
        }
    }
    for w in &mut design.connectivity.wires {
        if let Some(display) = wire_id_to_display.get(&w.id) {
            w.name = display.clone();
        }
        if let Some(diag) = wire_id_to_diagram.get(&w.id) {
            w.diagram_name = diag.clone();
        }
    }
    let mut seen_wire_ids: HashSet<String> = design
        .connectivity
        .wires
        .iter()
        .map(|w| w.id.clone())
        .collect();
    for (content_id, content) in &diagram_contents {
        let diagram_name = diagram_id_to_name
            .get(content_id)
            .cloned()
            .unwrap_or_else(|| content_id.clone());
        for w in &content.wires {
            let key = if !w.id.is_empty() {
                w.id.clone()
            } else if !w.name.is_empty() {
                w.name.clone()
            } else {
                continue;
            };
            if seen_wire_ids.insert(key.clone()) {
                let display = if !w.name.is_empty() {
                    w.name.clone()
                } else if !w.id.is_empty() {
                    format!("Wire {}", w.id)
                } else {
                    continue;
                };
                design.connectivity.wires.push(WireRef {
                    id: key,
                    name: display,
                    diagram_name: diagram_name.clone(),
                });
            }
        }
    }

    // Build cross-reference map from diagram content (not xrefs): (Type Of, Device, Name) -> [(Diagram, Coord)]
    let device_type_by_id: HashMap<String, (String, String)> = design
        .connectivity
        .devices
        .iter()
        .map(|e| {
            (
                e.id.clone(),
                (
                    e.type_label.clone().unwrap_or_default(),
                    e.name.clone(),
                ),
            )
        })
        .collect();
    for (content_id, content) in &diagram_contents {
        let diagram_name = diagram_id_to_name
            .get(content_id)
            .cloned()
            .unwrap_or_else(|| content_id.clone());
        for dev in &content.devices {
            let (type_label, _) = device_type_by_id
                .get(&dev.connref)
                .cloned()
                .unwrap_or_default();
            let name = if dev.name.is_empty() {
                dev.connref.clone()
            } else {
                dev.name.clone()
            };
            let coord = (dev.x, dev.y);
            let target = CrossRefTarget(diagram_name.clone(), coord);
            // Add entries for both name and connref when they differ (Elements list looks up by either)
            for key_name in [name.clone(), dev.connref.clone()] {
                if key_name.is_empty() {
                    continue;
                }
                let key = CrossRefKey("Device".to_string(), String::new(), key_name);
                design.cross_ref_map.entry(key).or_default().push(target.clone());
            }
            if !type_label.is_empty() {
                design
                    .cross_ref_map
                    .entry(CrossRefKey("Device".to_string(), type_label.clone(), name))
                    .or_default()
                    .push(target.clone());
            }
        }
        for conn in &content.connectors {
            let name = if conn.name.is_empty() {
                conn.connref.clone()
            } else {
                conn.name.clone()
            };
            let coord = (conn.x, conn.y);
            let target = CrossRefTarget(diagram_name.clone(), coord);
            for key_name in [name, conn.connref.clone()] {
                if !key_name.is_empty() {
                    let key = CrossRefKey("Connector".to_string(), String::new(), key_name);
                    design.cross_ref_map.entry(key).or_default().push(target.clone());
                }
            }
        }
        for sp in &content.splices {
            let name = if sp.name.is_empty() {
                sp.connref.clone()
            } else {
                sp.name.clone()
            };
            let coord = (sp.x, sp.y);
            let target = CrossRefTarget(diagram_name.clone(), coord);
            for key_name in [name, sp.connref.clone()] {
                if !key_name.is_empty() {
                    let key = CrossRefKey("Splice".to_string(), String::new(), key_name);
                    design.cross_ref_map.entry(key).or_default().push(target.clone());
                }
            }
        }
        for wire in &content.wires {
            let mut wire_coord: Option<(f64, f64)> = None;
            for seg in &wire.segments {
                let mid_x = (seg.x1 + seg.x2) / 2.0;
                let mid_y = (seg.y1 + seg.y2) / 2.0;
                let coord = (mid_x, mid_y);
                wire_coord = wire_coord.or(Some(coord));
                for ti in &seg.text_items {
                    if ti.is_wire_name && !ti.text.is_empty() {
                        let key = CrossRefKey("Wire".to_string(), String::new(), ti.text.clone());
                        design
                            .cross_ref_map
                            .entry(key)
                            .or_default()
                            .push(CrossRefTarget(diagram_name.clone(), coord));
                    }
                }
            }
            // Also add wire.id for Elements lookup by connectivity id (hit_test returns id)
            if !wire.id.is_empty() {
                let coord = wire_coord.unwrap_or((0.0, 0.0));
                let key = CrossRefKey("Wire".to_string(), String::new(), wire.id.clone());
                design
                    .cross_ref_map
                    .entry(key)
                    .or_default()
                    .push(CrossRefTarget(diagram_name.clone(), coord));
            }
        }
    }

    Ok((design, diagram_contents))
}

/// Backward compatibility: load diagram content for a single diagram.
/// Prefer parse_project when loading a new file to get all content at once.
pub fn load_diagram_content<R: BufRead>(
    reader: R,
    diagram_id: &str,
    _device_pins: &HashMap<String, HashMap<String, String>>,
    _pin_shortdescription: &HashMap<String, String>,
    _schempin_to_logicpin: &HashMap<(String, String), String>,
) -> Result<DiagramContent, String> {
    let (_design, diagram_contents) = parse_project(reader)?;
    Ok(diagram_contents
        .get(diagram_id)
        .cloned()
        .unwrap_or_else(|| {
            let mut empty = DiagramContent::default();
            empty.border = Some(BorderBounds {
                x: 0.0,
                y: 0.0,
                width: 1000.0,
                height: 800.0,
                units: "inch".to_string(),
            });
            empty
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagram_visible_property() {
        let xml = r#"<?xml version="1.0"?>
<project>
  <designmgr>
    <logicaldesign>
      <designsharedusagemgr>
        <diagramsharedusagemgr diagramid="_1" diagramname="Visible Diagram"></diagramsharedusagemgr>
        <diagramsharedusagemgr diagramid="_2" diagramname="Hidden Diagram"></diagramsharedusagemgr>
      </designsharedusagemgr>
      <connectivity/>
      <diagram id="_1" name="Visible Diagram"><property name="VISIBLE" val="True"/><diagramcontent diagramid="_1"/></diagram>
      <diagram id="_2" name="Hidden Diagram"><property name="VISIBLE" val="FALSE"/><diagramcontent diagramid="_2"/></diagram>
    </logicaldesign>
  </designmgr>
</project>"#;
        let (design, _) = parse_project(std::io::Cursor::new(xml)).unwrap();
        let visible: Vec<_> = design.diagrams.iter().filter(|d| d.visible).map(|d| d.name.as_str()).collect();
        let hidden: Vec<_> = design.diagrams.iter().filter(|d| !d.visible).map(|d| d.name.as_str()).collect();
        assert_eq!(visible, ["Visible Diagram"], "visible={:?} hidden={:?}", visible, hidden);
        assert_eq!(hidden, ["Hidden Diagram"]);
    }

    #[test]
    #[ignore] // Run with: cargo test test_tiger_visible -- --ignored
    fn test_tiger_visible() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("TIGER_SPEC_005b.xml");
        let f = std::fs::File::open(&path).expect("TIGER_SPEC_005b.xml not found");
        let (design, _) = parse_project(std::io::BufReader::new(f)).unwrap();
        let master = design.diagrams.iter().find(|d| d.name == "0 Master");
        let index = design.diagrams.iter().find(|d| d.name == ".index");
        assert!(master.is_some(), "0 Master should be in diagram list");
        assert!(index.is_some(), ".index should be in diagram list");
        assert!(!master.unwrap().visible, "0 Master should be hidden (VISIBLE=False)");
        assert!(!index.unwrap().visible, ".index should be hidden (VISIBLE=False)");
    }
}
