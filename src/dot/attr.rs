//! Enumeration of Graphviz DOT format attributes with typed payloads.
//!
//! This module provides a comprehensive enum of all Graphviz attributes as documented at
//! <https://graphviz.org/doc/info/attrs.html>. Each variant includes typed payload data
//! corresponding to the attribute's type.
//!
//! # Examples
//!
//! ```
//! use graphitude::dot::attr::Attr;
//!
//! let attr = Attr::parse("color", "red").unwrap();
//! assert_eq!(attr.to_string(), "color=red");
//! ```

use std::fmt;

use super::types::{
    ArrowType, Color, DirType, OutputMode, PageDir, Point, RankDir, RankType, Rect, Shape, Style,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError<'a> {
    UnknownAttribute(&'a str),
    InvalidValue(&'a str, &'a str),
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownAttribute(name) => write!(f, "Unknown attribute: {}", name),
            ParseError::InvalidValue(attr_type, value) => {
                write!(f, "Invalid {} value: {}", attr_type, value)
            }
        }
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

/// Enumeration of all Graphviz DOT attribute names with typed payloads.
///
/// Each variant corresponds to an attribute documented at
/// <https://graphviz.org/doc/info/attrs.html> and contains the attribute's value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Attr {
    /// Background drawing for a graph (xdot format)
    _background(String),
    /// Hyperlink URL or pathname
    URL(String),
    /// Preferred area for a node or empty cluster
    Area(f64),
    /// Style of arrowhead on the head node of an edge
    Arrowhead(ArrowType),
    /// Multiplicative scale factor for arrowheads
    Arrowsize(f64),
    /// Style of arrowhead on the tail node of an edge
    Arrowtail(ArrowType),
    /// Bounding box of drawing in points
    Bb(Rect),
    /// Whether to draw leaf nodes uniformly in a circle
    Beautify(bool),
    /// Canvas background color (single color or gradient list)
    Bgcolor(Vec<Color>),
    /// Whether to center the drawing
    Center(bool),
    /// Character encoding
    Charset(String),
    /// CSS classnames for SVG output
    Class(String),
    /// Whether the subgraph is a cluster
    Cluster(bool),
    /// Mode for handling clusters
    Clusterrank(String),
    /// Basic drawing color (single color or list for parallel splines)
    Color(Vec<Color>),
    /// Color scheme namespace
    Colorscheme(String),
    /// Comments inserted into output
    Comment(String),
    /// Allow edges between clusters
    Compound(bool),
    /// Use edge concentrators
    Concentrate(bool),
    /// Whether edge is used in ranking nodes
    Constraint(bool),
    /// Factor damping force motions
    Damping(f64),
    /// Connect edge label to edge with a line
    Decorate(bool),
    /// Distance between nodes in separate components
    Defaultdist(f64),
    /// Number of dimensions for layout
    Dim(i32),
    /// Number of dimensions for rendering
    Dimen(i32),
    /// Edge type for drawing arrowheads
    Dir(DirType),
    /// Constrain edges to point downwards (string variant)
    DiredgeconstraintsString(String),
    /// Constrain edges to point downwards (bool variant)
    DiredgeconstraintsBool(bool),
    /// Distortion factor for polygon shapes
    Distortion(f64),
    /// Pixels per inch on display device
    Dpi(f64),
    /// Synonym for edgeURL
    Edgehref(String),
    /// Browser window for edgeURL link
    Edgetarget(String),
    /// Tooltip for non-label part of edge
    Edgetooltip(String),
    /// Link for non-label parts of edge
    EdgeURL(String),
    /// Terminating condition for layout
    Epsilon(f64),
    /// Margin around polygons for spline routing (double variant)
    EsepDouble(f64),
    /// Margin around polygons for spline routing (point variant)
    EsepPoint(Point),
    /// Color to fill node or cluster background (single color or gradient list)
    Fillcolor(Vec<Color>),
    /// Whether to use specified width/height (bool variant)
    FixedsizeBool(bool),
    /// Whether to use specified width/height (string variant)
    FixedsizeString(String),
    /// Color used for text
    Fontcolor(Color),
    /// Font used for text
    Fontname(String),
    /// Font name representation in SVG
    Fontnames(String),
    /// Directory list for bitmap fonts
    Fontpath(String),
    /// Font size in points
    Fontsize(f64),
    /// Force placement of all xlabels
    Forcelabels(bool),
    /// Gradient angle for fill
    Gradientangle(i32),
    /// Name for a group of nodes
    Group(String),
    /// Center position of edge's head label
    Head_lp(Point),
    /// Clip head of edge to boundary
    Headclip(bool),
    /// Synonym for headURL
    Headhref(String),
    /// Text label near head of edge
    Headlabel(String),
    /// Where to attach head of edge
    Headport(String),
    /// Browser window for headURL link
    Headtarget(String),
    /// Tooltip for head of edge
    Headtooltip(String),
    /// Link for head label of edge
    HeadURL(String),
    /// Height of node in inches
    Height(f64),
    /// Synonym for URL
    Href(String),
    /// Identifier for graph objects
    Id(String),
    /// Image file to display inside node
    Image(String),
    /// Directories to look for image files
    Imagepath(String),
    /// How image is positioned in node
    Imagepos(String),
    /// How image fills containing node (bool variant)
    ImagescaleBool(bool),
    /// How image fills containing node (string variant)
    ImagescaleString(String),
    /// Scale input positions between units
    Inputscale(f64),
    /// Spring constant for virtual model
    K(f64),
    /// Text label attached to objects
    Label(String),
    /// Edge label node handling
    Label_scheme(i32),
    /// Angle for head/tail edge labels
    Labelangle(f64),
    /// Distance of head/tail labels
    Labeldistance(f64),
    /// Allow less constrained edge labels
    Labelfloat(bool),
    /// Color for headlabel and taillabel
    Labelfontcolor(Color),
    /// Font for headlabel and taillabel
    Labelfontname(String),
    /// Font size for headlabel and taillabel
    Labelfontsize(f64),
    /// Synonym for labelURL
    Labelhref(String),
    /// Justification for graph/cluster labels
    Labeljust(String),
    /// Vertical placement of labels
    Labelloc(String),
    /// Browser window for labelURL link
    Labeltarget(String),
    /// Tooltip for edge label
    Labeltooltip(String),
    /// Link for label of edge
    LabelURL(String),
    /// Render graph in landscape mode
    Landscape(bool),
    /// Layers in which component is present
    Layer(String),
    /// Separator for layerRange splitting
    Layerlistsep(String),
    /// List of layer names
    Layers(String),
    /// List of layers to emit
    Layerselect(String),
    /// Separator for layers attribute
    Layersep(String),
    /// Which layout engine to use
    Layout(String),
    /// Preferred edge length in inches
    Len(f64),
    /// Levels in multilevel scheme
    Levels(i32),
    /// Strictness of neato level constraints
    Levelsgap(f64),
    /// Logical head of edge
    Lhead(String),
    /// Height of graph/cluster label
    Lheight(f64),
    /// Line length for text output
    Linelength(i32),
    /// Label center position
    Lp(Point),
    /// Logical tail of edge
    Ltail(String),
    /// Width of graph/cluster label
    Lwidth(f64),
    /// Margins of canvas or around label (double variant)
    MarginDouble(f64),
    /// Margins of canvas or around label (point variant)
    MarginPoint(Point),
    /// Maximum iterations for layout
    Maxiter(i32),
    /// Upper bound on crossing minimization
    Mclimit(f64),
    /// Minimum separation between nodes
    Mindist(f64),
    /// Minimum edge length (rank difference)
    Minlen(i32),
    /// Technique for optimizing layout
    Mode(String),
    /// How distance matrix is computed
    Model(String),
    /// Use single global ranking
    Newrank(bool),
    /// Minimum space between adjacent nodes
    Nodesep(f64),
    /// Justify multiline text
    Nojustify(bool),
    /// Normalize coordinates of final layout (double variant)
    NormalizeDouble(f64),
    /// Normalize coordinates of final layout (bool variant)
    NormalizeBool(bool),
    /// Avoid translating layout to origin
    Notranslate(bool),
    /// Iterations in network simplex (x coords)
    Nslimit(f64),
    /// Iterations in network simplex (ranking)
    Nslimit1(f64),
    /// Draw circo graphs around one circle
    Oneblock(bool),
    /// Left-to-right ordering of node edges
    Ordering(String),
    /// Node shape rotation or graph orientation (double variant)
    OrientationDouble(f64),
    /// Node shape rotation or graph orientation (string variant)
    OrientationString(String),
    /// Order in which nodes and edges are drawn
    Outputorder(OutputMode),
    /// How to remove node overlaps (string variant)
    OverlapString(String),
    /// How to remove node overlaps (bool variant)
    OverlapBool(bool),
    /// Scale layout to reduce node overlap
    Overlap_scaling(f64),
    /// Compression pass for overlap removal
    Overlap_shrink(bool),
    /// Pack connected components separately (bool variant)
    PackBool(bool),
    /// Pack connected components separately (int variant)
    PackInt(i32),
    /// How to pack connected components
    Packmode(String),
    /// Extend drawing area around graph (double variant)
    PadDouble(f64),
    /// Extend drawing area around graph (point variant)
    PadPoint(Point),
    /// Width and height of output pages (double variant)
    PageDouble(f64),
    /// Width and height of output pages (point variant)
    PagePoint(Point),
    /// Order pages are emitted
    Pagedir(PageDir),
    /// Color for cluster bounding box
    Pencolor(Color),
    /// Width of pen for lines and curves
    Penwidth(f64),
    /// Number of peripheries for polygons
    Peripheries(i32),
    /// Keep node at given position
    Pin(bool),
    /// Position of node or spline control points (point variant)
    PosPoint(Point),
    /// Position of node or spline control points (splineType variant)
    PosString(String),
    /// Quadtree scheme to use (string variant)
    QuadtreeString(String),
    /// Quadtree scheme to use (bool variant)
    QuadtreeBool(bool),
    /// Quantum for node label dimensions
    Quantum(f64),
    /// Radius of rounded corners
    Radius(f64),
    /// Rank constraints on nodes in subgraph
    Rank(RankType),
    /// Direction of graph layout
    Rankdir(RankDir),
    /// Separation between ranks (single value or list)
    Ranksep(Vec<f64>),
    /// Aspect ratio for drawing (double variant)
    RatioDouble(f64),
    /// Aspect ratio for drawing (string variant)
    RatioString(String),
    /// Rectangles for record fields
    Rects(Rect),
    /// Force polygon to be regular
    Regular(bool),
    /// Run edge crossing minimization twice
    Remincross(bool),
    /// Power of repulsive force
    Repulsiveforce(f64),
    /// Synonym for dpi
    Resolution(f64),
    /// Nodes used as center of layout (string variant)
    RootString(String),
    /// Nodes used as center of layout (bool variant)
    RootBool(bool),
    /// Set drawing orientation to landscape
    Rotate(i32),
    /// Rotate final layout counter-clockwise
    Rotation(f64),
    /// Edges with same head point to same place
    Samehead(String),
    /// Edges with same tail point to same place
    Sametail(String),
    /// Number of points for circle/ellipse node
    Samplepoints(i32),
    /// Scale layout after initial layout (double variant)
    ScaleDouble(f64),
    /// Scale layout after initial layout (point variant)
    ScalePoint(Point),
    /// Maximum edges with negative cut values
    Searchsize(i32),
    /// Margin around nodes for overlap removal (double variant)
    SepDouble(f64),
    /// Margin around nodes for overlap removal (point variant)
    SepPoint(Point),
    /// Shape of a node
    Shape(Shape),
    /// File containing user-supplied node content
    Shapefile(String),
    /// Print guide boxes for debugging
    Showboxes(i32),
    /// Number of sides for polygon shape
    Sides(i32),
    /// Maximum width and height of drawing (double variant)
    SizeDouble(f64),
    /// Maximum width and height of drawing (point variant)
    SizePoint(Point),
    /// Skew factor for polygon shape
    Skew(f64),
    /// Smooth out uneven node distribution
    Smoothing(String),
    /// Sort order for packmode packing
    Sortv(i32),
    /// How edges are represented (bool variant)
    SplinesBool(bool),
    /// How edges are represented (string variant)
    SplinesString(String),
    /// Initial layout parameter
    Start(String),
    /// Style information for components
    Style(Style),
    /// XML stylesheet for SVG output
    Stylesheet(String),
    /// Position of edge's tail label
    Tail_lp(Point),
    /// Clip tail of edge to boundary
    Tailclip(bool),
    /// Synonym for tailURL
    Tailhref(String),
    /// Text label near tail of edge
    Taillabel(String),
    /// Where to attach tail of edge
    Tailport(String),
    /// Browser window for tailURL link
    Tailtarget(String),
    /// Tooltip for tail of edge
    Tailtooltip(String),
    /// Link for tail label of edge
    TailURL(String),
    /// Browser window for URL link
    Target(String),
    /// Which rank to move floating nodes to
    TBbalance(String),
    /// Tooltip annotation
    Tooltip(String),
    /// Use truecolor rendering
    Truecolor(bool),
    /// Coordinates of polygon vertices
    Vertices(Vec<Point>),
    /// Clipping window on final drawing
    Viewport(String),
    /// Tuning margin of Voronoi technique
    Voro_margin(f64),
    /// Weight of edge (int variant)
    WeightInt(i32),
    /// Weight of edge (double variant)
    WeightDouble(f64),
    /// Width of node in inches
    Width(f64),
    /// Version of xdot used in output
    Xdotversion(String),
    /// External label for node or edge
    Xlabel(String),
    /// Position of exterior label
    Xlp(Point),
    /// Z-coordinate for 3D layouts
    Z(f64),
}

