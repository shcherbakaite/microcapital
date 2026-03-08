//! hard_xml structs and conversion for diagram content (border, wires, devices, connectors).
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

use std::collections::{HashMap, HashSet};

use hard_xml::XmlRead;

use crate::model::{
    AttributeTextPlacement, BorderBounds, ConnectorKind, ConnectorOutline, ConnectorPin,
    CrossReference, ConnectorSymbol, DiagramContent, DevicePin, DeviceSymbol, DdAttribute,
    HorizontalJust, Schemindicator, SpliceSymbol, TextStyle, VerticalJust, Wire, WireSegment,
    WireTextItem,
};

// ---------------------------------------------------------------------------
// hard_xml structs for diagram content
// ---------------------------------------------------------------------------

#[derive(Debug, XmlRead)]
#[xml(tag = "schemwire")]
pub struct XmlSchemwire {
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    /// Harness wire id (harnwire); matches WireRef.id in connectivity for Elements view selection.
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(child = "schemsegment")]
    schemsegment: Vec<XmlSchemsegment>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemsegment")]
pub struct XmlSchemsegment {
    #[xml(attr = "x1")]
    x1: Option<f64>,
    #[xml(attr = "y1")]
    y1: Option<f64>,
    #[xml(attr = "x2")]
    x2: Option<f64>,
    #[xml(attr = "y2")]
    y2: Option<f64>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "compositetext")]
    compositetext: Vec<XmlCompositetext>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "compositetext")]
pub struct XmlCompositetext {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
    #[xml(attr = "hjust")]
    hjust: Option<String>,
    #[xml(attr = "vjust")]
    vjust: Option<String>,
    #[xml(attr = "rotation")]
    rotation: Option<f64>,
    #[xml(attr = "font")]
    font: Option<String>,
    #[xml(attr = "style")]
    style: Option<String>,
    #[xml(attr = "val")]
    val: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "decorationname")]
    decorationname: Option<String>,
    #[xml(child = "formattedvalue")]
    formattedvalue: Vec<XmlFormattedvalue>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "formattedvalue")]
pub struct XmlFormattedvalue {
    #[xml(child = "valueentry")]
    valueentry: Vec<XmlValueentry>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "refattachschem")]
pub struct XmlRefattachschem {
    #[xml(attr = "attachref")]
    attachref: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemdevice")]
pub struct XmlSchemdevice {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "refattachschem")]
    refattachschem: Vec<XmlRefattachschem>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlParameters>,
    #[xml(child = "schempin")]
    schempin: Vec<XmlSchempin>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
    #[xml(child = "arc")]
    arc: Vec<XmlArc>,
    #[xml(child = "circle")]
    circle: Vec<XmlCircle>,
    #[xml(child = "rectangle")]
    rectangle: Vec<XmlRectangle>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlPolyline>,
    #[xml(child = "polygon")]
    polygon: Vec<XmlPolygon>,
    #[xml(child = "curve")]
    curve: Vec<XmlCurve>,
    #[xml(child = "widtharc")]
    widtharc: Vec<XmlWidtharc>,
    #[xml(child = "attributetext")]
    attributetext: Vec<XmlAttributetext>,
    #[xml(child = "valueentry")]
    valueentry: Vec<XmlValueentry>,
}

/// Schematic ground device instance. Same structure as schemdevice; treated as Device for rendering.
#[derive(Debug, XmlRead)]
#[xml(tag = "schemgrounddevice")]
pub struct XmlSchemgrounddevice {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "refattachschem")]
    refattachschem: Vec<XmlRefattachschem>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlParameters>,
    #[xml(child = "schempin")]
    schempin: Vec<XmlSchempin>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
    #[xml(child = "arc")]
    arc: Vec<XmlArc>,
    #[xml(child = "circle")]
    circle: Vec<XmlCircle>,
    #[xml(child = "rectangle")]
    rectangle: Vec<XmlRectangle>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlPolyline>,
    #[xml(child = "polygon")]
    polygon: Vec<XmlPolygon>,
    #[xml(child = "curve")]
    curve: Vec<XmlCurve>,
    #[xml(child = "widtharc")]
    widtharc: Vec<XmlWidtharc>,
    #[xml(child = "attributetext")]
    attributetext: Vec<XmlAttributetext>,
    #[xml(child = "valueentry")]
    valueentry: Vec<XmlValueentry>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "parameters")]
pub struct XmlParameters {
    #[xml(child = "paramextent")]
    paramextent: Vec<XmlParamextent>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "paramextent")]
pub struct XmlParamextent {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "width")]
    width: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schempin")]
pub struct XmlSchempin {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(child = "attributetext")]
    attributetext: Vec<XmlAttributetext>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "line")]
pub struct XmlLine {
    #[xml(attr = "x1")]
    x1: Option<f64>,
    #[xml(attr = "y1")]
    y1: Option<f64>,
    #[xml(attr = "x2")]
    x2: Option<f64>,
    #[xml(attr = "y2")]
    y2: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "arc")]
pub struct XmlArc {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "radius")]
    radius: Option<f64>,
    #[xml(attr = "startangle")]
    startangle: Option<f64>,
    #[xml(attr = "travelangle")]
    travelangle: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "circle")]
pub struct XmlCircle {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "radius")]
    radius: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "rectangle")]
pub struct XmlRectangle {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "width")]
    width: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "polyline")]
pub struct XmlPolyline {
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(child = "point")]
    point: Vec<XmlPoint>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "polygon")]
pub struct XmlPolygon {
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(child = "point")]
    point: Vec<XmlPoint>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "curve")]
pub struct XmlCurve {
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(child = "point")]
    point: Vec<XmlPoint>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "widtharc")]
pub struct XmlWidtharc {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "radius")]
    radius: Option<f64>,
    #[xml(attr = "startangle")]
    startangle: Option<f64>,
    #[xml(attr = "travelangle")]
    travelangle: Option<f64>,
    #[xml(attr = "width")]
    width: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "point")]
pub struct XmlPoint {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "attributetext")]
pub struct XmlAttributetext {
    #[xml(attr = "val")]
    val: Option<String>,
    #[xml(attr = "nameref")]
    nameref: Option<String>,
    #[xml(attr = "font")]
    font: Option<String>,
    #[xml(attr = "style")]
    style: Option<String>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "rotation")]
    rotation: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
    #[xml(attr = "hjust")]
    hjust: Option<String>,
    #[xml(attr = "vjust")]
    vjust: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "valueentry")]
pub struct XmlValueentry {
    #[xml(attr = "val")]
    val: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "xreftext")]
pub struct XmlXreftext {
    #[xml(attr = "blocktext")]
    blocktext: Option<String>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "height")]
    height: Option<f64>,
    #[xml(attr = "hjust")]
    hjust: Option<String>,
    #[xml(attr = "vjust")]
    vjust: Option<String>,
    #[xml(attr = "rotation")]
    rotation: Option<f64>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "xreftextcontainer")]
