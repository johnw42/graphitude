//! Type definitions for Graphviz DOT format attributes.
//!
//! This module provides Rust type definitions for Graphviz attribute types that don't
//! correspond to standard Rust primitives. These types can be used for type-safe
//! parsing and manipulation of DOT format attributes.
//!
//! # Examples
//!
//! ```
//! use graphitude::dot::types::{Point, Color, Shape, ArrowType};
//!
//! // Parse a point from a string
//! let point: Point = "1.5,2.5".parse().unwrap();
//! assert_eq!(point.x, 1.5);
//! assert_eq!(point.y, 2.5);
//!
//! // Parse colors in various formats
//! let red: Color = "red".parse().unwrap();           // Named color
//! let blue: Color = "#0000ff".parse().unwrap();      // Hex RGB
//! let green: Color = "/greens5/3".parse().unwrap();  // Color scheme
//!
//! // Parse node shapes
//! let shape: Shape = "box".parse().unwrap();
//! assert_eq!(shape, Shape::Box);
//!
//! // Parse arrow types
//! let arrow: ArrowType = "diamond".parse().unwrap();
//! assert_eq!(arrow, ArrowType::Diamond);
//! ```
//!
//! Reference: <https://graphviz.org/doc/info/attrs.html>

use std::fmt;
use std::str::FromStr;

/// A 2D point with x and y coordinates (in inches or points).
///
/// Format: `"x,y"` or `"x,y!"` (the trailing `!` indicates the position is fixed)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub fixed: bool,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y, fixed: false }
    }

    pub fn with_fixed(x: f64, y: f64, fixed: bool) -> Self {
        Point { x, y, fixed }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.fixed {
            write!(f, "{},{}!", self.x, self.y)
        } else {
            write!(f, "{},{}", self.x, self.y)
        }
    }
}

impl FromStr for Point {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fixed = s.ends_with('!');
        let s = s.trim_end_matches('!');

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid point format: {}", s));
        }

        let x = parts[0]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid x coordinate: {}", e))?;
        let y = parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid y coordinate: {}", e))?;

        Ok(Point { x, y, fixed })
    }
}

/// A rectangle defined by lower-left and upper-right corners.
///
/// Format: `"llx,lly,urx,ury"`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub llx: f64, // lower-left x
    pub lly: f64, // lower-left y
    pub urx: f64, // upper-right x
    pub ury: f64, // upper-right y
}

impl Rect {
    pub fn new(llx: f64, lly: f64, urx: f64, ury: f64) -> Self {
        Rect { llx, lly, urx, ury }
    }

    pub fn width(&self) -> f64 {
        self.urx - self.llx
    }

    pub fn height(&self) -> f64 {
        self.ury - self.lly
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{}", self.llx, self.lly, self.urx, self.ury)
    }
}

impl FromStr for Rect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid rect format: {}", s));
        }

        let llx = parts[0]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid llx: {}", e))?;
        let lly = parts[1]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid lly: {}", e))?;
        let urx = parts[2]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid urx: {}", e))?;
        let ury = parts[3]
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Invalid ury: {}", e))?;

        Ok(Rect { llx, lly, urx, ury })
    }
}

/// A color specification in various formats.
///
/// Supports:
/// - X11 color names (e.g., "red", "blue")
/// - RGB: "#RGB" or "#RRGGBB"
/// - RGBA: "#RRGGBBAA"
/// - HSV: "H,S,V" (H in [0,1], S in [0,1], V in [0,1])
/// - Color scheme references: "/scheme/color"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Named(String),
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
    Hsv(String), // Store as string for simplicity
    Scheme { scheme: String, color: String },
}

