//! In-memory model for Capital Essentials design: design metadata, diagram list,
//! connectivity index (devices, splices, connectors, wires), and cached diagram content.

use std::collections::{HashMap, HashSet};

/// Cross-reference key: (Type Of, Device, Name) - e.g. ("Device", "RLY", "K7"), ("Wire", "", "123").
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct CrossRefKey(pub String, pub String, pub String);

/// Cross-reference target: diagram name and logical coordinates.
#[derive(Clone, Debug)]
pub struct CrossRefTarget(pub String, pub (f64, f64));

/// Map from (Type Of, Device, Name) to list of (Diagram, Coord) for right-click menu.
pub type CrossRefMap = std::collections::HashMap<CrossRefKey, Vec<CrossRefTarget>>;

/// Design-level data: project name, list of diagrams, connectivity index.
#[derive(Default)]
pub struct Design {
    pub project_name: String,
    /// Diagrams in display order (from designsharedusagemgr).
    pub diagrams: Vec<DiagramInfo>,
    pub connectivity: ConnectivityIndex,
    /// Cross-reference map: (Type Of, Device, Name) -> [(Diagram, Coord)]. Built from diagram content, not xrefs.
    pub cross_ref_map: CrossRefMap,
    /// Pin id -> shortdescription from logical design (device/deviceconnector pins).
    /// Propagated through connectedpin so connector pins get device pin descriptions.
    pub pin_shortdescription: HashMap<String, String>,
    /// (diagram_id, schempin_id) -> logicpin_id from designsharedpinusage.
    /// Used to resolve ShortDescription when diagram attributetext has visibility but no val.
    pub schempin_to_logicpin: HashMap<(String, String), String>,
    /// Pin connections (a, b) for propagation of shortdescription across nets.
    pub pin_connections: Vec<(String, String)>,
    /// device_id -> (pin_id -> shortdescription). Lookup device by connref, read pin attributes.
    pub device_pins: HashMap<String, HashMap<String, String>>,
    /// logicpin_id -> display name (for pin name resolution when valueentry is missing).
    pub pin_name: HashMap<String, String>,
    /// Pin IDs that are internal pins (from &lt;internalpin&gt; tag); these are hidden from display.
    pub internal_pin_ids: HashSet<String>,
    /// Connector id -> Plug/Jack from &lt;connector type="plug"|"jack"&gt;.
    pub connector_type: HashMap<String, ConnectorKind>,
    /// Element id (connref) -> shortdescription from connectivity (device, connector, splice).
    pub element_shortdescription: HashMap<String, String>,
    /// Wire id (sharedconductor id / harnwire) -> shortdescription from sharedconductormgr.
    pub wire_shortdescription: HashMap<String, String>,
}

/// One diagram/sheet in the design (id and name from diagramsharedusagemgr).
#[derive(Clone)]
pub struct DiagramInfo {
    pub id: String,
    pub name: String,
    /// If false, diagram is hidden from the list (from property VISIBLE).
    pub visible: bool,
}

/// Connectivity index: elements grouped by type for the element tree.
#[derive(Default)]
pub struct ConnectivityIndex {
    pub devices: Vec<ElementRef>,
    pub splices: Vec<ElementRef>,
    pub connectors: Vec<ElementRef>,
    pub wires: Vec<WireRef>,
}

/// Reference to a device, splice, or connector instance (for tree and "open diagram").
#[derive(Clone)]
pub struct ElementRef {
    pub id: String,
    pub name: String,
    /// Diagram name where this instance is placed (for opening the right page).
    pub diagram_name: String,
    /// Optional: type/category label (e.g. typecode "LAMP", "RLY" for devices).
    pub type_label: Option<String>,
}

/// Reference to a wire/conductor; can appear on multiple diagrams.
#[derive(Clone)]
pub struct WireRef {
    pub id: String,
    pub name: String,
    /// First (or primary) diagram name where this wire is shown.
    pub diagram_name: String,
}

/// Connector symbol type: plug (rounded, pink) or jack (square, blue).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorKind {
    Plug,
    Jack,
}

/// One pin on a connector symbol.
#[derive(Clone)]
pub struct ConnectorPin {
    pub x: f64,
    pub y: f64,
    pub text: String,
    /// Pin short description (from attributetext nameref="ShortDescription" or "DESCRIPTION"), when visible.
    pub short_description: Option<String>,
    /// Placement for pin name (attributetext nameref="Name") in pin-local coordinates.
    pub name_placement: Option<AttributeTextPlacement>,
    /// Placement for short description (attributetext nameref="ShortDescription") in pin-local coordinates.
    pub short_description_placement: Option<AttributeTextPlacement>,
    pub transform: [f64; 6],
    /// Whether to show pin name (from attributetext Name visibility).
    pub name_visible: bool,
}