pub struct XmlXreftextcontainer {
    #[xml(child = "xreftext")]
    xreftext: Vec<XmlXreftext>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemconnector")]
pub struct XmlSchemconnector {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlParameters>,
    #[xml(child = "glyph")]
    glyph: Vec<XmlGlyph>,
    #[xml(child = "xreftextcontainer")]
    xreftextcontainer: Vec<XmlXreftextcontainer>,
    #[xml(child = "schempin")]
    schempin: Vec<XmlSchempin>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
    #[xml(child = "arc")]
    arc: Vec<XmlArc>,
    #[xml(child = "circle")]
    circle: Vec<XmlCircle>,
    #[xml(child = "rectangle")]
    rectangle: Vec<XmlRectangle>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlPolyline>,
    #[xml(child = "polygon")]
    polygon: Vec<XmlPolygon>,
    #[xml(child = "curve")]
    curve: Vec<XmlCurve>,
    #[xml(child = "widtharc")]
    widtharc: Vec<XmlWidtharc>,
    #[xml(child = "attributetext")]
    attributetext: Vec<XmlAttributetext>,
    #[xml(child = "valueentry")]
    valueentry: Vec<XmlValueentry>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "glyph")]
pub struct XmlGlyph {
    #[xml(attr = "type")]
    type_: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "parameter")]
pub struct XmlParameter {
    #[xml(attr = "type")]
    type_: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(child = "glyph")]
    glyph: Vec<XmlGlyph>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemhookup")]
pub struct XmlSchemhookup {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlIndicatorParameters>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemindicator")]
pub struct XmlSchemindicator {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(attr = "indicatortype")]
    indicatortype: Option<String>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlIndicatorParameters>,
    #[xml(child = "schemhookup")]
    schemhookup: Vec<XmlSchemhookup>,
    #[xml(child = "arc")]
    arc: Vec<XmlArc>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlPolyline>,
    #[xml(child = "compositetext")]
    compositetext: Vec<XmlCompositetext>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "parameters")]
pub struct XmlIndicatorParameters {
    #[xml(child = "parameter")]
    parameter: Vec<XmlParameter>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "schemsplice")]
pub struct XmlSchemsplice {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
    #[xml(attr = "transform")]
    transform: Option<String>,
    #[xml(attr = "connref")]
    connref: Option<String>,
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "parameters")]
    parameters: Vec<XmlParameters>,
    #[xml(child = "circle")]
    circle: Vec<XmlCircle>,
    #[xml(child = "line")]
    line: Vec<XmlLine>,
    #[xml(child = "arc")]
    arc: Vec<XmlArc>,
    #[xml(child = "rectangle")]
    rectangle: Vec<XmlRectangle>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlPolyline>,
    #[xml(child = "polygon")]
    polygon: Vec<XmlPolygon>,
    #[xml(child = "attributetext")]
    attributetext: Vec<XmlAttributetext>,
    #[xml(child = "valueentry")]
    valueentry: Vec<XmlValueentry>,
    #[xml(child = "xreftextcontainer")]
    xreftextcontainer: Vec<XmlXreftextcontainer>,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

fn decode_entity(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn parse_transform(s: &str) -> [f64; 6] {
    let parts: Vec<f64> = s.split_whitespace().filter_map(|p| p.parse().ok()).collect();
    if parts.len() >= 6 {
        [parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]]
    } else {
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    }
}

fn parse_text_style(s: &str) -> TextStyle {
    match s.to_lowercase().as_str() {
        "bold" => TextStyle::Bold,
        "italic" => TextStyle::Italic,
        "bolditalic" => TextStyle::BoldItalic,
        _ => TextStyle::Plain,
    }
}

fn parse_hjust(s: Option<&String>) -> HorizontalJust {
    let v = s.as_deref().map(|x| x.to_lowercase());
    match v.as_deref() {
        Some("right") => HorizontalJust::Right,
        Some("middle") | Some("center") => HorizontalJust::Center,
        _ => HorizontalJust::Left,
    }
}

fn parse_vjust(s: Option<&String>) -> VerticalJust {
    let v = s.as_deref().map(|x| x.to_lowercase());
    match v.as_deref() {
        Some("bottom") => VerticalJust::Bottom,
        Some("center") => VerticalJust::Center,
        _ => VerticalJust::Top,
    }
}