impl Color {
    pub fn named(name: &str) -> Self {
        Color::Named(name.to_string())
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb(r, g, b)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color::Rgba(r, g, b, a)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Named(name) => write!(f, "{}", name),
            Color::Rgb(r, g, b) => write!(f, "#{:02x}{:02x}{:02x}", r, g, b),
            Color::Rgba(r, g, b, a) => write!(f, "#{:02x}{:02x}{:02x}{:02x}", r, g, b, a),
            Color::Hsv(hsv) => write!(f, "{}", hsv),
            Color::Scheme { scheme, color } => write!(f, "/{}/{}", scheme, color),
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Color scheme reference: "/scheme/color"
        if s.starts_with('/') {
            let parts: Vec<&str> = s.trim_start_matches('/').split('/').collect();
            if parts.len() >= 2 {
                return Ok(Color::Scheme {
                    scheme: parts[0].to_string(),
                    color: parts[1].to_string(),
                });
            }
        }

        // Hex color: #RGB, #RRGGBB, or #RRGGBBAA
        if s.starts_with('#') {
            let hex = s.trim_start_matches('#');
            match hex.len() {
                3 => {
                    // #RGB -> expand to #RRGGBB
                    let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                        .map_err(|e| format!("Invalid red component: {}", e))?;
                    let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                        .map_err(|e| format!("Invalid green component: {}", e))?;
                    let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                        .map_err(|e| format!("Invalid blue component: {}", e))?;
                    return Ok(Color::Rgb(r, g, b));
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16)
                        .map_err(|e| format!("Invalid red component: {}", e))?;
                    let g = u8::from_str_radix(&hex[2..4], 16)
                        .map_err(|e| format!("Invalid green component: {}", e))?;
                    let b = u8::from_str_radix(&hex[4..6], 16)
                        .map_err(|e| format!("Invalid blue component: {}", e))?;
                    return Ok(Color::Rgb(r, g, b));
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16)
                        .map_err(|e| format!("Invalid red component: {}", e))?;
                    let g = u8::from_str_radix(&hex[2..4], 16)
                        .map_err(|e| format!("Invalid green component: {}", e))?;
                    let b = u8::from_str_radix(&hex[4..6], 16)
                        .map_err(|e| format!("Invalid blue component: {}", e))?;
                    let a = u8::from_str_radix(&hex[6..8], 16)
                        .map_err(|e| format!("Invalid alpha component: {}", e))?;
                    return Ok(Color::Rgba(r, g, b, a));
                }
                _ => return Err(format!("Invalid hex color length: {}", s)),
            }
        }

        // HSV format: "H,S,V"
        if s.contains(',') {
            return Ok(Color::Hsv(s.to_string()));
        }

        // Default to named color
        Ok(Color::Named(s.to_string()))
    }
}

/// Arrow type for edge arrowheads and arrowtails.
///
/// Examples: "normal", "dot", "odot", "none", "empty", "diamond", "ediamond", "box", "open", "vee", "inv", "invdot", "invodot", "tee", "crow"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrowType {
    Normal,
    Dot,
    Odot,
    None,
    Empty,
    Diamond,
    Ediamond,
    Box,
    Open,
    Vee,
    Inv,
    Invdot,
    Invodot,
    Tee,
    Crow,
    Custom(String),
}

impl fmt::Display for ArrowType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowType::Normal => write!(f, "normal"),
            ArrowType::Dot => write!(f, "dot"),
            ArrowType::Odot => write!(f, "odot"),
            ArrowType::None => write!(f, "none"),
            ArrowType::Empty => write!(f, "empty"),
            ArrowType::Diamond => write!(f, "diamond"),
            ArrowType::Ediamond => write!(f, "ediamond"),
            ArrowType::Box => write!(f, "box"),
            ArrowType::Open => write!(f, "open"),
            ArrowType::Vee => write!(f, "vee"),
            ArrowType::Inv => write!(f, "inv"),
            ArrowType::Invdot => write!(f, "invdot"),
            ArrowType::Invodot => write!(f, "invodot"),
            ArrowType::Tee => write!(f, "tee"),
            ArrowType::Crow => write!(f, "crow"),
            ArrowType::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for ArrowType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(ArrowType::Normal),
            "dot" => Ok(ArrowType::Dot),
            "odot" => Ok(ArrowType::Odot),
            "none" => Ok(ArrowType::None),
            "empty" => Ok(ArrowType::Empty),
            "diamond" => Ok(ArrowType::Diamond),
            "ediamond" => Ok(ArrowType::Ediamond),
            "box" => Ok(ArrowType::Box),
            "open" => Ok(ArrowType::Open),
            "vee" => Ok(ArrowType::Vee),
            "inv" => Ok(ArrowType::Inv),
            "invdot" => Ok(ArrowType::Invdot),
            "invodot" => Ok(ArrowType::Invodot),
            "tee" => Ok(ArrowType::Tee),
            "crow" => Ok(ArrowType::Crow),
            _ => Ok(ArrowType::Custom(s.to_string())),
        }
    }
}