impl Attr {
    /// Parse an attribute from its name and string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use graphitude::dot::attr::Attr;
    ///
    /// let attr = Attr::parse("fontsize", "12").unwrap();
    /// let attr = Attr::parse("color", "red").unwrap();
    /// let attr = Attr::parse("shape", "box").unwrap();
    /// ```
    pub fn parse<'a>(name: &'a str, value: &'a str) -> Result<Self, ParseError<'a>> {
        match name.to_lowercase().as_str() {
            "_background" => Ok(Attr::_background(value.to_string())),
            "url" => Ok(Attr::URL(value.to_string())),
            "area" => value
                .parse::<f64>()
                .map(Attr::Area)
                .map_err(|_| ParseError::InvalidValue("area value", value)),
            "arrowhead" => value
                .parse::<ArrowType>()
                .map(Attr::Arrowhead)
                .map_err(|_| ParseError::InvalidValue("arrowhead", value)),
            "arrowsize" => value
                .parse::<f64>()
                .map(Attr::Arrowsize)
                .map_err(|_| ParseError::InvalidValue("arrowsize", value)),
            "arrowtail" => value
                .parse::<ArrowType>()
                .map(Attr::Arrowtail)
                .map_err(|_| ParseError::InvalidValue("arrowtail", value)),
            "bb" => value
                .parse::<Rect>()
                .map(Attr::Bb)
                .map_err(|_| ParseError::InvalidValue("bb", value)),
            "beautify" => parse_bool(value).map(Attr::Beautify),
            "bgcolor" => {
                // Try parsing as colon-separated list of colors
                let colors: Result<Vec<Color>, _> =
                    value.split(':').map(|s| s.parse::<Color>()).collect();
                colors
                    .map(Attr::Bgcolor)
                    .map_err(|_| ParseError::InvalidValue("bgcolor", value))
            }
            "center" => parse_bool(value).map(Attr::Center),
            "charset" => Ok(Attr::Charset(value.to_string())),
            "class" => Ok(Attr::Class(value.to_string())),
            "cluster" => parse_bool(value).map(Attr::Cluster),
            "clusterrank" => Ok(Attr::Clusterrank(value.to_string())),
            "color" => {
                // Try parsing as colon-separated list of colors
                let colors: Result<Vec<Color>, _> =
                    value.split(':').map(|s| s.parse::<Color>()).collect();
                colors
                    .map(Attr::Color)
                    .map_err(|_| ParseError::InvalidValue("color", value))
            }
            "colorscheme" => Ok(Attr::Colorscheme(value.to_string())),
            "comment" => Ok(Attr::Comment(value.to_string())),
            "compound" => parse_bool(value).map(Attr::Compound),
            "concentrate" => parse_bool(value).map(Attr::Concentrate),
            "constraint" => parse_bool(value).map(Attr::Constraint),
            "damping" => value
                .parse::<f64>()
                .map(Attr::Damping)
                .map_err(|_| ParseError::InvalidValue("damping", value)),
            "decorate" => parse_bool(value).map(Attr::Decorate),
            "defaultdist" => value
                .parse::<f64>()
                .map(Attr::Defaultdist)
                .map_err(|_| ParseError::InvalidValue("defaultdist", value)),
            "dim" => value
                .parse::<i32>()
                .map(Attr::Dim)
                .map_err(|_| ParseError::InvalidValue("dim", value)),
            "dimen" => value
                .parse::<i32>()
                .map(Attr::Dimen)
                .map_err(|_| ParseError::InvalidValue("dimen", value)),
            "dir" => value
                .parse::<DirType>()
                .map(Attr::Dir)
                .map_err(|_| ParseError::InvalidValue("dir", value)),
            "diredgeconstraints" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::DiredgeconstraintsBool(b))
                } else {
                    Ok(Attr::DiredgeconstraintsString(value.to_string()))
                }
            }
            "distortion" => value
                .parse::<f64>()
                .map(Attr::Distortion)
                .map_err(|_| ParseError::InvalidValue("distortion", value)),
            "dpi" => value
                .parse::<f64>()
                .map(Attr::Dpi)
                .map_err(|_| ParseError::InvalidValue("dpi", value)),
            "edgehref" => Ok(Attr::Edgehref(value.to_string())),
            "edgetarget" => Ok(Attr::Edgetarget(value.to_string())),
            "edgetooltip" => Ok(Attr::Edgetooltip(value.to_string())),
            "edgeurl" => Ok(Attr::EdgeURL(value.to_string())),
            "epsilon" => value
                .parse::<f64>()
                .map(Attr::Epsilon)
                .map_err(|_| ParseError::InvalidValue("epsilon", value)),
            "esep" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::EsepPoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::EsepDouble)
                        .map_err(|_| ParseError::InvalidValue("esep", value))
                }
            }
            "fillcolor" => {
                // Try parsing as colon-separated list of colors
                let colors: Result<Vec<Color>, _> =
                    value.split(':').map(|s| s.parse::<Color>()).collect();
                colors
                    .map(Attr::Fillcolor)
                    .map_err(|_| ParseError::InvalidValue("fillcolor", value))
            }
            "fixedsize" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::FixedsizeBool(b))
                } else {
                    Ok(Attr::FixedsizeString(value.to_string()))
                }
            }
            "fontcolor" => value
                .parse::<Color>()
                .map(Attr::Fontcolor)
                .map_err(|_| ParseError::InvalidValue("fontcolor", value)),
            "fontname" => Ok(Attr::Fontname(value.to_string())),
            "fontnames" => Ok(Attr::Fontnames(value.to_string())),
            "fontpath" => Ok(Attr::Fontpath(value.to_string())),
            "fontsize" => value
                .parse::<f64>()
                .map(Attr::Fontsize)
                .map_err(|_| ParseError::InvalidValue("fontsize", value)),
            "forcelabels" => parse_bool(value).map(Attr::Forcelabels),
            "gradientangle" => value
                .parse::<i32>()
                .map(Attr::Gradientangle)
                .map_err(|_| ParseError::InvalidValue("gradientangle", value)),
            "group" => Ok(Attr::Group(value.to_string())),
            "head_lp" => value
                .parse::<Point>()
                .map(Attr::Head_lp)
                .map_err(|_| ParseError::InvalidValue("head_lp", value)),
            "headclip" => parse_bool(value).map(Attr::Headclip),
            "headhref" => Ok(Attr::Headhref(value.to_string())),
            "headlabel" => Ok(Attr::Headlabel(value.to_string())),
            "headport" => Ok(Attr::Headport(value.to_string())),
            "headtarget" => Ok(Attr::Headtarget(value.to_string())),
            "headtooltip" => Ok(Attr::Headtooltip(value.to_string())),
            "headurl" => Ok(Attr::HeadURL(value.to_string())),
            "height" => value
                .parse::<f64>()
                .map(Attr::Height)
                .map_err(|_| ParseError::InvalidValue("height", value)),
            "href" => Ok(Attr::Href(value.to_string())),
            "id" => Ok(Attr::Id(value.to_string())),
            "image" => Ok(Attr::Image(value.to_string())),
            "imagepath" => Ok(Attr::Imagepath(value.to_string())),
            "imagepos" => Ok(Attr::Imagepos(value.to_string())),
            "imagescale" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::ImagescaleBool(b))
                } else {
                    Ok(Attr::ImagescaleString(value.to_string()))
                }
            }
            "inputscale" => value
                .parse::<f64>()
                .map(Attr::Inputscale)
                .map_err(|_| ParseError::InvalidValue("inputscale", value)),
            "k" => value
                .parse::<f64>()
                .map(Attr::K)
                .map_err(|_| ParseError::InvalidValue("k", value)),
            "label" => Ok(Attr::Label(value.to_string())),
            "label_scheme" => value
                .parse::<i32>()
                .map(Attr::Label_scheme)
                .map_err(|_| ParseError::InvalidValue("label_scheme", value)),
            "labelangle" => value
                .parse::<f64>()
                .map(Attr::Labelangle)
                .map_err(|_| ParseError::InvalidValue("labelangle", value)),
            "labeldistance" => value
                .parse::<f64>()
                .map(Attr::Labeldistance)
                .map_err(|_| ParseError::InvalidValue("labeldistance", value)),
            "labelfloat" => parse_bool(value).map(Attr::Labelfloat),
            "labelfontcolor" => value
                .parse::<Color>()
                .map(Attr::Labelfontcolor)
                .map_err(|_| ParseError::InvalidValue("labelfontcolor", value)),
            "labelfontname" => Ok(Attr::Labelfontname(value.to_string())),
            "labelfontsize" => value
                .parse::<f64>()
                .map(Attr::Labelfontsize)
                .map_err(|_| ParseError::InvalidValue("labelfontsize", value)),
            "labelhref" => Ok(Attr::Labelhref(value.to_string())),
            "labeljust" => Ok(Attr::Labeljust(value.to_string())),
            "labelloc" => Ok(Attr::Labelloc(value.to_string())),
            "labeltarget" => Ok(Attr::Labeltarget(value.to_string())),
            "labeltooltip" => Ok(Attr::Labeltooltip(value.to_string())),
            "labelurl" => Ok(Attr::LabelURL(value.to_string())),
            "landscape" => parse_bool(value).map(Attr::Landscape),
            "layer" => Ok(Attr::Layer(value.to_string())),
            "layerlistsep" => Ok(Attr::Layerlistsep(value.to_string())),
            "layers" => Ok(Attr::Layers(value.to_string())),
            "layerselect" => Ok(Attr::Layerselect(value.to_string())),
            "layersep" => Ok(Attr::Layersep(value.to_string())),
            "layout" => Ok(Attr::Layout(value.to_string())),
            "len" => value
                .parse::<f64>()
                .map(Attr::Len)
                .map_err(|_| ParseError::InvalidValue("len", value)),
            "levels" => value
                .parse::<i32>()
                .map(Attr::Levels)
                .map_err(|_| ParseError::InvalidValue("levels", value)),
            "levelsgap" => value
                .parse::<f64>()
                .map(Attr::Levelsgap)
                .map_err(|_| ParseError::InvalidValue("levelsgap", value)),
            "lhead" => Ok(Attr::Lhead(value.to_string())),
            "lheight" => value
                .parse::<f64>()
                .map(Attr::Lheight)
                .map_err(|_| ParseError::InvalidValue("lheight", value)),
            "linelength" => value
                .parse::<i32>()
                .map(Attr::Linelength)
                .map_err(|_| ParseError::InvalidValue("linelength", value)),
            "lp" => value
                .parse::<Point>()
                .map(Attr::Lp)
                .map_err(|_| ParseError::InvalidValue("lp", value)),
            "ltail" => Ok(Attr::Ltail(value.to_string())),
            "lwidth" => value
                .parse::<f64>()
                .map(Attr::Lwidth)
                .map_err(|_| ParseError::InvalidValue("lwidth", value)),
            "margin" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::MarginPoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::MarginDouble)
                        .map_err(|_| ParseError::InvalidValue("margin", value))
                }
            }
            "maxiter" => value
                .parse::<i32>()
                .map(Attr::Maxiter)
                .map_err(|_| ParseError::InvalidValue("maxiter", value)),
            "mclimit" => value
                .parse::<f64>()
                .map(Attr::Mclimit)
                .map_err(|_| ParseError::InvalidValue("mclimit", value)),
            "mindist" => value
                .parse::<f64>()
                .map(Attr::Mindist)
                .map_err(|_| ParseError::InvalidValue("mindist", value)),
            "minlen" => value
                .parse::<i32>()
                .map(Attr::Minlen)
                .map_err(|_| ParseError::InvalidValue("minlen", value)),
            "mode" => Ok(Attr::Mode(value.to_string())),
            "model" => Ok(Attr::Model(value.to_string())),
            "newrank" => parse_bool(value).map(Attr::Newrank),
            "nodesep" => value
                .parse::<f64>()
                .map(Attr::Nodesep)
                .map_err(|_| ParseError::InvalidValue("nodesep", value)),
            "nojustify" => parse_bool(value).map(Attr::Nojustify),
            "normalize" => {
                if let Ok(d) = value.parse::<f64>() {
                    Ok(Attr::NormalizeDouble(d))
                } else if let Ok(b) = parse_bool(value) {
                    Ok(Attr::NormalizeBool(b))
                } else {
                    Err(ParseError::InvalidValue("normalize", value))
                }
            }
            "notranslate" => parse_bool(value).map(Attr::Notranslate),
            "nslimit" => value
                .parse::<f64>()
                .map(Attr::Nslimit)
                .map_err(|_| ParseError::InvalidValue("nslimit", value)),
            "nslimit1" => value
                .parse::<f64>()
                .map(Attr::Nslimit1)
                .map_err(|_| ParseError::InvalidValue("nslimit1", value)),
            "oneblock" => parse_bool(value).map(Attr::Oneblock),
            "ordering" => Ok(Attr::Ordering(value.to_string())),
            "orientation" => {
                if let Ok(d) = value.parse::<f64>() {
                    Ok(Attr::OrientationDouble(d))
                } else {
                    Ok(Attr::OrientationString(value.to_string()))
                }
            }
            "outputorder" => value
                .parse::<OutputMode>()
                .map(Attr::Outputorder)
                .map_err(|_| ParseError::InvalidValue("outputorder", value)),
            "overlap" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::OverlapBool(b))
                } else {
                    Ok(Attr::OverlapString(value.to_string()))
                }
            }
            "overlap_scaling" => value
                .parse::<f64>()
                .map(Attr::Overlap_scaling)
                .map_err(|_| ParseError::InvalidValue("overlap_scaling", value)),
            "overlap_shrink" => parse_bool(value).map(Attr::Overlap_shrink),
            "pack" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::PackBool(b))
                } else {
                    value
                        .parse::<i32>()
                        .map(Attr::PackInt)
                        .map_err(|_| ParseError::InvalidValue("pack", value))
                }
            }
            "packmode" => Ok(Attr::Packmode(value.to_string())),
            "pad" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::PadPoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::PadDouble)
                        .map_err(|_| ParseError::InvalidValue("pad", value))
                }
            }
            "page" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::PagePoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::PageDouble)
                        .map_err(|_| ParseError::InvalidValue("page", value))
                }
            }
            "pagedir" => value
                .parse::<PageDir>()
                .map(Attr::Pagedir)
                .map_err(|_| ParseError::InvalidValue("pagedir", value)),
            "pencolor" => value
                .parse::<Color>()
                .map(Attr::Pencolor)
                .map_err(|_| ParseError::InvalidValue("pencolor", value)),
            "penwidth" => value
                .parse::<f64>()
                .map(Attr::Penwidth)
                .map_err(|_| ParseError::InvalidValue("penwidth", value)),
            "peripheries" => value
                .parse::<i32>()
                .map(Attr::Peripheries)
                .map_err(|_| ParseError::InvalidValue("peripheries", value)),
            "pin" => parse_bool(value).map(Attr::Pin),
            "pos" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::PosPoint(pt))
                } else {
                    Ok(Attr::PosString(value.to_string()))
                }
            }
            "quadtree" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::QuadtreeBool(b))
                } else {
                    Ok(Attr::QuadtreeString(value.to_string()))
                }
            }
            "quantum" => value
                .parse::<f64>()
                .map(Attr::Quantum)
                .map_err(|_| ParseError::InvalidValue("quantum", value)),
            "radius" => value
                .parse::<f64>()
                .map(Attr::Radius)
                .map_err(|_| ParseError::InvalidValue("radius", value)),
            "rank" => value
                .parse::<RankType>()
                .map(Attr::Rank)
                .map_err(|_| ParseError::InvalidValue("rank", value)),
            "rankdir" => value
                .parse::<RankDir>()
                .map(Attr::Rankdir)
                .map_err(|_| ParseError::InvalidValue("rankdir", value)),
            "ranksep" => {
                // Try parsing as colon-separated list of doubles
                let values: Result<Vec<f64>, _> =
                    value.split(':').map(|s| s.parse::<f64>()).collect();
                values
                    .map(Attr::Ranksep)
                    .map_err(|_| ParseError::InvalidValue("ranksep", value))
            }
            "ratio" => {
                if let Ok(d) = value.parse::<f64>() {
                    Ok(Attr::RatioDouble(d))
                } else {
                    Ok(Attr::RatioString(value.to_string()))
                }
            }
            "rects" => value
                .parse::<Rect>()
                .map(Attr::Rects)
                .map_err(|_| ParseError::InvalidValue("rects", value)),
            "regular" => parse_bool(value).map(Attr::Regular),
            "remincross" => parse_bool(value).map(Attr::Remincross),
            "repulsiveforce" => value
                .parse::<f64>()
                .map(Attr::Repulsiveforce)
                .map_err(|_| ParseError::InvalidValue("repulsiveforce", value)),
            "resolution" => value
                .parse::<f64>()
                .map(Attr::Resolution)
                .map_err(|_| ParseError::InvalidValue("resolution", value)),
            "root" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::RootBool(b))
                } else {
                    Ok(Attr::RootString(value.to_string()))
                }
            }
            "rotate" => value
                .parse::<i32>()
                .map(Attr::Rotate)
                .map_err(|_| ParseError::InvalidValue("rotate", value)),
            "rotation" => value
                .parse::<f64>()
                .map(Attr::Rotation)
                .map_err(|_| ParseError::InvalidValue("rotation", value)),
            "samehead" => Ok(Attr::Samehead(value.to_string())),
            "sametail" => Ok(Attr::Sametail(value.to_string())),
            "samplepoints" => value
                .parse::<i32>()
                .map(Attr::Samplepoints)
                .map_err(|_| ParseError::InvalidValue("samplepoints", value)),
            "scale" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::ScalePoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::ScaleDouble)
                        .map_err(|_| ParseError::InvalidValue("scale", value))
                }
            }
            "searchsize" => value
                .parse::<i32>()
                .map(Attr::Searchsize)
                .map_err(|_| ParseError::InvalidValue("searchsize", value)),
            "sep" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::SepPoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::SepDouble)
                        .map_err(|_| ParseError::InvalidValue("sep", value))
                }
            }
            "shape" => value
                .parse::<Shape>()
                .map(Attr::Shape)
                .map_err(|_| ParseError::InvalidValue("shape", value)),
            "shapefile" => Ok(Attr::Shapefile(value.to_string())),
            "showboxes" => value
                .parse::<i32>()
                .map(Attr::Showboxes)
                .map_err(|_| ParseError::InvalidValue("showboxes", value)),
            "sides" => value
                .parse::<i32>()
                .map(Attr::Sides)
                .map_err(|_| ParseError::InvalidValue("sides", value)),
            "size" => {
                if let Ok(pt) = value.parse::<Point>() {
                    Ok(Attr::SizePoint(pt))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::SizeDouble)
                        .map_err(|_| ParseError::InvalidValue("size", value))
                }
            }
            "skew" => value
                .parse::<f64>()
                .map(Attr::Skew)
                .map_err(|_| ParseError::InvalidValue("skew", value)),
            "smoothing" => Ok(Attr::Smoothing(value.to_string())),
            "sortv" => value
                .parse::<i32>()
                .map(Attr::Sortv)
                .map_err(|_| ParseError::InvalidValue("sortv", value)),
            "splines" => {
                if let Ok(b) = parse_bool(value) {
                    Ok(Attr::SplinesBool(b))
                } else {
                    Ok(Attr::SplinesString(value.to_string()))
                }
            }
            "start" => Ok(Attr::Start(value.to_string())),
            "style" => value
                .parse::<Style>()
                .map(Attr::Style)
                .map_err(|_| ParseError::InvalidValue("style", value)),
            "stylesheet" => Ok(Attr::Stylesheet(value.to_string())),
            "tail_lp" => value
                .parse::<Point>()
                .map(Attr::Tail_lp)
                .map_err(|_| ParseError::InvalidValue("tail_lp", value)),
            "tailclip" => parse_bool(value).map(Attr::Tailclip),
            "tailhref" => Ok(Attr::Tailhref(value.to_string())),
            "taillabel" => Ok(Attr::Taillabel(value.to_string())),
            "tailport" => Ok(Attr::Tailport(value.to_string())),
            "tailtarget" => Ok(Attr::Tailtarget(value.to_string())),
            "tailtooltip" => Ok(Attr::Tailtooltip(value.to_string())),
            "tailurl" => Ok(Attr::TailURL(value.to_string())),
            "target" => Ok(Attr::Target(value.to_string())),
            "tbbalance" => Ok(Attr::TBbalance(value.to_string())),
            "tooltip" => Ok(Attr::Tooltip(value.to_string())),
            "truecolor" => parse_bool(value).map(Attr::Truecolor),
            "vertices" => {
                // Parse as colon-separated list of points
                let points: Result<Vec<Point>, _> =
                    value.split(':').map(|s| s.parse::<Point>()).collect();
                points
                    .map(Attr::Vertices)
                    .map_err(|_| ParseError::InvalidValue("vertices", value))
            }
            "viewport" => Ok(Attr::Viewport(value.to_string())),
            "voro_margin" => value
                .parse::<f64>()
                .map(Attr::Voro_margin)
                .map_err(|_| ParseError::InvalidValue("voro_margin", value)),
            "weight" => {
                if let Ok(i) = value.parse::<i32>() {
                    Ok(Attr::WeightInt(i))
                } else {
                    value
                        .parse::<f64>()
                        .map(Attr::WeightDouble)
                        .map_err(|_| ParseError::InvalidValue("weight", value))
                }
            }
            "width" => value
                .parse::<f64>()
                .map(Attr::Width)
                .map_err(|_| ParseError::InvalidValue("width", value)),
            "xdotversion" => Ok(Attr::Xdotversion(value.to_string())),
            "xlabel" => Ok(Attr::Xlabel(value.to_string())),
            "xlp" => value
                .parse::<Point>()
                .map(Attr::Xlp)
                .map_err(|_| ParseError::InvalidValue("xlp", value)),
            "z" => value
                .parse::<f64>()
                .map(Attr::Z)
                .map_err(|_| ParseError::InvalidValue("z", value)),
            _ => Err(ParseError::UnknownAttribute(name)),
        }
    }

    /// Returns the canonical attribute name
    pub fn name(&self) -> &'static str {
        match self {
            Attr::_background(_) => "_background",
            Attr::URL(_) => "URL",
            Attr::Area(_) => "area",
            Attr::Arrowhead(_) => "arrowhead",
            Attr::Arrowsize(_) => "arrowsize",
            Attr::Arrowtail(_) => "arrowtail",
            Attr::Bb(_) => "bb",
            Attr::Beautify(_) => "beautify",
            Attr::Bgcolor(_) => "bgcolor",
            Attr::Center(_) => "center",
            Attr::Charset(_) => "charset",
            Attr::Class(_) => "class",
            Attr::Cluster(_) => "cluster",
            Attr::Clusterrank(_) => "clusterrank",
            Attr::Color(_) => "color",
            Attr::Colorscheme(_) => "colorscheme",
            Attr::Comment(_) => "comment",
            Attr::Compound(_) => "compound",
            Attr::Concentrate(_) => "concentrate",
            Attr::Constraint(_) => "constraint",
            Attr::Damping(_) => "Damping",
            Attr::Decorate(_) => "decorate",
            Attr::Defaultdist(_) => "defaultdist",
            Attr::Dim(_) => "dim",
            Attr::Dimen(_) => "dimen",
            Attr::Dir(_) => "dir",
            Attr::DiredgeconstraintsString(_) => "diredgeconstraints",
            Attr::DiredgeconstraintsBool(_) => "diredgeconstraints",
            Attr::Distortion(_) => "distortion",
            Attr::Dpi(_) => "dpi",
            Attr::Edgehref(_) => "edgehref",
            Attr::Edgetarget(_) => "edgetarget",
            Attr::Edgetooltip(_) => "edgetooltip",
            Attr::EdgeURL(_) => "edgeURL",
            Attr::Epsilon(_) => "epsilon",
            Attr::EsepDouble(_) => "esep",
            Attr::EsepPoint(_) => "esep",
            Attr::Fillcolor(_) => "fillcolor",
            Attr::FixedsizeBool(_) => "fixedsize",
            Attr::FixedsizeString(_) => "fixedsize",
            Attr::Fontcolor(_) => "fontcolor",
            Attr::Fontname(_) => "fontname",
            Attr::Fontnames(_) => "fontnames",
            Attr::Fontpath(_) => "fontpath",
            Attr::Fontsize(_) => "fontsize",
            Attr::Forcelabels(_) => "forcelabels",
            Attr::Gradientangle(_) => "gradientangle",
            Attr::Group(_) => "group",
            Attr::Head_lp(_) => "head_lp",
            Attr::Headclip(_) => "headclip",
            Attr::Headhref(_) => "headhref",
            Attr::Headlabel(_) => "headlabel",
            Attr::Headport(_) => "headport",
            Attr::Headtarget(_) => "headtarget",
            Attr::Headtooltip(_) => "headtooltip",
            Attr::HeadURL(_) => "headURL",
            Attr::Height(_) => "height",
            Attr::Href(_) => "href",
            Attr::Id(_) => "id",
            Attr::Image(_) => "image",
            Attr::Imagepath(_) => "imagepath",
            Attr::Imagepos(_) => "imagepos",
            Attr::ImagescaleBool(_) => "imagescale",
            Attr::ImagescaleString(_) => "imagescale",
            Attr::Inputscale(_) => "inputscale",
            Attr::K(_) => "K",
            Attr::Label(_) => "label",
            Attr::Label_scheme(_) => "label_scheme",
            Attr::Labelangle(_) => "labelangle",
            Attr::Labeldistance(_) => "labeldistance",
            Attr::Labelfloat(_) => "labelfloat",
            Attr::Labelfontcolor(_) => "labelfontcolor",
            Attr::Labelfontname(_) => "labelfontname",
            Attr::Labelfontsize(_) => "labelfontsize",
            Attr::Labelhref(_) => "labelhref",
            Attr::Labeljust(_) => "labeljust",
            Attr::Labelloc(_) => "labelloc",
            Attr::Labeltarget(_) => "labeltarget",
            Attr::Labeltooltip(_) => "labeltooltip",
            Attr::LabelURL(_) => "labelURL",
            Attr::Landscape(_) => "landscape",
            Attr::Layer(_) => "layer",
            Attr::Layerlistsep(_) => "layerlistsep",
            Attr::Layers(_) => "layers",
            Attr::Layerselect(_) => "layerselect",
            Attr::Layersep(_) => "layersep",
            Attr::Layout(_) => "layout",
            Attr::Len(_) => "len",
            Attr::Levels(_) => "levels",
            Attr::Levelsgap(_) => "levelsgap",
            Attr::Lhead(_) => "lhead",
            Attr::Lheight(_) => "lheight",
            Attr::Linelength(_) => "linelength",
            Attr::Lp(_) => "lp",
            Attr::Ltail(_) => "ltail",
            Attr::Lwidth(_) => "lwidth",
            Attr::MarginDouble(_) => "margin",
            Attr::MarginPoint(_) => "margin",
            Attr::Maxiter(_) => "maxiter",
            Attr::Mclimit(_) => "mclimit",
            Attr::Mindist(_) => "mindist",
            Attr::Minlen(_) => "minlen",
            Attr::Mode(_) => "mode",
            Attr::Model(_) => "model",
            Attr::Newrank(_) => "newrank",
            Attr::Nodesep(_) => "nodesep",
            Attr::Nojustify(_) => "nojustify",
            Attr::NormalizeDouble(_) => "normalize",
            Attr::NormalizeBool(_) => "normalize",
            Attr::Notranslate(_) => "notranslate",
            Attr::Nslimit(_) => "nslimit",
            Attr::Nslimit1(_) => "nslimit1",
            Attr::Oneblock(_) => "oneblock",
            Attr::Ordering(_) => "ordering",
            Attr::OrientationDouble(_) => "orientation",
            Attr::OrientationString(_) => "orientation",
            Attr::Outputorder(_) => "outputorder",
            Attr::OverlapString(_) => "overlap",
            Attr::OverlapBool(_) => "overlap",
            Attr::Overlap_scaling(_) => "overlap_scaling",
            Attr::Overlap_shrink(_) => "overlap_shrink",
            Attr::PackBool(_) => "pack",
            Attr::PackInt(_) => "pack",
            Attr::Packmode(_) => "packmode",
            Attr::PadDouble(_) => "pad",
            Attr::PadPoint(_) => "pad",
            Attr::PageDouble(_) => "page",
            Attr::PagePoint(_) => "page",
            Attr::Pagedir(_) => "pagedir",
            Attr::Pencolor(_) => "pencolor",
            Attr::Penwidth(_) => "penwidth",
            Attr::Peripheries(_) => "peripheries",
            Attr::Pin(_) => "pin",
            Attr::PosPoint(_) => "pos",
            Attr::PosString(_) => "pos",
            Attr::QuadtreeString(_) => "quadtree",
            Attr::QuadtreeBool(_) => "quadtree",
            Attr::Quantum(_) => "quantum",
            Attr::Radius(_) => "radius",
            Attr::Rank(_) => "rank",
            Attr::Rankdir(_) => "rankdir",
            Attr::Ranksep(_) => "ranksep",
            Attr::RatioDouble(_) => "ratio",
            Attr::RatioString(_) => "ratio",
            Attr::Rects(_) => "rects",
            Attr::Regular(_) => "regular",
            Attr::Remincross(_) => "remincross",
            Attr::Repulsiveforce(_) => "repulsiveforce",
            Attr::Resolution(_) => "resolution",
            Attr::RootString(_) => "root",
            Attr::RootBool(_) => "root",
            Attr::Rotate(_) => "rotate",
            Attr::Rotation(_) => "rotation",
            Attr::Samehead(_) => "samehead",
            Attr::Sametail(_) => "sametail",
            Attr::Samplepoints(_) => "samplepoints",
            Attr::ScaleDouble(_) => "scale",
            Attr::ScalePoint(_) => "scale",
            Attr::Searchsize(_) => "searchsize",
            Attr::SepDouble(_) => "sep",
            Attr::SepPoint(_) => "sep",
            Attr::Shape(_) => "shape",
            Attr::Shapefile(_) => "shapefile",
            Attr::Showboxes(_) => "showboxes",
            Attr::Sides(_) => "sides",
            Attr::SizeDouble(_) => "size",
            Attr::SizePoint(_) => "size",
            Attr::Skew(_) => "skew",
            Attr::Smoothing(_) => "smoothing",
            Attr::Sortv(_) => "sortv",
            Attr::SplinesBool(_) => "splines",
            Attr::SplinesString(_) => "splines",
            Attr::Start(_) => "start",
            Attr::Style(_) => "style",
            Attr::Stylesheet(_) => "stylesheet",
            Attr::Tail_lp(_) => "tail_lp",
            Attr::Tailclip(_) => "tailclip",
            Attr::Tailhref(_) => "tailhref",
            Attr::Taillabel(_) => "taillabel",
            Attr::Tailport(_) => "tailport",
            Attr::Tailtarget(_) => "tailtarget",
            Attr::Tailtooltip(_) => "tailtooltip",
            Attr::TailURL(_) => "tailURL",
            Attr::Target(_) => "target",
            Attr::TBbalance(_) => "TBbalance",
            Attr::Tooltip(_) => "tooltip",
            Attr::Truecolor(_) => "truecolor",
            Attr::Vertices(_) => "vertices",
            Attr::Viewport(_) => "viewport",
            Attr::Voro_margin(_) => "voro_margin",
            Attr::WeightInt(_) => "weight",
            Attr::WeightDouble(_) => "weight",
            Attr::Width(_) => "width",
            Attr::Xdotversion(_) => "xdotversion",
            Attr::Xlabel(_) => "xlabel",
            Attr::Xlp(_) => "xlp",
            Attr::Z(_) => "z",
        }
    }
}