/// Geometry primitive for connector outline (in connector-local coordinates).
#[derive(Clone)]
pub enum ConnectorOutline {
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    Rectangle { x: f64, y: f64, width: f64, height: f64 },
    Circle { x: f64, y: f64, radius: f64 },
    Arc {
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        travel_angle: f64,
    },
    /// Pie-wedge arc (arc with radial width).
    WidthArc {
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        travel_angle: f64,
        width: f64,
    },
    Polyline { points: Vec<(f64, f64)> },
    /// Closed polygon (filled).
    Polygon { points: Vec<(f64, f64)> },
    /// Smooth curve through control points (B-spline/bezier).
    Curve { points: Vec<(f64, f64)> },
}

/// One pin on a device symbol (from schemdeviceconnector or schempin).
#[derive(Clone)]
pub struct DevicePin {
    pub x: f64,
    pub y: f64,
    pub text: String,
    /// Pin short description (from attributetext nameref="ShortDescription" or "DESCRIPTION"), when visible.
    pub short_description: Option<String>,
    /// Placement for pin name (attributetext nameref="Name") in pin-local coordinates.
    pub name_placement: Option<AttributeTextPlacement>,
    /// Placement for short description (attributetext nameref="ShortDescription") in pin-local coordinates.
    pub short_description_placement: Option<AttributeTextPlacement>,
    pub transform: [f64; 6],
    /// Whether to show pin name (from attributetext Name visibility).
    pub name_visible: bool,
}