/// Parse cross-reference text (e.g. "/0 Master" or "/Design:Diagram") to extract diagram name.
fn parse_xref_diagram_name(text: &str) -> Option<String> {
    let s = text.trim().strip_prefix('/')?.trim();
    if s.is_empty() {
        return None;
    }
    // Format: "Design:Diagram" or "0 Master" - use part after ":" if present, else whole string
    let name = if let Some(idx) = s.rfind(':') {
        s[idx + 1..].trim().to_string()
    } else {
        s.to_string()
    };
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn compositetext_to_wire_text(ct: &XmlCompositetext) -> Option<WireTextItem> {
    let text = ct.val.as_ref()
        .filter(|v| !v.is_empty())
        .map(|v| decode_entity(v))
        .or_else(|| {
            let s: String = ct.formattedvalue
                .iter()
                .flat_map(|fv| &fv.valueentry)
                .filter_map(|ve| ve.val.as_ref())
                .map(|v| decode_entity(v))
                .collect::<Vec<_>>()
                .join("");
            if s.is_empty() { None } else { Some(s) }
        })?;
    if text.is_empty() {
        return None;
    }
    let is_wire_name = ct.decorationname
        .as_deref()
        .map(|s| s.starts_with("Name"))
        .unwrap_or(false);
    let is_xref = text.trim().starts_with('/') && (text.contains(':') || text.contains(' '));
    let xref_diagram_name = if is_xref {
        parse_xref_diagram_name(&text)
    } else {
        None
    };
    Some(WireTextItem {
        text,
        x: ct.x.unwrap_or(0.0),
        y: ct.y.unwrap_or(0.0),
        height: ct.height.unwrap_or(1382.0),
        hjust: parse_hjust(ct.hjust.as_ref()),
        vjust: parse_vjust(ct.vjust.as_ref()),
        rotation: ct.rotation.unwrap_or(0.0),
        font: ct.font.clone(),
        style: ct.style.as_deref().map(parse_text_style).unwrap_or(TextStyle::Plain),
        is_wire_name,
        is_xref,
        xref_diagram_name,
    })
}

fn attributetext_to_placement(at: &XmlAttributetext) -> Option<AttributeTextPlacement> {
    if !is_visible(at.visibility.as_ref()) {
        return None;
    }
    let x = at.x.unwrap_or(0.0);
    let y = at.y.unwrap_or(0.0);
    let height = at.height.unwrap_or(1920.0);
    let rotation = at.rotation.unwrap_or(0.0);
    Some(AttributeTextPlacement {
        x,
        y,
        hjust: parse_hjust(at.hjust.as_ref()),
        vjust: parse_vjust(at.vjust.as_ref()),
        rotation,
        height,
    })
}

fn is_visible(s: Option<&String>) -> bool {
    s.map(|v| v != "false").unwrap_or(true)
}

/// Convert XmlDiagramcontent to DiagramContent.
pub fn convert_diagram_content(
    dc: &super::XmlDiagramcontent,
    diagram_id: &str,
    diagram_id_to_name: &HashMap<String, String>,
    device_pins: &HashMap<String, HashMap<String, String>>,
    pin_shortdescription: &HashMap<String, String>,
    pin_name: &HashMap<String, String>,
    schempin_to_logicpin: &HashMap<(String, String), String>,
    internal_pin_ids: &HashSet<String>,
    connector_type: &HashMap<String, ConnectorKind>,
) -> DiagramContent {
    let mut content = DiagramContent::default();

    // Border
    for b in &dc.border {
        let x = b.paperx.or(b.x).unwrap_or(0.0);
        let y = b.papery.or(b.y).unwrap_or(0.0);
        let width = b.paperwidth.or(b.width).unwrap_or(0.0);
        let height = b.paperheight.or(b.height).unwrap_or(0.0);
        content.border = Some(BorderBounds {
            x,
            y,
            width,
            height,
            units: b.units.clone().unwrap_or_else(|| "inch".to_string()),
        });
        break;
    }

    // Datadictionary
    for dd in &dc.datadictionary {
        for dda in &dd.ddattribute {
            if let Some(id) = &dda.id {
                let thickness = dda.thickness.as_ref().and_then(|s| {
                    if s.eq_ignore_ascii_case("inherit") || s.is_empty() {
                        None
                    } else {
                        s.parse().ok()
                    }
                });
                let mut attr = DdAttribute {
                    color: None,
                    thickness,
                    linestyle: dda.linestyle.clone(),
                };
                for c in &dda.color {
                    if let Some(v) = &c.value {
                        attr.color = Some(v.clone());
                    }
                }
                content.datadictionary.insert(id.clone(), attr);
            }
        }
    }

    // Schemwires and schemsegments: each schemwire is one Wire with its segments
    for sw in &dc.schemwire {
        let wire_attr = sw.attributeref.clone();
        let mut wire_segments = Vec::new();
        let mut wire_name = String::new();
        for seg in &sw.schemsegment {
            let text_items: Vec<WireTextItem> = seg.compositetext
                .iter()
                .filter(|ct| is_visible(ct.visibility.as_ref()))
                .filter_map(|ct| compositetext_to_wire_text(ct))
                .collect();
            if wire_name.is_empty() {
                if let Some(ti) = text_items.iter().find(|ti| ti.is_wire_name && !ti.text.is_empty()) {
                    wire_name = ti.text.clone();
                }
            }
            wire_segments.push(WireSegment {
                x1: seg.x1.unwrap_or(0.0),
                y1: seg.y1.unwrap_or(0.0),
                x2: seg.x2.unwrap_or(0.0),
                y2: seg.y2.unwrap_or(0.0),
                attributeref: seg.attributeref.clone().or(wire_attr.clone()),
                text_items,
                bounds: None,
            });
        }
        content.wires.push(Wire {
            name: wire_name,
            id: sw.connref.clone().unwrap_or_default(),
            segments: wire_segments,
        });
    }

    // Build map of schemdevice id -> XmlSchemdevice for refattachschem resolution
    let schemdevice_by_id: HashMap<String, &XmlSchemdevice> = dc
        .schemdevice
        .iter()
        .filter_map(|sd| sd.id.as_ref().map(|id| (id.clone(), sd)))
        .collect();
    let schemgrounddevice_by_id: HashMap<String, &XmlSchemgrounddevice> = dc
        .schemgrounddevice
        .iter()
        .filter_map(|sgd| sgd.id.as_ref().map(|id| (id.clone(), sgd)))
        .collect();

    // Schemdevices
    for sd in &dc.schemdevice {
        if !is_visible(sd.visibility.as_ref()) {
            continue;
        }
        if let Some(device) = convert_schemdevice(
            sd,
            &schemdevice_by_id,
            diagram_id,
            diagram_id_to_name,
            device_pins,
            pin_shortdescription,
            pin_name,
            schempin_to_logicpin,
            internal_pin_ids,
        ) {
            content.devices.push(device);
        }
    }

    // Schemgrounddevices (treated as devices for rendering)
    for sgd in &dc.schemgrounddevice {
        if !is_visible(sgd.visibility.as_ref()) {
            continue;
        }
        if let Some(device) = convert_schemgrounddevice(
            sgd,
            &schemgrounddevice_by_id,
            diagram_id,
            diagram_id_to_name,
            device_pins,
            pin_shortdescription,
            pin_name,
            schempin_to_logicpin,
            internal_pin_ids,
        ) {
            content.devices.push(device);
        }
    }

    // Schemconnectors
    for sc in &dc.schemconnector {
        if let Some(conn) = convert_schemconnector(
            sc,
            diagram_id,
            diagram_id_to_name,
            pin_name,
            schempin_to_logicpin,
            connector_type,
        ) {
            content.connectors.push(conn);
        }
    }

    // Schemsplices
    for ss in &dc.schemsplice {
        if !is_visible(ss.visibility.as_ref()) {
            continue;
        }
        if let Some(splice) = convert_schemsplice(ss, diagram_id, diagram_id_to_name) {
            content.splices.push(splice);
        }
    }

    // Schemindicators (oval, twist, shield)
    for si in &dc.schemindicator {
        if !is_visible(si.visibility.as_ref()) {
            continue;
        }
        if let Some(ind) = convert_schemindicator(si) {
            content.schemindicators.push(ind);
        }
    }

    // Remove from segment text_items any text that matches a pin short_description, so
    // ShortDescription is drawn only once at the pin (avoids duplicate with pin attributetext).
    let mut pin_short_descriptions = std::collections::HashSet::new();
    for c in &content.connectors {
        for p in &c.pins {
            if let Some(ref s) = p.short_description {
                if !s.is_empty() {
                    pin_short_descriptions.insert(s.clone());
                }
            }
        }
    }
    for d in &content.devices {
        for p in &d.pins {
            if let Some(ref s) = p.short_description {
                if !s.is_empty() {
                    pin_short_descriptions.insert(s.clone());
                }
            }
        }
    }
    for wire in &mut content.wires {
        for seg in &mut wire.segments {
            seg.text_items.retain(|ti| !pin_short_descriptions.contains(&ti.text));
        }
    }

    crate::bounds::compute_all_bounds(&mut content);

    content
}

fn collect_outline_from_schemdevice(
    sd: &XmlSchemdevice,
    outline: &mut Vec<ConnectorOutline>,
    bounds_points: &mut Vec<(f64, f64)>,
) {
    for l in &sd.line {
        if is_visible(l.visibility.as_ref()) {
            let (x1, y1, x2, y2) = (
                l.x1.unwrap_or(0.0),
                l.y1.unwrap_or(0.0),
                l.x2.unwrap_or(0.0),
                l.y2.unwrap_or(0.0),
            );
            outline.push(ConnectorOutline::Line { x1, y1, x2, y2 });
            bounds_points.push((x1, y1));
            bounds_points.push((x2, y2));
        }
    }
    for a in &sd.arc {
        if is_visible(a.visibility.as_ref()) {
            let (x, y, r) = (
                a.x.unwrap_or(0.0),
                a.y.unwrap_or(0.0),
                a.radius.unwrap_or(0.0),
            );
            outline.push(ConnectorOutline::Arc {
                x,
                y,
                radius: r,
                start_angle: a.startangle.unwrap_or(0.0),
                travel_angle: a.travelangle.unwrap_or(0.0),
            });
            bounds_points.push((x - r, y - r));
            bounds_points.push((x + r, y + r));
        }
    }
    for c in &sd.circle {
        if is_visible(c.visibility.as_ref()) {
            let (x, y, r) = (
                c.x.unwrap_or(0.0),
                c.y.unwrap_or(0.0),
                c.radius.unwrap_or(0.0),
            );
            if r > 0.0 {
                outline.push(ConnectorOutline::Circle { x, y, radius: r });
                bounds_points.push((x - r, y - r));
                bounds_points.push((x + r, y + r));
            }
        }
    }
    for r in &sd.rectangle {
        if is_visible(r.visibility.as_ref()) {
            let (x, y, w, h) = (
                r.x.unwrap_or(0.0),
                r.y.unwrap_or(0.0),
                r.width.unwrap_or(9600.0),
                r.height.unwrap_or(0.0),
            );
            let h_final = if h > 0.0 { h } else { w * 0.75 };
            outline.push(ConnectorOutline::Rectangle {
                x,
                y,
                width: w,
                height: h_final,
            });
            bounds_points.push((x, y));
            bounds_points.push((x + w, y + h_final));
        }
    }
    for pl in &sd.polyline {
        if is_visible(pl.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pl
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polyline { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for pg in &sd.polygon {
        if is_visible(pg.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pg
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polygon { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for cu in &sd.curve {
        if is_visible(cu.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = cu
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Curve { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for wa in &sd.widtharc {
        if is_visible(wa.visibility.as_ref()) {
            if let (Some(x), Some(y), Some(r), Some(sa), Some(ta), Some(w)) = (
                wa.x,
                wa.y,
                wa.radius,
                wa.startangle,
                wa.travelangle,
                wa.width,
            ) {
                if r > 0.0 && w > 0.0 {
                    outline.push(ConnectorOutline::WidthArc {
                        x,
                        y,
                        radius: r,
                        start_angle: sa,
                        travel_angle: ta,
                        width: w,
                    });
                    bounds_points.push((x - r - w, y - r - w));
                    bounds_points.push((x + r + w, y + r + w));
                }
            }
        }
    }
}

fn convert_schemdevice(
    sd: &XmlSchemdevice,
    schemdevice_by_id: &HashMap<String, &XmlSchemdevice>,
    diagram_id: &str,
    _diagram_id_to_name: &HashMap<String, String>,
    device_pins: &HashMap<String, HashMap<String, String>>,
    pin_shortdescription: &HashMap<String, String>,
    pin_name: &HashMap<String, String>,
    schempin_to_logicpin: &HashMap<(String, String), String>,
    internal_pin_ids: &HashSet<String>,
) -> Option<DeviceSymbol> {
    let connref = sd.connref.clone().unwrap_or_default();
    let mut name = String::new();
    let mut name_font = None;
    let mut name_style = TextStyle::Plain;
    let mut name_placement = None;
    for at in &sd.attributetext {
        if at.nameref.as_deref() == Some("Name") {
            if let Some(v) = &at.val {
                name = decode_entity(v);
            }
            name_font = at.font.clone();
            name_style = at.style.as_deref().map(parse_text_style).unwrap_or(TextStyle::Plain);
            name_placement = attributetext_to_placement(at);
            break;
        }
    }
    for ve in &sd.valueentry {
        // Only use valueentry without connref for device name (entries with connref are pin-specific).
        if ve.connref.is_none() {
            if let Some(v) = &ve.val {
                name.push_str(&decode_entity(v));
            }
        }
    }
    if name.is_empty() {
        name = connref.clone();
    }

    // Pin names from device-level valueentry with connref (custom symbols store pin names here).
    let pin_value_names: std::collections::HashMap<String, String> = sd
        .valueentry
        .iter()
        .filter_map(|ve| {
            let connref = ve.connref.as_ref()?;
            let val = ve.val.as_ref().map(|s| decode_entity(s)).filter(|s| !s.is_empty())?;
            Some((connref.clone(), val))
        })
        .collect();

    let mut extent_x = 0.0;
    let mut extent_y = 0.0;
    let mut width = 9600.0;
    let mut height = 7200.0;
    for p in &sd.parameters {
        for pe in &p.paramextent {
            extent_x = pe.x.unwrap_or(0.0);
            extent_y = pe.y.unwrap_or(0.0);
            width = pe.width.unwrap_or(9600.0);
            let h = pe.height.unwrap_or(0.0);
            height = if h > 0.0 { h } else { width * 0.75 };
        }
    }

    let mut outline: Vec<ConnectorOutline> = Vec::new();
    let mut bounds_points: Vec<(f64, f64)> = Vec::new();

    // Collect geometry from this device
    collect_outline_from_schemdevice(sd, &mut outline, &mut bounds_points);

    // Merge geometry from attached schematics (refattachschem)
    let mut seen: HashSet<String> = HashSet::new();
    let mut to_attach: Vec<&str> = sd
        .refattachschem
        .iter()
        .filter_map(|r| r.attachref.as_deref())
        .collect();
    while let Some(attachref) = to_attach.pop() {
        if seen.contains(attachref) {
            continue;
        }
        seen.insert(attachref.to_string());
        if let Some(attached) = schemdevice_by_id.get(attachref) {
            collect_outline_from_schemdevice(attached, &mut outline, &mut bounds_points);
            for r in &attached.refattachschem {
                if let Some(ref id) = r.attachref {
                    if !seen.contains(id) {
                        to_attach.push(id);
                    }
                }
            }
        }
    }

    if !bounds_points.is_empty() && extent_x == 0.0 && extent_y == 0.0 {
        let (min_x, max_x) = bounds_points
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), (x, _)| (a.min(*x), b.max(*x)));
        let (min_y, max_y) = bounds_points
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), (_, y)| (a.min(*y), b.max(*y)));
        let pad = 480.0;
        extent_x = min_x - pad;
        extent_y = min_y - pad;
        width = (max_x - min_x + 2.0 * pad).max(960.0);
        height = (max_y - min_y + 2.0 * pad).max(480.0);
    }

    let mut pins: Vec<DevicePin> = Vec::new();
    let mut pin_index = 0;
    for sp in &sd.schempin {
        pin_index += 1;
        let x = sp.x.unwrap_or(0.0);
        let y = sp.y.unwrap_or(0.0);
        let transform = sp
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let pin_id = sp
            .connref
            .as_ref()
            .or_else(|| {
                sp.id.as_ref().and_then(|sid| {
                    schempin_to_logicpin.get(&(diagram_id.to_string(), sid.clone()))
                })
            });
        let short_description = pin_id
            .and_then(|lp| device_pins.get(&connref).and_then(|pins| pins.get(lp)))
            .or_else(|| pin_id.and_then(|lp| pin_shortdescription.get(lp)))
            .filter(|s| !s.is_empty())
            .map(|s| decode_entity(s));
        let mut name_placement = None;
        let mut short_description_placement = None;
        // Display name: logical pin name (A1+, 14), then valueentry, then logicpin, else safe connref, else index.
        let mut pin_name_str = pin_id
            .and_then(|id| pin_name.get(id).cloned())
            .or_else(|| pin_id.and_then(|id| pin_value_names.get(id).cloned()))
            .or_else(|| sp.connref.as_ref().and_then(|c| pin_value_names.get(c).cloned()))
            .or_else(|| sp.id.as_ref().and_then(|id| pin_value_names.get(id).cloned()))
            .or_else(|| {
                sp.id
                    .as_ref()
                    .and_then(|sid| {
                        schempin_to_logicpin
                            .get(&(diagram_id.to_string(), sid.clone()))
                            .cloned()
                    })
            })
            .or_else(|| {
                let c = sp.connref.as_ref()?;
                let s = c.as_str();
                if s == connref.as_str() || s.starts_with('_') {
                    None
                } else {
                    Some(c.clone())
                }
            })
            .unwrap_or_else(|| pin_index.to_string());
        let mut name_visible = true;
        for at in &sp.attributetext {
            if at.nameref.as_deref() == Some("Name") {
                name_placement = attributetext_to_placement(at);
                name_visible = is_visible(at.visibility.as_ref());
                if let Some(ref v) = at.val {
                    let decoded = decode_entity(v);
                    if !decoded.is_empty() {
                        pin_name_str = decoded;
                    }
                }
            } else if at.nameref.as_deref() == Some("ShortDescription")
                || at.nameref.as_deref() == Some("DESCRIPTION")
            {
                short_description_placement = attributetext_to_placement(at);
            }
        }
        let is_internal = pin_id
            .as_ref()
            .map_or(false, |id| internal_pin_ids.contains(id.as_str()));
        if !is_internal {
            pins.push(DevicePin {
                x,
                y,
                text: pin_name_str,
                short_description,
                name_placement,
                short_description_placement,
                name_visible,
                transform,
            });
        }
    }

    Some(DeviceSymbol {
        name,
        connref,
        attributeref: sd.attributeref.clone(),
        name_font,
        name_style,
        name_placement,
        x: sd.x.unwrap_or(0.0),
        y: sd.y.unwrap_or(0.0),
        extent_x,
        extent_y,
        width,
        height,
        transform: sd
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        pins,
        outline,
        bounds: None,
    })
}

fn collect_outline_from_schemgrounddevice(
    sgd: &XmlSchemgrounddevice,
    outline: &mut Vec<ConnectorOutline>,
    bounds_points: &mut Vec<(f64, f64)>,
) {
    for l in &sgd.line {
        if is_visible(l.visibility.as_ref()) {
            let (x1, y1, x2, y2) = (
                l.x1.unwrap_or(0.0),
                l.y1.unwrap_or(0.0),
                l.x2.unwrap_or(0.0),
                l.y2.unwrap_or(0.0),
            );
            outline.push(ConnectorOutline::Line { x1, y1, x2, y2 });
            bounds_points.push((x1, y1));
            bounds_points.push((x2, y2));
        }
    }
    for a in &sgd.arc {
        if is_visible(a.visibility.as_ref()) {
            let (x, y, r) = (
                a.x.unwrap_or(0.0),
                a.y.unwrap_or(0.0),
                a.radius.unwrap_or(0.0),
            );
            outline.push(ConnectorOutline::Arc {
                x,
                y,
                radius: r,
                start_angle: a.startangle.unwrap_or(0.0),
                travel_angle: a.travelangle.unwrap_or(0.0),
            });
            bounds_points.push((x - r, y - r));
            bounds_points.push((x + r, y + r));
        }
    }
    for c in &sgd.circle {
        if is_visible(c.visibility.as_ref()) {
            let (x, y, r) = (
                c.x.unwrap_or(0.0),
                c.y.unwrap_or(0.0),
                c.radius.unwrap_or(0.0),
            );
            if r > 0.0 {
                outline.push(ConnectorOutline::Circle { x, y, radius: r });
                bounds_points.push((x - r, y - r));
                bounds_points.push((x + r, y + r));
            }
        }
    }
    for r in &sgd.rectangle {
        if is_visible(r.visibility.as_ref()) {
            let (x, y, w, h) = (
                r.x.unwrap_or(0.0),
                r.y.unwrap_or(0.0),
                r.width.unwrap_or(9600.0),
                r.height.unwrap_or(0.0),
            );
            let h_final = if h > 0.0 { h } else { w * 0.75 };
            outline.push(ConnectorOutline::Rectangle {
                x,
                y,
                width: w,
                height: h_final,
            });
            bounds_points.push((x, y));
            bounds_points.push((x + w, y + h_final));
        }
    }
    for pl in &sgd.polyline {
        if is_visible(pl.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pl
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polyline { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for pg in &sgd.polygon {
        if is_visible(pg.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pg
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polygon { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for cu in &sgd.curve {
        if is_visible(cu.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = cu
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Curve { points: pts.clone() });
                bounds_points.extend(pts);
            }
        }
    }
    for wa in &sgd.widtharc {
        if is_visible(wa.visibility.as_ref()) {
            if let (Some(x), Some(y), Some(r), Some(sa), Some(ta), Some(w)) = (
                wa.x,
                wa.y,
                wa.radius,
                wa.startangle,
                wa.travelangle,
                wa.width,
            ) {
                if r > 0.0 && w > 0.0 {
                    outline.push(ConnectorOutline::WidthArc {
                        x,
                        y,
                        radius: r,
                        start_angle: sa,
                        travel_angle: ta,
                        width: w,
                    });
                    bounds_points.push((x - r - w, y - r - w));
                    bounds_points.push((x + r + w, y + r + w));
                }
            }
        }
    }
}

fn convert_schemgrounddevice(
    sgd: &XmlSchemgrounddevice,
    schemgrounddevice_by_id: &HashMap<String, &XmlSchemgrounddevice>,
    diagram_id: &str,
    _diagram_id_to_name: &HashMap<String, String>,
    device_pins: &HashMap<String, HashMap<String, String>>,
    pin_shortdescription: &HashMap<String, String>,
    pin_name: &HashMap<String, String>,
    schempin_to_logicpin: &HashMap<(String, String), String>,
    internal_pin_ids: &HashSet<String>,
) -> Option<DeviceSymbol> {
    let connref = sgd.connref.clone().unwrap_or_default();
    let mut name = String::new();
    let mut name_font = None;
    let mut name_style = TextStyle::Plain;
    let mut name_placement = None;
    for at in &sgd.attributetext {
        if at.nameref.as_deref() == Some("Name") {
            if let Some(v) = &at.val {
                name = decode_entity(v);
            }
            name_font = at.font.clone();
            name_style = at.style.as_deref().map(parse_text_style).unwrap_or(TextStyle::Plain);
            name_placement = attributetext_to_placement(at);
            break;
        }
    }
    for ve in &sgd.valueentry {
        if ve.connref.is_none() {
            if let Some(v) = &ve.val {
                name.push_str(&decode_entity(v));
            }
        }
    }
    if name.is_empty() {
        name = connref.clone();
    }

    let pin_value_names: std::collections::HashMap<String, String> = sgd
        .valueentry
        .iter()
        .filter_map(|ve| {
            let connref = ve.connref.as_ref()?;
            let val = ve.val.as_ref().map(|s| decode_entity(s)).filter(|s| !s.is_empty())?;
            Some((connref.clone(), val))
        })
        .collect();

    let mut extent_x = 0.0;
    let mut extent_y = 0.0;
    let mut width = 9600.0;
    let mut height = 7200.0;
    for p in &sgd.parameters {
        for pe in &p.paramextent {
            extent_x = pe.x.unwrap_or(0.0);
            extent_y = pe.y.unwrap_or(0.0);
            width = pe.width.unwrap_or(9600.0);
            let h = pe.height.unwrap_or(0.0);
            height = if h > 0.0 { h } else { width * 0.75 };
        }
    }

    let mut outline: Vec<ConnectorOutline> = Vec::new();
    let mut bounds_points: Vec<(f64, f64)> = Vec::new();

    collect_outline_from_schemgrounddevice(sgd, &mut outline, &mut bounds_points);

    let mut seen: HashSet<String> = HashSet::new();
    let mut to_attach: Vec<&str> = sgd
        .refattachschem
        .iter()
        .filter_map(|r| r.attachref.as_deref())
        .collect();
    while let Some(attachref) = to_attach.pop() {
        if seen.contains(attachref) {
            continue;
        }
        seen.insert(attachref.to_string());
        if let Some(attached) = schemgrounddevice_by_id.get(attachref) {
            collect_outline_from_schemgrounddevice(attached, &mut outline, &mut bounds_points);
            for r in &attached.refattachschem {
                if let Some(ref id) = r.attachref {
                    if !seen.contains(id) {
                        to_attach.push(id);
                    }
                }
            }
        }
    }

    if !bounds_points.is_empty() && extent_x == 0.0 && extent_y == 0.0 {
        let (min_x, max_x) = bounds_points
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), (x, _)| (a.min(*x), b.max(*x)));
        let (min_y, max_y) = bounds_points
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), (_, y)| (a.min(*y), b.max(*y)));
        let pad = 480.0;
        extent_x = min_x - pad;
        extent_y = min_y - pad;
        width = (max_x - min_x + 2.0 * pad).max(960.0);
        height = (max_y - min_y + 2.0 * pad).max(480.0);
    }

    let mut pins: Vec<DevicePin> = Vec::new();
    let mut pin_index = 0;
    for sp in &sgd.schempin {
        pin_index += 1;
        let x = sp.x.unwrap_or(0.0);
        let y = sp.y.unwrap_or(0.0);
        let transform = sp
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let pin_id = sp
            .connref
            .as_ref()
            .or_else(|| {
                sp.id.as_ref().and_then(|sid| {
                    schempin_to_logicpin.get(&(diagram_id.to_string(), sid.clone()))
                })
            });
        let short_description = pin_id
            .and_then(|lp| device_pins.get(&connref).and_then(|pins| pins.get(lp)))
            .or_else(|| pin_id.and_then(|lp| pin_shortdescription.get(lp)))
            .filter(|s| !s.is_empty())
            .map(|s| decode_entity(s));
        let mut name_placement = None;
        let mut short_description_placement = None;
        let mut pin_name_str = pin_id
            .and_then(|id| pin_name.get(id).cloned())
            .or_else(|| pin_id.and_then(|id| pin_value_names.get(id).cloned()))
            .or_else(|| sp.connref.as_ref().and_then(|c| pin_value_names.get(c).cloned()))
            .or_else(|| sp.id.as_ref().and_then(|id| pin_value_names.get(id).cloned()))
            .or_else(|| {
                sp.id
                    .as_ref()
                    .and_then(|sid| {
                        schempin_to_logicpin
                            .get(&(diagram_id.to_string(), sid.clone()))
                            .cloned()
                    })
            })
            .or_else(|| {
                let c = sp.connref.as_ref()?;
                let s = c.as_str();
                if s == connref.as_str() || s.starts_with('_') {
                    None
                } else {
                    Some(c.clone())
                }
            })
            .unwrap_or_else(|| pin_index.to_string());
        let mut name_visible = true;
        for at in &sp.attributetext {
            if at.nameref.as_deref() == Some("Name") {
                name_placement = attributetext_to_placement(at);
                name_visible = is_visible(at.visibility.as_ref());
                if let Some(ref v) = at.val {
                    let decoded = decode_entity(v);
                    if !decoded.is_empty() {
                        pin_name_str = decoded;
                    }
                }
            } else if at.nameref.as_deref() == Some("ShortDescription")
                || at.nameref.as_deref() == Some("DESCRIPTION")
            {
                short_description_placement = attributetext_to_placement(at);
            }
        }
        let is_internal = pin_id
            .as_ref()
            .map_or(false, |id| internal_pin_ids.contains(id.as_str()));
        if !is_internal {
            pins.push(DevicePin {
                x,
                y,
                text: pin_name_str,
                short_description,
                name_placement,
                short_description_placement,
                name_visible,
                transform,
            });
        }
    }

    Some(DeviceSymbol {
        name,
        connref,
        attributeref: sgd.attributeref.clone(),
        name_font,
        name_style,
        name_placement,
        x: sgd.x.unwrap_or(0.0),
        y: sgd.y.unwrap_or(0.0),
        extent_x,
        extent_y,
        width,
        height,
        transform: sgd
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        pins,
        outline,
        bounds: None,
    })
}

fn convert_schemconnector(
    sc: &XmlSchemconnector,
    diagram_id: &str,
    _diagram_id_to_name: &HashMap<String, String>,
    pin_name: &HashMap<String, String>,
    schempin_to_logicpin: &HashMap<(String, String), String>,
    connector_type: &HashMap<String, ConnectorKind>,
) -> Option<ConnectorSymbol> {
    let connref = sc.connref.clone().unwrap_or_default();
    // Use type="plug" | type="jack" from <connector> tag (connref = connector id)
    let kind = connector_type
        .get(&connref)
        .copied()
        .unwrap_or(ConnectorKind::Jack);

    let mut name = String::new();
    let mut name_font = None;
    let mut name_style = TextStyle::Plain;
    let mut name_placement = None;
    for at in &sc.attributetext {
        if at.nameref.as_deref() == Some("Name") {
            if let Some(v) = &at.val {
                name = decode_entity(v);
            }
            name_font = at.font.clone();
            name_style = at.style.as_deref().map(parse_text_style).unwrap_or(TextStyle::Plain);
            name_placement = attributetext_to_placement(at);
            break;
        }
    }
    for ve in &sc.valueentry {
        // Only use valueentry without connref for connector label (entries with connref are pin-specific).
        if ve.connref.is_none() {
            if let Some(v) = &ve.val {
                name.push_str(&decode_entity(v));
            }
        }
    }

    // Pin names from connector-level valueentry with connref (same as device custom symbols).
    let conn_pin_value_names: std::collections::HashMap<String, String> = sc
        .valueentry
        .iter()
        .filter_map(|ve| {
            let cr = ve.connref.as_ref()?;
            let val = ve.val.as_ref().map(|s| decode_entity(s)).filter(|s| !s.is_empty())?;
            Some((cr.clone(), val))
        })
        .collect();

    let mut extent_x = 0.0;
    let mut extent_y = 0.0;
    let mut width = 0.0;
    let mut height = 0.0;
    for p in &sc.parameters {
        for pe in &p.paramextent {
            extent_x = pe.x.unwrap_or(0.0);
            extent_y = pe.y.unwrap_or(0.0);
            width = pe.width.unwrap_or(0.0);
            let h = pe.height.unwrap_or(0.0);
            height = if h > 0.0 { h } else { width * 0.25 };
        }
    }

    let mut outline: Vec<ConnectorOutline> = Vec::new();
    for l in &sc.line {
        if is_visible(l.visibility.as_ref()) {
            outline.push(ConnectorOutline::Line {
                x1: l.x1.unwrap_or(0.0),
                y1: l.y1.unwrap_or(0.0),
                x2: l.x2.unwrap_or(0.0),
                y2: l.y2.unwrap_or(0.0),
            });
        }
    }
    for a in &sc.arc {
        if is_visible(a.visibility.as_ref()) {
            outline.push(ConnectorOutline::Arc {
                x: a.x.unwrap_or(0.0),
                y: a.y.unwrap_or(0.0),
                radius: a.radius.unwrap_or(0.0),
                start_angle: a.startangle.unwrap_or(0.0),
                travel_angle: a.travelangle.unwrap_or(0.0),
            });
        }
    }
    for c in &sc.circle {
        if is_visible(c.visibility.as_ref()) && c.radius.unwrap_or(0.0) > 0.0 {
            outline.push(ConnectorOutline::Circle {
                x: c.x.unwrap_or(0.0),
                y: c.y.unwrap_or(0.0),
                radius: c.radius.unwrap_or(0.0),
            });
        }
    }
    for r in &sc.rectangle {
        if is_visible(r.visibility.as_ref()) {
            let w = r.width.unwrap_or(0.0);
            let h = r.height.unwrap_or(0.0);
            let height_val = if h > 0.0 { h } else { w * 0.25 };
            if w > 0.0 && width == 0.0 {
                extent_x = r.x.unwrap_or(0.0);
                extent_y = r.y.unwrap_or(0.0);
                width = w;
                height = height_val;
            }
            outline.push(ConnectorOutline::Rectangle {
                x: r.x.unwrap_or(0.0),
                y: r.y.unwrap_or(0.0),
                width: w,
                height: height_val,
            });
        }
    }
    for pl in &sc.polyline {
        if is_visible(pl.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pl
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polyline { points: pts });
            }
        }
    }
    for pg in &sc.polygon {
        if is_visible(pg.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pg
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polygon { points: pts });
            }
        }
    }
    for cu in &sc.curve {
        if is_visible(cu.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = cu
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Curve { points: pts });
            }
        }
    }
    for wa in &sc.widtharc {
        if is_visible(wa.visibility.as_ref()) {
            if let (Some(x), Some(y), Some(r), Some(sa), Some(ta), Some(w)) = (
                wa.x,
                wa.y,
                wa.radius,
                wa.startangle,
                wa.travelangle,
                wa.width,
            ) {
                if r > 0.0 && w > 0.0 {
                    outline.push(ConnectorOutline::WidthArc {
                        x,
                        y,
                        radius: r,
                        start_angle: sa,
                        travel_angle: ta,
                        width: w,
                    });
                }
            }
        }
    }

    let mut pins: Vec<ConnectorPin> = Vec::new();
    let mut pin_index = 0;
    for sp in &sc.schempin {
        pin_index += 1;
        let x = sp.x.unwrap_or(0.0);
        let y = sp.y.unwrap_or(0.0);
        let transform = sp
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        let pin_id = sp
            .connref
            .as_ref()
            .or_else(|| {
                sp.id.as_ref().and_then(|sid| {
                    schempin_to_logicpin.get(&(diagram_id.to_string(), sid.clone()))
                })
            });
        // Connector pin short_description: use only this pin's own attributetext value.
        let mut short_description = None;
        let mut name_placement = None;
        let mut short_description_placement = None;
        let mut name_visible = true;
        // Display name: logical pin name, then valueentry, then logicpin, else safe connref, else index.
        // Connectivity often stores connector pin ids as "J200.1", "J200.2" while designsharedpinusage gives logicpin "1", "2" - try both.
        let mut pin_name_str = pin_id
            .and_then(|id| pin_name.get(id).cloned())
            .or_else(|| {
                pin_id.and_then(|id| {
                    let key_dot = format!("{}.{}", connref, id);
                    let key_dash = format!("{}-{}", connref, id);
                    pin_name.get(&key_dot)
                        .or_else(|| pin_name.get(&key_dash))
                        .cloned()
                })
            })
            .or_else(|| pin_id.and_then(|id| conn_pin_value_names.get(id).cloned()))
            .or_else(|| sp.connref.as_ref().and_then(|c| conn_pin_value_names.get(c).cloned()))
            .or_else(|| sp.id.as_ref().and_then(|id| conn_pin_value_names.get(id).cloned()))
            .or_else(|| {
                sp.id.as_ref().and_then(|sid| {
                    schempin_to_logicpin
                        .get(&(diagram_id.to_string(), sid.clone()))
                        .cloned()
                })
            })
            .or_else(|| {
                let c = sp.connref.as_ref()?;
                let s = c.as_str();
                if s == connref.as_str() || s.starts_with('_') {
                    None
                } else {
                    Some(c.clone())
                }
            })
            .unwrap_or_else(|| pin_index.to_string());
        for at in &sp.attributetext {
            if at.nameref.as_deref() == Some("Name") {
                name_placement = attributetext_to_placement(at);
                name_visible = is_visible(at.visibility.as_ref());
                if let Some(ref v) = at.val {
                    let decoded = decode_entity(v);
                    if !decoded.is_empty() {
                        pin_name_str = decoded;
                    }
                }
            } else if at.nameref.as_deref() == Some("ShortDescription")
                || at.nameref.as_deref() == Some("DESCRIPTION")
            {
                short_description_placement = attributetext_to_placement(at);
                if short_description.is_none() {
                    short_description = at
                        .val
                        .as_ref()
                        .map(|s| decode_entity(s))
                        .filter(|s| !s.is_empty());
                }
            }
        }
        pins.push(ConnectorPin {
            x,
            y,
            text: pin_name_str,
            short_description,
            name_placement,
            short_description_placement,
            name_visible,
            transform,
        });
    }

    if width <= 0.0 && !pins.is_empty() {
        let (min_x, max_x) = pins
            .iter()
            .map(|p| p.x)
            .fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let (min_y, max_y) = pins
            .iter()
            .map(|p| p.y)
            .fold((f64::MAX, f64::MIN), |(a, b), y| (a.min(y), b.max(y)));
        extent_x = min_x;
        extent_y = min_y;
        width = (max_x - min_x).max(960.0);
        height = (max_y - min_y).max(480.0);
    } else if width <= 0.0 {
        width = 960.0;
        height = 480.0;
    }

    // Include xref even when visibility=false so users can navigate; Capital often hides them by default.
    let cross_references: Vec<CrossReference> = sc
        .xreftextcontainer
        .iter()
        .flat_map(|xc| &xc.xreftext)
        .filter_map(|xt| {
            let text = xt.blocktext.as_ref()?.trim();
            if text.is_empty() {
                return None;
            }
            let diagram_name = parse_xref_diagram_name(text);
            let placement = AttributeTextPlacement {
                x: xt.x.unwrap_or(0.0),
                y: xt.y.unwrap_or(0.0),
                hjust: parse_hjust(xt.hjust.as_ref()),
                vjust: parse_vjust(xt.vjust.as_ref()),
                rotation: xt.rotation.unwrap_or(0.0),
                height: xt.height.unwrap_or(1382.0),
            };
            Some(CrossReference {
                text: text.to_string(),
                diagram_name,
                placement,
            })
        })
        .collect();

    Some(ConnectorSymbol {
        kind,
        name,
        connref,
        attributeref: sc.attributeref.clone(),
        name_font,
        name_style,
        name_placement,
        x: sc.x.unwrap_or(0.0),
        y: sc.y.unwrap_or(0.0),
        extent_x,
        extent_y,
        width,
        height,
        transform: sc
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        pins,
        outline,
        cross_references,
        bounds: None,
    })
}