/// Direction type for edges.
///
/// Values: "forward", "back", "both", "none"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirType {
    Forward,
    Back,
    Both,
    None,
}

impl fmt::Display for DirType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirType::Forward => write!(f, "forward"),
            DirType::Back => write!(f, "back"),
            DirType::Both => write!(f, "both"),
            DirType::None => write!(f, "none"),
        }
    }
}

impl FromStr for DirType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "forward" => Ok(DirType::Forward),
            "back" => Ok(DirType::Back),
            "both" => Ok(DirType::Both),
            "none" => Ok(DirType::None),
            _ => Err(format!("Invalid dirType: {}", s)),
        }
    }
}

/// Rank direction for graph layout.
///
/// Values: "TB" (top to bottom), "BT" (bottom to top), "LR" (left to right), "RL" (right to left)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankDir {
    TB, // Top to Bottom
    BT, // Bottom to Top
    LR, // Left to Right
    RL, // Right to Left
}

impl fmt::Display for RankDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RankDir::TB => write!(f, "TB"),
            RankDir::BT => write!(f, "BT"),
            RankDir::LR => write!(f, "LR"),
            RankDir::RL => write!(f, "RL"),
        }
    }
}

impl FromStr for RankDir {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TB" => Ok(RankDir::TB),
            "BT" => Ok(RankDir::BT),
            "LR" => Ok(RankDir::LR),
            "RL" => Ok(RankDir::RL),
            _ => Err(format!("Invalid rankdir: {}", s)),
        }
    }
}

/// Rank type for subgraph constraints.
///
/// Values: "same", "min", "source", "max", "sink"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankType {
    Same,
    Min,
    Source,
    Max,
    Sink,
}

impl fmt::Display for RankType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RankType::Same => write!(f, "same"),
            RankType::Min => write!(f, "min"),
            RankType::Source => write!(f, "source"),
            RankType::Max => write!(f, "max"),
            RankType::Sink => write!(f, "sink"),
        }
    }
}

impl FromStr for RankType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "same" => Ok(RankType::Same),
            "min" => Ok(RankType::Min),
            "source" => Ok(RankType::Source),
            "max" => Ok(RankType::Max),
            "sink" => Ok(RankType::Sink),
            _ => Err(format!("Invalid rankType: {}", s)),
        }
    }
}