impl fmt::Display for Attr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attr::_background(v) => write!(f, "_background={}", v),
            Attr::URL(v) => write!(f, "URL={}", v),
            Attr::Area(v) => write!(f, "area={}", v),
            Attr::Arrowhead(v) => write!(f, "arrowhead={}", v),
            Attr::Arrowsize(v) => write!(f, "arrowsize={}", v),
            Attr::Arrowtail(v) => write!(f, "arrowtail={}", v),
            Attr::Bb(v) => write!(f, "bb={}", v),
            Attr::Beautify(v) => write!(f, "beautify={}", v),
            Attr::Bgcolor(v) => {
                let colors: Vec<String> = v.iter().map(|c| c.to_string()).collect();
                write!(f, "bgcolor={}", colors.join(":"))
            }
            Attr::Center(v) => write!(f, "center={}", v),
            Attr::Charset(v) => write!(f, "charset={}", v),
            Attr::Class(v) => write!(f, "class={}", v),
            Attr::Cluster(v) => write!(f, "cluster={}", v),
            Attr::Clusterrank(v) => write!(f, "clusterrank={}", v),
            Attr::Color(v) => {
                let colors: Vec<String> = v.iter().map(|c| c.to_string()).collect();
                write!(f, "color={}", colors.join(":"))
            }
            Attr::Colorscheme(v) => write!(f, "colorscheme={}", v),
            Attr::Comment(v) => write!(f, "comment={}", v),
            Attr::Compound(v) => write!(f, "compound={}", v),
            Attr::Concentrate(v) => write!(f, "concentrate={}", v),
            Attr::Constraint(v) => write!(f, "constraint={}", v),
            Attr::Damping(v) => write!(f, "Damping={}", v),
            Attr::Decorate(v) => write!(f, "decorate={}", v),
            Attr::Defaultdist(v) => write!(f, "defaultdist={}", v),
            Attr::Dim(v) => write!(f, "dim={}", v),
            Attr::Dimen(v) => write!(f, "dimen={}", v),
            Attr::Dir(v) => write!(f, "dir={}", v),
            Attr::DiredgeconstraintsString(v) => write!(f, "diredgeconstraints={}", v),
            Attr::DiredgeconstraintsBool(v) => write!(f, "diredgeconstraints={}", v),
            Attr::Distortion(v) => write!(f, "distortion={}", v),
            Attr::Dpi(v) => write!(f, "dpi={}", v),
            Attr::Edgehref(v) => write!(f, "edgehref={}", v),
            Attr::Edgetarget(v) => write!(f, "edgetarget={}", v),
            Attr::Edgetooltip(v) => write!(f, "edgetooltip={}", v),
            Attr::EdgeURL(v) => write!(f, "edgeURL={}", v),
            Attr::Epsilon(v) => write!(f, "epsilon={}", v),
            Attr::EsepDouble(v) => write!(f, "esep={}", v),
            Attr::EsepPoint(v) => write!(f, "esep={}", v),
            Attr::Fillcolor(v) => {
                let colors: Vec<String> = v.iter().map(|c| c.to_string()).collect();
                write!(f, "fillcolor={}", colors.join(":"))
            }
            Attr::FixedsizeBool(v) => write!(f, "fixedsize={}", v),
            Attr::FixedsizeString(v) => write!(f, "fixedsize={}", v),
            Attr::Fontcolor(v) => write!(f, "fontcolor={}", v),
            Attr::Fontname(v) => write!(f, "fontname={}", v),
            Attr::Fontnames(v) => write!(f, "fontnames={}", v),
            Attr::Fontpath(v) => write!(f, "fontpath={}", v),
            Attr::Fontsize(v) => write!(f, "fontsize={}", v),
            Attr::Forcelabels(v) => write!(f, "forcelabels={}", v),
            Attr::Gradientangle(v) => write!(f, "gradientangle={}", v),
            Attr::Group(v) => write!(f, "group={}", v),
            Attr::Head_lp(v) => write!(f, "head_lp={}", v),
            Attr::Headclip(v) => write!(f, "headclip={}", v),
            Attr::Headhref(v) => write!(f, "headhref={}", v),
            Attr::Headlabel(v) => write!(f, "headlabel={}", v),
            Attr::Headport(v) => write!(f, "headport={}", v),
            Attr::Headtarget(v) => write!(f, "headtarget={}", v),
            Attr::Headtooltip(v) => write!(f, "headtooltip={}", v),
            Attr::HeadURL(v) => write!(f, "headURL={}", v),
            Attr::Height(v) => write!(f, "height={}", v),
            Attr::Href(v) => write!(f, "href={}", v),
            Attr::Id(v) => write!(f, "id={}", v),
            Attr::Image(v) => write!(f, "image={}", v),
            Attr::Imagepath(v) => write!(f, "imagepath={}", v),
            Attr::Imagepos(v) => write!(f, "imagepos={}", v),
            Attr::ImagescaleBool(v) => write!(f, "imagescale={}", v),
            Attr::ImagescaleString(v) => write!(f, "imagescale={}", v),
            Attr::Inputscale(v) => write!(f, "inputscale={}", v),
            Attr::K(v) => write!(f, "K={}", v),
            Attr::Label(v) => write!(f, "label={}", v),
            Attr::Label_scheme(v) => write!(f, "label_scheme={}", v),
            Attr::Labelangle(v) => write!(f, "labelangle={}", v),
            Attr::Labeldistance(v) => write!(f, "labeldistance={}", v),
            Attr::Labelfloat(v) => write!(f, "labelfloat={}", v),
            Attr::Labelfontcolor(v) => write!(f, "labelfontcolor={}", v),
            Attr::Labelfontname(v) => write!(f, "labelfontname={}", v),
            Attr::Labelfontsize(v) => write!(f, "labelfontsize={}", v),
            Attr::Labelhref(v) => write!(f, "labelhref={}", v),
            Attr::Labeljust(v) => write!(f, "labeljust={}", v),
            Attr::Labelloc(v) => write!(f, "labelloc={}", v),
            Attr::Labeltarget(v) => write!(f, "labeltarget={}", v),
            Attr::Labeltooltip(v) => write!(f, "labeltooltip={}", v),
            Attr::LabelURL(v) => write!(f, "labelURL={}", v),
            Attr::Landscape(v) => write!(f, "landscape={}", v),
            Attr::Layer(v) => write!(f, "layer={}", v),
            Attr::Layerlistsep(v) => write!(f, "layerlistsep={}", v),
            Attr::Layers(v) => write!(f, "layers={}", v),
            Attr::Layerselect(v) => write!(f, "layerselect={}", v),
            Attr::Layersep(v) => write!(f, "layersep={}", v),
            Attr::Layout(v) => write!(f, "layout={}", v),
            Attr::Len(v) => write!(f, "len={}", v),
            Attr::Levels(v) => write!(f, "levels={}", v),
            Attr::Levelsgap(v) => write!(f, "levelsgap={}", v),
            Attr::Lhead(v) => write!(f, "lhead={}", v),
            Attr::Lheight(v) => write!(f, "lheight={}", v),
            Attr::Linelength(v) => write!(f, "linelength={}", v),
            Attr::Lp(v) => write!(f, "lp={}", v),
            Attr::Ltail(v) => write!(f, "ltail={}", v),
            Attr::Lwidth(v) => write!(f, "lwidth={}", v),
            Attr::MarginDouble(v) => write!(f, "margin={}", v),
            Attr::MarginPoint(v) => write!(f, "margin={}", v),
            Attr::Maxiter(v) => write!(f, "maxiter={}", v),
            Attr::Mclimit(v) => write!(f, "mclimit={}", v),
            Attr::Mindist(v) => write!(f, "mindist={}", v),
            Attr::Minlen(v) => write!(f, "minlen={}", v),
            Attr::Mode(v) => write!(f, "mode={}", v),
            Attr::Model(v) => write!(f, "model={}", v),
            Attr::Newrank(v) => write!(f, "newrank={}", v),
            Attr::Nodesep(v) => write!(f, "nodesep={}", v),
            Attr::Nojustify(v) => write!(f, "nojustify={}", v),
            Attr::NormalizeDouble(v) => write!(f, "normalize={}", v),
            Attr::NormalizeBool(v) => write!(f, "normalize={}", v),
            Attr::Notranslate(v) => write!(f, "notranslate={}", v),
            Attr::Nslimit(v) => write!(f, "nslimit={}", v),
            Attr::Nslimit1(v) => write!(f, "nslimit1={}", v),
            Attr::Oneblock(v) => write!(f, "oneblock={}", v),
            Attr::Ordering(v) => write!(f, "ordering={}", v),
            Attr::OrientationDouble(v) => write!(f, "orientation={}", v),
            Attr::OrientationString(v) => write!(f, "orientation={}", v),
            Attr::Outputorder(v) => write!(f, "outputorder={}", v),
            Attr::OverlapString(v) => write!(f, "overlap={}", v),
            Attr::OverlapBool(v) => write!(f, "overlap={}", v),
            Attr::Overlap_scaling(v) => write!(f, "overlap_scaling={}", v),
            Attr::Overlap_shrink(v) => write!(f, "overlap_shrink={}", v),
            Attr::PackBool(v) => write!(f, "pack={}", v),
            Attr::PackInt(v) => write!(f, "pack={}", v),
            Attr::Packmode(v) => write!(f, "packmode={}", v),
            Attr::PadDouble(v) => write!(f, "pad={}", v),
            Attr::PadPoint(v) => write!(f, "pad={}", v),
            Attr::PageDouble(v) => write!(f, "page={}", v),
            Attr::PagePoint(v) => write!(f, "page={}", v),
            Attr::Pagedir(v) => write!(f, "pagedir={}", v),
            Attr::Pencolor(v) => write!(f, "pencolor={}", v),
            Attr::Penwidth(v) => write!(f, "penwidth={}", v),
            Attr::Peripheries(v) => write!(f, "peripheries={}", v),
            Attr::Pin(v) => write!(f, "pin={}", v),
            Attr::PosPoint(v) => write!(f, "pos={}", v),
            Attr::PosString(v) => write!(f, "pos={}", v),
            Attr::QuadtreeString(v) => write!(f, "quadtree={}", v),
            Attr::QuadtreeBool(v) => write!(f, "quadtree={}", v),
            Attr::Quantum(v) => write!(f, "quantum={}", v),
            Attr::Radius(v) => write!(f, "radius={}", v),
            Attr::Rank(v) => write!(f, "rank={}", v),
            Attr::Rankdir(v) => write!(f, "rankdir={}", v),
            Attr::Ranksep(v) => {
                let values: Vec<String> = v.iter().map(|d| d.to_string()).collect();
                write!(f, "ranksep={}", values.join(":"))
            }
            Attr::RatioDouble(v) => write!(f, "ratio={}", v),
            Attr::RatioString(v) => write!(f, "ratio={}", v),
            Attr::Rects(v) => write!(f, "rects={}", v),
            Attr::Regular(v) => write!(f, "regular={}", v),
            Attr::Remincross(v) => write!(f, "remincross={}", v),
            Attr::Repulsiveforce(v) => write!(f, "repulsiveforce={}", v),
            Attr::Resolution(v) => write!(f, "resolution={}", v),
            Attr::RootString(v) => write!(f, "root={}", v),
            Attr::RootBool(v) => write!(f, "root={}", v),
            Attr::Rotate(v) => write!(f, "rotate={}", v),
            Attr::Rotation(v) => write!(f, "rotation={}", v),
            Attr::Samehead(v) => write!(f, "samehead={}", v),
            Attr::Sametail(v) => write!(f, "sametail={}", v),
            Attr::Samplepoints(v) => write!(f, "samplepoints={}", v),
            Attr::ScaleDouble(v) => write!(f, "scale={}", v),
            Attr::ScalePoint(v) => write!(f, "scale={}", v),
            Attr::Searchsize(v) => write!(f, "searchsize={}", v),
            Attr::SepDouble(v) => write!(f, "sep={}", v),
            Attr::SepPoint(v) => write!(f, "sep={}", v),
            Attr::Shape(v) => write!(f, "shape={}", v),
            Attr::Shapefile(v) => write!(f, "shapefile={}", v),
            Attr::Showboxes(v) => write!(f, "showboxes={}", v),
            Attr::Sides(v) => write!(f, "sides={}", v),
            Attr::SizeDouble(v) => write!(f, "size={}", v),
            Attr::SizePoint(v) => write!(f, "size={}", v),
            Attr::Skew(v) => write!(f, "skew={}", v),
            Attr::Smoothing(v) => write!(f, "smoothing={}", v),
            Attr::Sortv(v) => write!(f, "sortv={}", v),
            Attr::SplinesBool(v) => write!(f, "splines={}", v),
            Attr::SplinesString(v) => write!(f, "splines={}", v),
            Attr::Start(v) => write!(f, "start={}", v),
            Attr::Style(v) => write!(f, "style={}", v),
            Attr::Stylesheet(v) => write!(f, "stylesheet={}", v),
            Attr::Tail_lp(v) => write!(f, "tail_lp={}", v),
            Attr::Tailclip(v) => write!(f, "tailclip={}", v),
            Attr::Tailhref(v) => write!(f, "tailhref={}", v),
            Attr::Taillabel(v) => write!(f, "taillabel={}", v),
            Attr::Tailport(v) => write!(f, "tailport={}", v),
            Attr::Tailtarget(v) => write!(f, "tailtarget={}", v),
            Attr::Tailtooltip(v) => write!(f, "tailtooltip={}", v),
            Attr::TailURL(v) => write!(f, "tailURL={}", v),
            Attr::Target(v) => write!(f, "target={}", v),
            Attr::TBbalance(v) => write!(f, "TBbalance={}", v),
            Attr::Tooltip(v) => write!(f, "tooltip={}", v),
            Attr::Truecolor(v) => write!(f, "truecolor={}", v),
            Attr::Vertices(v) => {
                let points: Vec<String> = v.iter().map(|p| p.to_string()).collect();
                write!(f, "vertices={}", points.join(":"))
            }
            Attr::Viewport(v) => write!(f, "viewport={}", v),
            Attr::Voro_margin(v) => write!(f, "voro_margin={}", v),
            Attr::WeightInt(v) => write!(f, "weight={}", v),
            Attr::WeightDouble(v) => write!(f, "weight={}", v),
            Attr::Width(v) => write!(f, "width={}", v),
            Attr::Xdotversion(v) => write!(f, "xdotversion={}", v),
            Attr::Xlabel(v) => write!(f, "xlabel={}", v),
            Attr::Xlp(v) => write!(f, "xlp={}", v),
            Attr::Z(v) => write!(f, "z={}", v),
        }
    }
}