fn convert_schemindicator(si: &XmlSchemindicator) -> Option<Schemindicator> {
    let connref = si.connref.clone().unwrap_or_default();
    let indicatortype = si
        .indicatortype
        .clone()
        .unwrap_or_else(|| "oval".to_string());
    let x = si.x.unwrap_or(0.0);
    let y = si.y.unwrap_or(0.0);
    let transform = si
        .transform
        .as_deref()
        .map(parse_transform)
        .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    // Center decoration from parameters (vline for shield, x for twist)
    let mut center_decoration = None;
    for p in &si.parameters {
        for param in &p.parameter {
            if param.type_.as_deref() == Some("center") {
                if let Some(glyph) = param.glyph.first() {
                    center_decoration = glyph.name.clone();
                    break;
                }
            }
        }
    }
    // Shield: oval with parameter type="shield" or center glyph name="vline"
    if center_decoration.is_none() {
        for p in &si.parameters {
            for param in &p.parameter {
                if param.type_.as_deref() == Some("shield") {
                    center_decoration = Some("vline".to_string());
                    break;
                }
            }
        }
    }

    let mut outline: Vec<ConnectorOutline> = Vec::new();
    for a in &si.arc {
        if is_visible(a.visibility.as_ref()) {
            outline.push(ConnectorOutline::Arc {
                x: a.x.unwrap_or(0.0),
                y: a.y.unwrap_or(0.0),
                radius: a.radius.unwrap_or(0.0),
                start_angle: a.startangle.unwrap_or(0.0),
                travel_angle: a.travelangle.unwrap_or(0.0),
            });
        }
    }
    for l in &si.line {
        if is_visible(l.visibility.as_ref()) {
            outline.push(ConnectorOutline::Line {
                x1: l.x1.unwrap_or(0.0),
                y1: l.y1.unwrap_or(0.0),
                x2: l.x2.unwrap_or(0.0),
                y2: l.y2.unwrap_or(0.0),
            });
        }
    }
    for pl in &si.polyline {
        if is_visible(pl.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pl
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polyline { points: pts });
            }
        }
    }

    Some(Schemindicator {
        connref,
        indicatortype,
        x,
        y,
        transform,
        attributeref: si.attributeref.clone(),
        outline,
        center_decoration,
        bounds: None,
    })
}