/// Node shape types.
///
/// Common shapes: box, polygon, ellipse, oval, circle, point, egg, triangle, plaintext,
/// plain, diamond, trapezium, parallelogram, house, pentagon, hexagon, septagon, octagon,
/// doublecircle, doubleoctagon, tripleoctagon, invtriangle, invtrapezium, invhouse,
/// Mdiamond, Msquare, Mcircle, rect, rectangle, square, star, none, underline, cylinder,
/// note, tab, folder, box3d, component, promoter, cds, terminator, utr, primersite,
/// restrictionsite, fivepoverhang, threepoverhang, noverhang, assembly, signature,
/// insulator, ribosite, rnastab, proteasesite, proteinstab, rpromoter, rarrow, larrow,
/// lpromoter
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Box,
    Polygon,
    Ellipse,
    Oval,
    Circle,
    Point,
    Egg,
    Triangle,
    Plaintext,
    Plain,
    Diamond,
    Trapezium,
    Parallelogram,
    House,
    Pentagon,
    Hexagon,
    Septagon,
    Octagon,
    Doublecircle,
    Rectangle,
    Square,
    Star,
    None,
    Record,
    Mrecord,
    Custom(String),
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Box => write!(f, "box"),
            Shape::Polygon => write!(f, "polygon"),
            Shape::Ellipse => write!(f, "ellipse"),
            Shape::Oval => write!(f, "oval"),
            Shape::Circle => write!(f, "circle"),
            Shape::Point => write!(f, "point"),
            Shape::Egg => write!(f, "egg"),
            Shape::Triangle => write!(f, "triangle"),
            Shape::Plaintext => write!(f, "plaintext"),
            Shape::Plain => write!(f, "plain"),
            Shape::Diamond => write!(f, "diamond"),
            Shape::Trapezium => write!(f, "trapezium"),
            Shape::Parallelogram => write!(f, "parallelogram"),
            Shape::House => write!(f, "house"),
            Shape::Pentagon => write!(f, "pentagon"),
            Shape::Hexagon => write!(f, "hexagon"),
            Shape::Septagon => write!(f, "septagon"),
            Shape::Octagon => write!(f, "octagon"),
            Shape::Doublecircle => write!(f, "doublecircle"),
            Shape::Rectangle => write!(f, "rectangle"),
            Shape::Square => write!(f, "square"),
            Shape::Star => write!(f, "star"),
            Shape::None => write!(f, "none"),
            Shape::Record => write!(f, "record"),
            Shape::Mrecord => write!(f, "Mrecord"),
            Shape::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for Shape {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "box" => Ok(Shape::Box),
            "polygon" => Ok(Shape::Polygon),
            "ellipse" => Ok(Shape::Ellipse),
            "oval" => Ok(Shape::Oval),
            "circle" => Ok(Shape::Circle),
            "point" => Ok(Shape::Point),
            "egg" => Ok(Shape::Egg),
            "triangle" => Ok(Shape::Triangle),
            "plaintext" => Ok(Shape::Plaintext),
            "plain" => Ok(Shape::Plain),
            "diamond" => Ok(Shape::Diamond),
            "trapezium" => Ok(Shape::Trapezium),
            "parallelogram" => Ok(Shape::Parallelogram),
            "house" => Ok(Shape::House),
            "pentagon" => Ok(Shape::Pentagon),
            "hexagon" => Ok(Shape::Hexagon),
            "septagon" => Ok(Shape::Septagon),
            "octagon" => Ok(Shape::Octagon),
            "doublecircle" => Ok(Shape::Doublecircle),
            "rectangle" | "rect" => Ok(Shape::Rectangle),
            "square" => Ok(Shape::Square),
            "star" => Ok(Shape::Star),
            "none" => Ok(Shape::None),
            "record" => Ok(Shape::Record),
            "mrecord" => Ok(Shape::Mrecord),
            _ => Ok(Shape::Custom(s.to_string())),
        }
    }
}

/// Style attribute values.
///
/// Common styles: filled, invisible, diagonals, rounded, dashed, dotted, solid, bold
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style {
    pub styles: Vec<String>,
}

impl Style {
    pub fn new() -> Self {
        Style { styles: Vec::new() }
    }

    pub fn with_style(style: &str) -> Self {
        Style {
            styles: vec![style.to_string()],
        }
    }

    pub fn add_style(&mut self, style: &str) {
        self.styles.push(style.to_string());
    }

    pub fn has_style(&self, style: &str) -> bool {
        self.styles.iter().any(|s| s.eq_ignore_ascii_case(style))
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.styles.join(","))
    }
}

impl FromStr for Style {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let styles = s
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(Style { styles })
    }
}

/// Output order mode.
///
/// Values: "breadthfirst", "nodesfirst", "edgesfirst"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Breadthfirst,
    Nodesfirst,
    Edgesfirst,
}

impl fmt::Display for OutputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputMode::Breadthfirst => write!(f, "breadthfirst"),
            OutputMode::Nodesfirst => write!(f, "nodesfirst"),
            OutputMode::Edgesfirst => write!(f, "edgesfirst"),
        }
    }
}

impl FromStr for OutputMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "breadthfirst" => Ok(OutputMode::Breadthfirst),
            "nodesfirst" => Ok(OutputMode::Nodesfirst),
            "edgesfirst" => Ok(OutputMode::Edgesfirst),
            _ => Err(format!("Invalid outputMode: {}", s)),
        }
    }
}

/// Page direction for multi-page output.
///
/// Values: BL, BR, TL, TR, RB, RT, LB, LT (two-letter codes for ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageDir {
    BL, // Bottom-Left
    BR, // Bottom-Right
    TL, // Top-Left
    TR, // Top-Right
    RB, // Right-Bottom
    RT, // Right-Top
    LB, // Left-Bottom
    LT, // Left-Top
}