/// Device symbol instance (generic rectangle or complex outline).
#[derive(Clone)]
pub struct DeviceSymbol {
    pub name: String,
    /// Connectivity reference (matches ElementRef.id for name resolution).
    pub connref: String,
    /// Optional reference to datadictionary for color/thickness (attributeref from schemdevice).
    pub attributeref: Option<String>,
    /// Font family for name label (from attributetext nameref="Name").
    pub name_font: Option<String>,
    /// Style for name label (plain, bold, italic, bolditalic).
    pub name_style: TextStyle,
    /// Placement for name label from attributetext nameref="Name" (x, y, hjust, vjust in symbol-local coords).
    pub name_placement: Option<AttributeTextPlacement>,
    pub x: f64,
    pub y: f64,
    pub extent_x: f64,
    pub extent_y: f64,
    pub width: f64,
    pub height: f64,
    pub transform: [f64; 6],
    pub pins: Vec<DevicePin>,
    /// Outline geometry (line, arc, polyline) for complex symbols.
    pub outline: Vec<ConnectorOutline>,
    /// Precomputed bounding rect (min_x, min_y, max_x, max_y) in logical coords. Used for grid extent and hit tests.
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// Cross-reference text (e.g. "/0 Master") that links to another diagram.
#[derive(Clone)]
pub struct CrossReference {
    pub text: String,
    /// Parsed diagram name for navigation (e.g. "0 Master" from "/0 Master").
    pub diagram_name: Option<String>,
    pub placement: AttributeTextPlacement,
}

/// Connector symbol instance (plug or jack) with pins.
#[derive(Clone)]
pub struct ConnectorSymbol {
    pub kind: ConnectorKind,
    pub name: String,
    /// Connectivity reference (matches ElementRef.id for name resolution).
    pub connref: String,
    /// Optional reference to datadictionary for color/thickness (attributeref from schemconnector).
    pub attributeref: Option<String>,
    /// Font family for name label (from attributetext nameref="Name").
    pub name_font: Option<String>,
    /// Style for name label (plain, bold, italic, bolditalic).
    pub name_style: TextStyle,
    /// Placement for name label from attributetext nameref="Name" (x, y, hjust, vjust in symbol-local coords).
    pub name_placement: Option<AttributeTextPlacement>,
    pub x: f64,
    pub y: f64,
    /// paramextent origin (local coords)
    pub extent_x: f64,
    pub extent_y: f64,
    pub width: f64,
    pub height: f64,
    pub transform: [f64; 6],
    pub pins: Vec<ConnectorPin>,
    /// Outline geometry (polyline, arc, line) in connector-local coords.
    pub outline: Vec<ConnectorOutline>,
    /// Cross-reference text (e.g. "/0 Master") linking to other diagrams.
    pub cross_references: Vec<CrossReference>,
    /// Precomputed bounding rect (min_x, min_y, max_x, max_y) in logical coords. Used for grid extent and hit tests.
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// Splice symbol instance (typically a circle where wires meet).
#[derive(Clone)]
pub struct SpliceSymbol {
    pub name: String,
    /// Connectivity reference (matches ElementRef.id for name resolution).
    pub connref: String,
    pub x: f64,
    pub y: f64,
    pub transform: [f64; 6],
    /// Optional reference to datadictionary for color/thickness.
    pub attributeref: Option<String>,
    /// Font family for name label.
    pub name_font: Option<String>,
    /// Style for name label (plain, bold, italic, bolditalic).
    pub name_style: TextStyle,
    /// Placement for name label (x, y, hjust, vjust in symbol-local coords).
    pub name_placement: Option<AttributeTextPlacement>,
    /// Outline geometry (typically a circle at origin).
    pub outline: Vec<ConnectorOutline>,
    /// Cross-reference text linking to other diagrams.
    pub cross_references: Vec<CrossReference>,
    /// Precomputed bounding rect (min_x, min_y, max_x, max_y) in logical coords. Used for grid extent and hit tests.
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// Schematic indicator (oval, twist, shield) on a wire.
#[derive(Clone)]
pub struct Schemindicator {
    /// Wire/conductor reference.
    pub connref: String,
    /// Indicator type: "oval", "twist-commercial", "shield", etc.
    pub indicatortype: String,
    pub x: f64,
    pub y: f64,
    pub transform: [f64; 6],
    /// Optional reference to datadictionary for color/thickness.
    pub attributeref: Option<String>,
    /// Outline geometry (arcs, lines, polylines) in indicator-local coords.
    pub outline: Vec<ConnectorOutline>,
    /// Center decoration: "vline" (shield), "x" (twist), etc.
    pub center_decoration: Option<String>,
    /// Precomputed bounding rect (min_x, min_y, max_x, max_y) in logical coords.
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// Cached content for one diagram: border bounds and wires for drawing.
#[derive(Clone, Default)]
pub struct DiagramContent {
    pub border: Option<BorderBounds>,
    /// Wires, each with its segments. Used for drawing and wire selection.
    pub wires: Vec<Wire>,
    /// Connector symbols (plug/jack) with pins.
    pub connectors: Vec<ConnectorSymbol>,
    /// Splice symbols (wire junction points).
    pub splices: Vec<SpliceSymbol>,
    /// Device symbols (generic rectangles or complex outlines).
    pub devices: Vec<DeviceSymbol>,
    /// Schematic indicators (oval, twist, shield) on wires.
    pub schemindicators: Vec<Schemindicator>,
    /// Optional: id -> color/thickness for attributeref resolution.
    pub datadictionary: HashMap<String, DdAttribute>,
}

/// Border bounds from diagramcontent/border (paper or global bounds).
#[derive(Clone)]
pub struct BorderBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub units: String,
}

/// Horizontal justification for text (left, right, center).
#[derive(Clone, Copy, Debug, Default)]
pub enum HorizontalJust {
    #[default]
    Left,
    Right,
    Center,
}

/// Vertical justification for text (top, bottom, center).
#[derive(Clone, Copy, Debug, Default)]
pub enum VerticalJust {
    #[default]
    Top,
    Bottom,
    Center,
}

/// Capital XML font style: plain, bold, italic, or bolditalic.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextStyle {
    #[default]
    Plain,
    Bold,
    Italic,
    BoldItalic,
}

/// Placement for attribute text (Name label) from attributetext x, y, hjust, vjust.
#[derive(Clone, Debug, Default)]
pub struct AttributeTextPlacement {
    /// X in symbol-local coordinates.
    pub x: f64,
    /// Y in symbol-local coordinates.
    pub y: f64,
    pub hjust: HorizontalJust,
    pub vjust: VerticalJust,
    /// Rotation in degrees.
    pub rotation: f64,
    /// Font height in logical units (e.g. 1920, 1382).
    pub height: f64,
}

/// Text label on a wire segment (wire name, gauge, color, etc.).
#[derive(Clone)]
pub struct WireTextItem {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub hjust: HorizontalJust,
    pub vjust: VerticalJust,
    pub rotation: f64,
    /// Font family name from XML (e.g. "Stroke", "Century Gothic").
    pub font: Option<String>,
    /// Text style from XML (plain, bold, italic, bolditalic).
    pub style: TextStyle,
    /// True if this is the wire name (decorationname="Name_1" etc.); false for gauge/color.
    pub is_wire_name: bool,
    /// True if this is cross-reference text (e.g. "/0 Master") linking to another diagram.
    pub is_xref: bool,
    /// Parsed diagram name for navigation when is_xref (e.g. "0 Master" from "/0 Master").
    pub xref_diagram_name: Option<String>,
}

/// One segment of a schematic wire (x1,y1 -> x2,y2 in logical units).
#[derive(Clone)]
pub struct WireSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    /// Optional reference into diagram's datadictionary for color/thickness.
    pub attributeref: Option<String>,
    /// Text labels on this segment (wire name, gauge, color).
    pub text_items: Vec<WireTextItem>,
    /// Precomputed bounding rect (min_x, min_y, max_x, max_y) in logical coords. Used for grid extent and hit tests.
    pub bounds: Option<(f64, f64, f64, f64)>,
}

/// A wire: logical conductor with one or more segments. Segments are ordered along the wire path.
#[derive(Clone, Default)]
pub struct Wire {
    /// Wire name for selection (from first segment's text or connectivity). Matches WireRef.name.
    pub name: String,
    /// Wire id from connectivity if available. Matches WireRef.id. Empty for diagram-only wires.
    pub id: String,
    /// Segments belonging to this wire.
    pub segments: Vec<WireSegment>,
}

/// Data dictionary attribute (color, thickness, linestyle) for rendering.
#[derive(Clone)]
pub struct DdAttribute {
    pub color: Option<String>,
    pub thickness: Option<f64>,
    pub linestyle: Option<String>,
}