fn convert_schemsplice(
    ss: &XmlSchemsplice,
    _diagram_id: &str,
    _diagram_id_to_name: &HashMap<String, String>,
) -> Option<SpliceSymbol> {
    let connref = ss.connref.clone().unwrap_or_default();

    let mut name = String::new();
    let mut name_font = None;
    let mut name_style = TextStyle::Plain;
    let mut name_placement = None;
    for at in &ss.attributetext {
        if at.nameref.as_deref() == Some("Name") {
            if let Some(v) = &at.val {
                name = decode_entity(v);
            }
            name_font = at.font.clone();
            name_style = at.style.as_deref().map(parse_text_style).unwrap_or(TextStyle::Plain);
            name_placement = attributetext_to_placement(at);
            break;
        }
    }
    for ve in &ss.valueentry {
        if ve.connref.is_none() {
            if let Some(v) = &ve.val {
                name.push_str(&decode_entity(v));
            }
        }
    }
    if name.is_empty() {
        name = connref.clone();
    }

    let mut outline: Vec<ConnectorOutline> = Vec::new();
    for c in &ss.circle {
        if is_visible(c.visibility.as_ref()) && c.radius.unwrap_or(0.0) > 0.0 {
            outline.push(ConnectorOutline::Circle {
                x: c.x.unwrap_or(0.0),
                y: c.y.unwrap_or(0.0),
                radius: c.radius.unwrap_or(0.0),
            });
        }
    }
    for l in &ss.line {
        if is_visible(l.visibility.as_ref()) {
            outline.push(ConnectorOutline::Line {
                x1: l.x1.unwrap_or(0.0),
                y1: l.y1.unwrap_or(0.0),
                x2: l.x2.unwrap_or(0.0),
                y2: l.y2.unwrap_or(0.0),
            });
        }
    }
    for a in &ss.arc {
        if is_visible(a.visibility.as_ref()) {
            outline.push(ConnectorOutline::Arc {
                x: a.x.unwrap_or(0.0),
                y: a.y.unwrap_or(0.0),
                radius: a.radius.unwrap_or(0.0),
                start_angle: a.startangle.unwrap_or(0.0),
                travel_angle: a.travelangle.unwrap_or(0.0),
            });
        }
    }
    for r in &ss.rectangle {
        if is_visible(r.visibility.as_ref()) {
            let w = r.width.unwrap_or(0.0);
            let h = r.height.unwrap_or(0.0);
            let height_val = if h > 0.0 { h } else { w * 0.25 };
            if w > 0.0 {
                outline.push(ConnectorOutline::Rectangle {
                    x: r.x.unwrap_or(0.0),
                    y: r.y.unwrap_or(0.0),
                    width: w,
                    height: height_val,
                });
            }
        }
    }
    for pl in &ss.polyline {
        if is_visible(pl.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pl
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polyline { points: pts });
            }
        }
    }
    for pg in &ss.polygon {
        if is_visible(pg.visibility.as_ref()) {
            let pts: Vec<(f64, f64)> = pg
                .point
                .iter()
                .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                .collect();
            if !pts.is_empty() {
                outline.push(ConnectorOutline::Polygon { points: pts });
            }
        }
    }

    // Default circle if no outline (splice symbol is typically a circle)
    if outline.is_empty() {
        outline.push(ConnectorOutline::Circle {
            x: 0.0,
            y: 0.0,
            radius: 768.0,
        });
    }

    let cross_references: Vec<CrossReference> = ss
        .xreftextcontainer
        .iter()
        .flat_map(|xc| &xc.xreftext)
        .filter_map(|xt| {
            let text = xt.blocktext.as_ref()?.trim();
            if text.is_empty() {
                return None;
            }
            let diagram_name = parse_xref_diagram_name(text);
            let placement = AttributeTextPlacement {
                x: xt.x.unwrap_or(0.0),
                y: xt.y.unwrap_or(0.0),
                hjust: parse_hjust(xt.hjust.as_ref()),
                vjust: parse_vjust(xt.vjust.as_ref()),
                rotation: xt.rotation.unwrap_or(0.0),
                height: xt.height.unwrap_or(1382.0),
            };
            Some(CrossReference {
                text: text.to_string(),
                diagram_name,
                placement,
            })
        })
        .collect();

    Some(SpliceSymbol {
        name,
        connref,
        x: ss.x.unwrap_or(0.0),
        y: ss.y.unwrap_or(0.0),
        transform: ss
            .transform
            .as_deref()
            .map(parse_transform)
            .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        attributeref: ss.attributeref.clone(),
        name_font,
        name_style,
        name_placement,
        outline,
        cross_references,
        bounds: None,
    })
}