impl fmt::Display for PageDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageDir::BL => write!(f, "BL"),
            PageDir::BR => write!(f, "BR"),
            PageDir::TL => write!(f, "TL"),
            PageDir::TR => write!(f, "TR"),
            PageDir::RB => write!(f, "RB"),
            PageDir::RT => write!(f, "RT"),
            PageDir::LB => write!(f, "LB"),
            PageDir::LT => write!(f, "LT"),
        }
    }
}

impl FromStr for PageDir {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BL" => Ok(PageDir::BL),
            "BR" => Ok(PageDir::BR),
            "TL" => Ok(PageDir::TL),
            "TR" => Ok(PageDir::TR),
            "RB" => Ok(PageDir::RB),
            "RT" => Ok(PageDir::RT),
            "LB" => Ok(PageDir::LB),
            "LT" => Ok(PageDir::LT),
            _ => Err(format!("Invalid pagedir: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_parsing() {
        let p: Point = "1.5,2.5".parse().unwrap();
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, 2.5);
        assert!(!p.fixed);

        let p: Point = "1.5,2.5!".parse().unwrap();
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, 2.5);
        assert!(p.fixed);
    }

    #[test]
    fn test_rect_parsing() {
        let r: Rect = "0,0,100,50".parse().unwrap();
        assert_eq!(r.llx, 0.0);
        assert_eq!(r.lly, 0.0);
        assert_eq!(r.urx, 100.0);
        assert_eq!(r.ury, 50.0);
        assert_eq!(r.width(), 100.0);
        assert_eq!(r.height(), 50.0);
    }

    #[test]
    fn test_color_parsing() {
        // Named color
        let c: Color = "red".parse().unwrap();
        assert_eq!(c, Color::Named("red".to_string()));

        // RGB hex
        let c: Color = "#ff0000".parse().unwrap();
        assert_eq!(c, Color::Rgb(255, 0, 0));

        // RGBA hex
        let c: Color = "#ff0000ff".parse().unwrap();
        assert_eq!(c, Color::Rgba(255, 0, 0, 255));

        // Short hex
        let c: Color = "#f00".parse().unwrap();
        assert_eq!(c, Color::Rgb(255, 0, 0));

        // Color scheme
        let c: Color = "/blues5/3".parse().unwrap();
        match c {
            Color::Scheme { scheme, color } => {
                assert_eq!(scheme, "blues5");
                assert_eq!(color, "3");
            }
            _ => panic!("Expected Color::Scheme"),
        }
    }

    #[test]
    fn test_arrow_type_parsing() {
        let a: ArrowType = "normal".parse().unwrap();
        assert_eq!(a, ArrowType::Normal);

        let a: ArrowType = "diamond".parse().unwrap();
        assert_eq!(a, ArrowType::Diamond);

        let a: ArrowType = "none".parse().unwrap();
        assert_eq!(a, ArrowType::None);
    }

    #[test]
    fn test_dir_type_parsing() {
        let d: DirType = "forward".parse().unwrap();
        assert_eq!(d, DirType::Forward);

        let d: DirType = "both".parse().unwrap();
        assert_eq!(d, DirType::Both);
    }

    #[test]
    fn test_rankdir_parsing() {
        let r: RankDir = "TB".parse().unwrap();
        assert_eq!(r, RankDir::TB);

        let r: RankDir = "LR".parse().unwrap();
        assert_eq!(r, RankDir::LR);
    }

    #[test]
    fn test_shape_parsing() {
        let s: Shape = "box".parse().unwrap();
        assert_eq!(s, Shape::Box);

        let s: Shape = "ellipse".parse().unwrap();
        assert_eq!(s, Shape::Ellipse);

        let s: Shape = "record".parse().unwrap();
        assert_eq!(s, Shape::Record);
    }

    #[test]
    fn test_style_parsing() {
        let s: Style = "filled".parse().unwrap();
        assert!(s.has_style("filled"));

        let s: Style = "filled,rounded".parse().unwrap();
        assert!(s.has_style("filled"));
        assert!(s.has_style("rounded"));
    }
}