/// Helper function to parse boolean values
fn parse_bool<'a>(s: &'a str) -> Result<bool, ParseError<'a>> {
    // Try parsing as integer first - any nonzero integer is true
    if let Ok(n) = s.parse::<i64>() {
        return Ok(n != 0);
    }

    // Fall back to text-based parsing
    match s.to_lowercase().as_str() {
        "true" | "yes" => Ok(true),
        "false" | "no" => Ok(false),
        _ => Err(ParseError::InvalidValue("boolean", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_types() {
        let attr = Attr::parse("fontsize", "12").unwrap();
        assert_eq!(attr.name(), "fontsize");
        assert_eq!(attr.to_string(), "fontsize=12");

        let attr = Attr::parse("label", "Hello").unwrap();
        assert_eq!(attr.name(), "label");
        assert_eq!(attr.to_string(), "label=Hello");
    }

    #[test]
    fn test_parse_bool() {
        let attr = Attr::parse("center", "true").unwrap();
        assert_eq!(attr.name(), "center");
        assert_eq!(attr.to_string(), "center=true");

        let attr = Attr::parse("center", "false").unwrap();
        assert_eq!(attr.to_string(), "center=false");
    }

    #[test]
    fn test_parse_color() {
        let attr = Attr::parse("color", "red").unwrap();
        assert_eq!(attr.name(), "color");

        let attr = Attr::parse("bgcolor", "#ff0000").unwrap();
        assert_eq!(attr.name(), "bgcolor");
    }

    #[test]
    fn test_parse_shape() {
        let attr = Attr::parse("shape", "box").unwrap();
        assert_eq!(attr.name(), "shape");
        assert!(attr.to_string().contains("box"));
    }

    #[test]
    fn test_parse_arrow_type() {
        let attr = Attr::parse("arrowhead", "diamond").unwrap();
        assert_eq!(attr.name(), "arrowhead");
    }

    #[test]
    fn test_parse_dir_type() {
        let attr = Attr::parse("dir", "forward").unwrap();
        assert_eq!(attr.name(), "dir");
    }

    #[test]
    fn test_parse_unknown_attribute() {
        let result = Attr::parse("unknown_attr", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_value() {
        let result = Attr::parse("fontsize", "not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip() {
        let cases = vec![
            ("fontsize", "14"),
            ("label", "Test"),
            ("color", "blue"),
            ("center", "true"),
            ("width", "2.5"),
        ];

        for (name, value) in cases {
            let attr = Attr::parse(name, value).unwrap();
            let formatted = attr.to_string();
            assert!(formatted.starts_with(name));
            assert!(formatted.contains(value) || formatted.contains(&value));
        }
    }
}
