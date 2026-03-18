//! Parser for Capital Symbol Library XML (e.g. Firefly Pin Comment Symbols.xml).
//!
//! Structure: symbollib > symbol > gfx > propertiedgraphic > line | polyline

use std::collections::HashMap;
use std::io::{BufRead, Read};

use hard_xml::XmlRead;

// ---------------------------------------------------------------------------
// hard_xml structs for symbollib schema
// ---------------------------------------------------------------------------

#[derive(Debug, XmlRead)]
#[xml(tag = "symbollib")]
struct XmlSymbollib {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "scaletype")]
    scaletype: Option<String>,
    #[xml(attr = "realmapping")]
    realmapping: Option<String>,
    #[xml(attr = "realmappingunits")]
    realmappingunits: Option<String>,
    #[xml(child = "symbol")]
    symbol: Vec<XmlSymbol>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "symbol")]
struct XmlSymbol {
    #[xml(attr = "id")]
    id: Option<String>,
    #[xml(attr = "name")]
    name: Option<String>,
    #[xml(attr = "type")]
    type_: Option<String>,
    #[xml(attr = "subtype")]
    subtype: Option<String>,
    #[xml(child = "gfx")]
    gfx: Vec<XmlGfx>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "gfx")]
struct XmlGfx {
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(child = "propertiedgraphic")]
    propertiedgraphic: Vec<XmlPropertiedgraphic>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "propertiedgraphic")]
struct XmlPropertiedgraphic {
    #[xml(child = "line")]
    line: Vec<XmlSymLine>,
    #[xml(child = "polyline")]
    polyline: Vec<XmlSymPolyline>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "line")]
struct XmlSymLine {
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
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "polyline")]
struct XmlSymPolyline {
    #[xml(attr = "visibility")]
    visibility: Option<String>,
    #[xml(attr = "attributeref")]
    attributeref: Option<String>,
    #[xml(child = "point")]
    point: Vec<XmlSymPoint>,
}

#[derive(Debug, XmlRead)]
#[xml(tag = "point")]
struct XmlSymPoint {
    #[xml(attr = "x")]
    x: Option<f64>,
    #[xml(attr = "y")]
    y: Option<f64>,
}

// ---------------------------------------------------------------------------
// Parsed model
// ---------------------------------------------------------------------------

/// Geometry for one symbol: lines and polylines in symbol-local coordinates.
#[derive(Clone, Debug, Default)]
pub struct SymbolGfx {
    pub lines: Vec<(f64, f64, f64, f64)>,
    pub polylines: Vec<Vec<(f64, f64)>>,
}

/// One symbol from a library (id, name, geometry).
#[derive(Clone, Debug)]
pub struct SymbolDef {
    pub id: String,
    pub name: String,
    pub type_: String,
    pub subtype: Option<String>,
    pub gfx: SymbolGfx,
}

/// Parsed symbol library: id -> SymbolDef.
#[derive(Clone, Debug, Default)]
pub struct SymbolLibrary {
    pub name: String,
    pub scaletype: Option<String>,
    pub realmapping: Option<String>,
    pub realmappingunits: Option<String>,
    /// symbol id -> definition
    pub symbols: HashMap<String, SymbolDef>,
}

fn is_visible(s: Option<&String>) -> bool {
    s.map(|v| v != "false").unwrap_or(true)
}

/// Parse a symbol library XML file.
pub fn parse_symbol_library<R: BufRead>(reader: R) -> Result<SymbolLibrary, String> {
    let mut xml = String::new();
    reader
        .take(50 * 1024 * 1024)
        .read_to_string(&mut xml)
        .map_err(|e| format!("Read error: {}", e))?;

    let lib: XmlSymbollib = XmlSymbollib::from_str(&xml)
        .map_err(|e| format!("XML parse error: {}", e))?;

    let mut symbols: HashMap<String, SymbolDef> = HashMap::new();

    for sym in &lib.symbol {
        let id = sym.id.clone().unwrap_or_default();
        let name = sym.name.clone().unwrap_or_default();
        let type_ = sym.type_.clone().unwrap_or_default();
        let subtype = sym.subtype.clone();

        let mut gfx = SymbolGfx::default();
        for g in &sym.gfx {
            if !is_visible(g.visibility.as_ref()) {
                continue;
            }
            for pg in &g.propertiedgraphic {
                for l in &pg.line {
                    if is_visible(l.visibility.as_ref()) {
                        gfx.lines.push((
                            l.x1.unwrap_or(0.0),
                            l.y1.unwrap_or(0.0),
                            l.x2.unwrap_or(0.0),
                            l.y2.unwrap_or(0.0),
                        ));
                    }
                }
                for pl in &pg.polyline {
                    if is_visible(pl.visibility.as_ref()) {
                        let pts: Vec<(f64, f64)> = pl
                            .point
                            .iter()
                            .map(|p| (p.x.unwrap_or(0.0), p.y.unwrap_or(0.0)))
                            .collect();
                        if !pts.is_empty() {
                            gfx.polylines.push(pts);
                        }
                    }
                }
            }
        }

        if !id.is_empty() {
            symbols.insert(
                id.clone(),
                SymbolDef {
                    id: id.clone(),
                    name,
                    type_,
                    subtype,
                    gfx,
                },
            );
        }
    }

    Ok(SymbolLibrary {
        name: lib.name.unwrap_or_default(),
        scaletype: lib.scaletype,
        realmapping: lib.realmapping,
        realmappingunits: lib.realmappingunits,
        symbols,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_firefly_pin_comment_symbols() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("Firefly Pin Comment Symbols.xml");
        let f = std::fs::File::open(&path).expect("Firefly Pin Comment Symbols.xml not found");
        let lib = parse_symbol_library(std::io::BufReader::new(f)).unwrap();
        assert_eq!(lib.name, "Firefly Pin Comment Symbols");
        assert_eq!(lib.symbols.len(), 3);
        let pin = lib.symbols.values().find(|s| s.name == "Pin").unwrap();
        assert_eq!(pin.gfx.lines.len(), 1);
        assert_eq!(pin.gfx.polylines.len(), 1);
        assert_eq!(pin.gfx.polylines[0].len(), 3);
    }
}
