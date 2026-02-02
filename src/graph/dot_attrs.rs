//! Enumeration of Graphviz DOT format attributes with typed payloads.
//!
//! This module provides a comprehensive enum of all Graphviz attributes as documented at
//! <https://graphviz.org/doc/info/attrs.html>. Each variant includes typed payload data
//! corresponding to the attribute's type.
//!
//! # Examples
//!
//! ```
//! use graphitude::DotAttr;
//!
//! let attr = DotAttr::parse("color", "red").unwrap();
//! assert_eq!(attr.to_string(), "color=red");
//! ```

use std::fmt;

use super::dot_types::{
    ArrowType, Color, DirType, OutputMode, PageDir, Point, RankDir, RankType, Rect, Shape, Style,
};

/// Enumeration of all Graphviz DOT attribute names with typed payloads.
///
/// Each variant corresponds to an attribute documented at
/// <https://graphviz.org/doc/info/attrs.html> and contains the attribute's value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum DotAttr {
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
    /// Canvas background color
    Bgcolor(Color),
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
    /// Basic drawing color
    Color(Color),
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
    /// Constrain edges to point downwards
    Diredgeconstraints(String), // string | bool
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
    /// Margin around polygons for spline routing
    Esep(f64), // addDouble | addPoint
    /// Color to fill node or cluster background
    Fillcolor(Color),
    /// Whether to use specified width/height
    Fixedsize(String), // bool | string
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
    /// How image fills containing node
    Imagescale(String), // bool | string
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
    /// Margins of canvas or around label
    Margin(f64), // double | point
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
    /// Normalize coordinates of final layout
    Normalize(String), // double | bool
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
    /// Node shape rotation or graph orientation
    Orientation(String), // double | string
    /// Order in which nodes and edges are drawn
    Outputorder(OutputMode),
    /// How to remove node overlaps
    Overlap(String), // string | bool
    /// Scale layout to reduce node overlap
    Overlap_scaling(f64),
    /// Compression pass for overlap removal
    Overlap_shrink(bool),
    /// Pack connected components separately
    Pack(String), // bool | int
    /// How to pack connected components
    Packmode(String),
    /// Extend drawing area around graph
    Pad(f64), // double | point
    /// Width and height of output pages
    Page(f64), // double | point
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
    /// Position of node or spline control points
    Pos(String), // point | splineType
    /// Quadtree scheme to use
    Quadtree(String), // quadType | bool
    /// Quantum for node label dimensions
    Quantum(f64),
    /// Radius of rounded corners
    Radius(f64),
    /// Rank constraints on nodes in subgraph
    Rank(RankType),
    /// Direction of graph layout
    Rankdir(RankDir),
    /// Separation between ranks
    Ranksep(f64), // double | doubleList
    /// Aspect ratio for drawing
    Ratio(String), // double | string
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
    /// Nodes used as center of layout
    Root(String), // string | bool
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
    /// Scale layout after initial layout
    Scale(f64), // double | point
    /// Maximum edges with negative cut values
    Searchsize(i32),
    /// Margin around nodes for overlap removal
    Sep(f64), // addDouble | addPoint
    /// Shape of a node
    Shape(Shape),
    /// File containing user-supplied node content
    Shapefile(String),
    /// Print guide boxes for debugging
    Showboxes(i32),
    /// Number of sides for polygon shape
    Sides(i32),
    /// Maximum width and height of drawing
    Size(f64), // double | point
    /// Skew factor for polygon shape
    Skew(f64),
    /// Smooth out uneven node distribution
    Smoothing(String),
    /// Sort order for packmode packing
    Sortv(i32),
    /// How edges are represented
    Splines(String), // bool | string
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
    Vertices(String), // pointList
    /// Clipping window on final drawing
    Viewport(String),
    /// Tuning margin of Voronoi technique
    Voro_margin(f64),
    /// Weight of edge
    Weight(f64), // int | double
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

impl DotAttr {
    /// Parse an attribute from its name and string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use graphitude::DotAttr;
    ///
    /// let attr = DotAttr::parse("fontsize", "12").unwrap();
    /// let attr = DotAttr::parse("color", "red").unwrap();
    /// let attr = DotAttr::parse("shape", "box").unwrap();
    /// ```
    pub fn parse(name: &str, value: &str) -> Result<Self, String> {
        match name.to_lowercase().as_str() {
            "_background" => Ok(DotAttr::_background(value.to_string())),
            "url" => Ok(DotAttr::URL(value.to_string())),
            "area" => value
                .parse::<f64>()
                .map(DotAttr::Area)
                .map_err(|e| format!("Invalid area value: {}", e)),
            "arrowhead" => value
                .parse::<ArrowType>()
                .map(DotAttr::Arrowhead)
                .map_err(|e| format!("Invalid arrowhead: {}", e)),
            "arrowsize" => value
                .parse::<f64>()
                .map(DotAttr::Arrowsize)
                .map_err(|e| format!("Invalid arrowsize: {}", e)),
            "arrowtail" => value
                .parse::<ArrowType>()
                .map(DotAttr::Arrowtail)
                .map_err(|e| format!("Invalid arrowtail: {}", e)),
            "bb" => value
                .parse::<Rect>()
                .map(DotAttr::Bb)
                .map_err(|e| format!("Invalid bb: {}", e)),
            "beautify" => parse_bool(value).map(DotAttr::Beautify),
            "bgcolor" => value
                .parse::<Color>()
                .map(DotAttr::Bgcolor)
                .map_err(|e| format!("Invalid bgcolor: {}", e)),
            "center" => parse_bool(value).map(DotAttr::Center),
            "charset" => Ok(DotAttr::Charset(value.to_string())),
            "class" => Ok(DotAttr::Class(value.to_string())),
            "cluster" => parse_bool(value).map(DotAttr::Cluster),
            "clusterrank" => Ok(DotAttr::Clusterrank(value.to_string())),
            "color" => value
                .parse::<Color>()
                .map(DotAttr::Color)
                .map_err(|e| format!("Invalid color: {}", e)),
            "colorscheme" => Ok(DotAttr::Colorscheme(value.to_string())),
            "comment" => Ok(DotAttr::Comment(value.to_string())),
            "compound" => parse_bool(value).map(DotAttr::Compound),
            "concentrate" => parse_bool(value).map(DotAttr::Concentrate),
            "constraint" => parse_bool(value).map(DotAttr::Constraint),
            "damping" => value
                .parse::<f64>()
                .map(DotAttr::Damping)
                .map_err(|e| format!("Invalid damping: {}", e)),
            "decorate" => parse_bool(value).map(DotAttr::Decorate),
            "defaultdist" => value
                .parse::<f64>()
                .map(DotAttr::Defaultdist)
                .map_err(|e| format!("Invalid defaultdist: {}", e)),
            "dim" => value
                .parse::<i32>()
                .map(DotAttr::Dim)
                .map_err(|e| format!("Invalid dim: {}", e)),
            "dimen" => value
                .parse::<i32>()
                .map(DotAttr::Dimen)
                .map_err(|e| format!("Invalid dimen: {}", e)),
            "dir" => value
                .parse::<DirType>()
                .map(DotAttr::Dir)
                .map_err(|e| format!("Invalid dir: {}", e)),
            "diredgeconstraints" => Ok(DotAttr::Diredgeconstraints(value.to_string())),
            "distortion" => value
                .parse::<f64>()
                .map(DotAttr::Distortion)
                .map_err(|e| format!("Invalid distortion: {}", e)),
            "dpi" => value
                .parse::<f64>()
                .map(DotAttr::Dpi)
                .map_err(|e| format!("Invalid dpi: {}", e)),
            "edgehref" => Ok(DotAttr::Edgehref(value.to_string())),
            "edgetarget" => Ok(DotAttr::Edgetarget(value.to_string())),
            "edgetooltip" => Ok(DotAttr::Edgetooltip(value.to_string())),
            "edgeurl" => Ok(DotAttr::EdgeURL(value.to_string())),
            "epsilon" => value
                .parse::<f64>()
                .map(DotAttr::Epsilon)
                .map_err(|e| format!("Invalid epsilon: {}", e)),
            "esep" => value
                .parse::<f64>()
                .map(DotAttr::Esep)
                .map_err(|e| format!("Invalid esep: {}", e)),
            "fillcolor" => value
                .parse::<Color>()
                .map(DotAttr::Fillcolor)
                .map_err(|e| format!("Invalid fillcolor: {}", e)),
            "fixedsize" => Ok(DotAttr::Fixedsize(value.to_string())),
            "fontcolor" => value
                .parse::<Color>()
                .map(DotAttr::Fontcolor)
                .map_err(|e| format!("Invalid fontcolor: {}", e)),
            "fontname" => Ok(DotAttr::Fontname(value.to_string())),
            "fontnames" => Ok(DotAttr::Fontnames(value.to_string())),
            "fontpath" => Ok(DotAttr::Fontpath(value.to_string())),
            "fontsize" => value
                .parse::<f64>()
                .map(DotAttr::Fontsize)
                .map_err(|e| format!("Invalid fontsize: {}", e)),
            "forcelabels" => parse_bool(value).map(DotAttr::Forcelabels),
            "gradientangle" => value
                .parse::<i32>()
                .map(DotAttr::Gradientangle)
                .map_err(|e| format!("Invalid gradientangle: {}", e)),
            "group" => Ok(DotAttr::Group(value.to_string())),
            "head_lp" => value
                .parse::<Point>()
                .map(DotAttr::Head_lp)
                .map_err(|e| format!("Invalid head_lp: {}", e)),
            "headclip" => parse_bool(value).map(DotAttr::Headclip),
            "headhref" => Ok(DotAttr::Headhref(value.to_string())),
            "headlabel" => Ok(DotAttr::Headlabel(value.to_string())),
            "headport" => Ok(DotAttr::Headport(value.to_string())),
            "headtarget" => Ok(DotAttr::Headtarget(value.to_string())),
            "headtooltip" => Ok(DotAttr::Headtooltip(value.to_string())),
            "headurl" => Ok(DotAttr::HeadURL(value.to_string())),
            "height" => value
                .parse::<f64>()
                .map(DotAttr::Height)
                .map_err(|e| format!("Invalid height: {}", e)),
            "href" => Ok(DotAttr::Href(value.to_string())),
            "id" => Ok(DotAttr::Id(value.to_string())),
            "image" => Ok(DotAttr::Image(value.to_string())),
            "imagepath" => Ok(DotAttr::Imagepath(value.to_string())),
            "imagepos" => Ok(DotAttr::Imagepos(value.to_string())),
            "imagescale" => Ok(DotAttr::Imagescale(value.to_string())),
            "inputscale" => value
                .parse::<f64>()
                .map(DotAttr::Inputscale)
                .map_err(|e| format!("Invalid inputscale: {}", e)),
            "k" => value
                .parse::<f64>()
                .map(DotAttr::K)
                .map_err(|e| format!("Invalid k: {}", e)),
            "label" => Ok(DotAttr::Label(value.to_string())),
            "label_scheme" => value
                .parse::<i32>()
                .map(DotAttr::Label_scheme)
                .map_err(|e| format!("Invalid label_scheme: {}", e)),
            "labelangle" => value
                .parse::<f64>()
                .map(DotAttr::Labelangle)
                .map_err(|e| format!("Invalid labelangle: {}", e)),
            "labeldistance" => value
                .parse::<f64>()
                .map(DotAttr::Labeldistance)
                .map_err(|e| format!("Invalid labeldistance: {}", e)),
            "labelfloat" => parse_bool(value).map(DotAttr::Labelfloat),
            "labelfontcolor" => value
                .parse::<Color>()
                .map(DotAttr::Labelfontcolor)
                .map_err(|e| format!("Invalid labelfontcolor: {}", e)),
            "labelfontname" => Ok(DotAttr::Labelfontname(value.to_string())),
            "labelfontsize" => value
                .parse::<f64>()
                .map(DotAttr::Labelfontsize)
                .map_err(|e| format!("Invalid labelfontsize: {}", e)),
            "labelhref" => Ok(DotAttr::Labelhref(value.to_string())),
            "labeljust" => Ok(DotAttr::Labeljust(value.to_string())),
            "labelloc" => Ok(DotAttr::Labelloc(value.to_string())),
            "labeltarget" => Ok(DotAttr::Labeltarget(value.to_string())),
            "labeltooltip" => Ok(DotAttr::Labeltooltip(value.to_string())),
            "labelurl" => Ok(DotAttr::LabelURL(value.to_string())),
            "landscape" => parse_bool(value).map(DotAttr::Landscape),
            "layer" => Ok(DotAttr::Layer(value.to_string())),
            "layerlistsep" => Ok(DotAttr::Layerlistsep(value.to_string())),
            "layers" => Ok(DotAttr::Layers(value.to_string())),
            "layerselect" => Ok(DotAttr::Layerselect(value.to_string())),
            "layersep" => Ok(DotAttr::Layersep(value.to_string())),
            "layout" => Ok(DotAttr::Layout(value.to_string())),
            "len" => value
                .parse::<f64>()
                .map(DotAttr::Len)
                .map_err(|e| format!("Invalid len: {}", e)),
            "levels" => value
                .parse::<i32>()
                .map(DotAttr::Levels)
                .map_err(|e| format!("Invalid levels: {}", e)),
            "levelsgap" => value
                .parse::<f64>()
                .map(DotAttr::Levelsgap)
                .map_err(|e| format!("Invalid levelsgap: {}", e)),
            "lhead" => Ok(DotAttr::Lhead(value.to_string())),
            "lheight" => value
                .parse::<f64>()
                .map(DotAttr::Lheight)
                .map_err(|e| format!("Invalid lheight: {}", e)),
            "linelength" => value
                .parse::<i32>()
                .map(DotAttr::Linelength)
                .map_err(|e| format!("Invalid linelength: {}", e)),
            "lp" => value
                .parse::<Point>()
                .map(DotAttr::Lp)
                .map_err(|e| format!("Invalid lp: {}", e)),
            "ltail" => Ok(DotAttr::Ltail(value.to_string())),
            "lwidth" => value
                .parse::<f64>()
                .map(DotAttr::Lwidth)
                .map_err(|e| format!("Invalid lwidth: {}", e)),
            "margin" => value
                .parse::<f64>()
                .map(DotAttr::Margin)
                .map_err(|e| format!("Invalid margin: {}", e)),
            "maxiter" => value
                .parse::<i32>()
                .map(DotAttr::Maxiter)
                .map_err(|e| format!("Invalid maxiter: {}", e)),
            "mclimit" => value
                .parse::<f64>()
                .map(DotAttr::Mclimit)
                .map_err(|e| format!("Invalid mclimit: {}", e)),
            "mindist" => value
                .parse::<f64>()
                .map(DotAttr::Mindist)
                .map_err(|e| format!("Invalid mindist: {}", e)),
            "minlen" => value
                .parse::<i32>()
                .map(DotAttr::Minlen)
                .map_err(|e| format!("Invalid minlen: {}", e)),
            "mode" => Ok(DotAttr::Mode(value.to_string())),
            "model" => Ok(DotAttr::Model(value.to_string())),
            "newrank" => parse_bool(value).map(DotAttr::Newrank),
            "nodesep" => value
                .parse::<f64>()
                .map(DotAttr::Nodesep)
                .map_err(|e| format!("Invalid nodesep: {}", e)),
            "nojustify" => parse_bool(value).map(DotAttr::Nojustify),
            "normalize" => Ok(DotAttr::Normalize(value.to_string())),
            "notranslate" => parse_bool(value).map(DotAttr::Notranslate),
            "nslimit" => value
                .parse::<f64>()
                .map(DotAttr::Nslimit)
                .map_err(|e| format!("Invalid nslimit: {}", e)),
            "nslimit1" => value
                .parse::<f64>()
                .map(DotAttr::Nslimit1)
                .map_err(|e| format!("Invalid nslimit1: {}", e)),
            "oneblock" => parse_bool(value).map(DotAttr::Oneblock),
            "ordering" => Ok(DotAttr::Ordering(value.to_string())),
            "orientation" => Ok(DotAttr::Orientation(value.to_string())),
            "outputorder" => value
                .parse::<OutputMode>()
                .map(DotAttr::Outputorder)
                .map_err(|e| format!("Invalid outputorder: {}", e)),
            "overlap" => Ok(DotAttr::Overlap(value.to_string())),
            "overlap_scaling" => value
                .parse::<f64>()
                .map(DotAttr::Overlap_scaling)
                .map_err(|e| format!("Invalid overlap_scaling: {}", e)),
            "overlap_shrink" => parse_bool(value).map(DotAttr::Overlap_shrink),
            "pack" => Ok(DotAttr::Pack(value.to_string())),
            "packmode" => Ok(DotAttr::Packmode(value.to_string())),
            "pad" => value
                .parse::<f64>()
                .map(DotAttr::Pad)
                .map_err(|e| format!("Invalid pad: {}", e)),
            "page" => value
                .parse::<f64>()
                .map(DotAttr::Page)
                .map_err(|e| format!("Invalid page: {}", e)),
            "pagedir" => value
                .parse::<PageDir>()
                .map(DotAttr::Pagedir)
                .map_err(|e| format!("Invalid pagedir: {}", e)),
            "pencolor" => value
                .parse::<Color>()
                .map(DotAttr::Pencolor)
                .map_err(|e| format!("Invalid pencolor: {}", e)),
            "penwidth" => value
                .parse::<f64>()
                .map(DotAttr::Penwidth)
                .map_err(|e| format!("Invalid penwidth: {}", e)),
            "peripheries" => value
                .parse::<i32>()
                .map(DotAttr::Peripheries)
                .map_err(|e| format!("Invalid peripheries: {}", e)),
            "pin" => parse_bool(value).map(DotAttr::Pin),
            "pos" => Ok(DotAttr::Pos(value.to_string())),
            "quadtree" => Ok(DotAttr::Quadtree(value.to_string())),
            "quantum" => value
                .parse::<f64>()
                .map(DotAttr::Quantum)
                .map_err(|e| format!("Invalid quantum: {}", e)),
            "radius" => value
                .parse::<f64>()
                .map(DotAttr::Radius)
                .map_err(|e| format!("Invalid radius: {}", e)),
            "rank" => value
                .parse::<RankType>()
                .map(DotAttr::Rank)
                .map_err(|e| format!("Invalid rank: {}", e)),
            "rankdir" => value
                .parse::<RankDir>()
                .map(DotAttr::Rankdir)
                .map_err(|e| format!("Invalid rankdir: {}", e)),
            "ranksep" => value
                .parse::<f64>()
                .map(DotAttr::Ranksep)
                .map_err(|e| format!("Invalid ranksep: {}", e)),
            "ratio" => Ok(DotAttr::Ratio(value.to_string())),
            "rects" => value
                .parse::<Rect>()
                .map(DotAttr::Rects)
                .map_err(|e| format!("Invalid rects: {}", e)),
            "regular" => parse_bool(value).map(DotAttr::Regular),
            "remincross" => parse_bool(value).map(DotAttr::Remincross),
            "repulsiveforce" => value
                .parse::<f64>()
                .map(DotAttr::Repulsiveforce)
                .map_err(|e| format!("Invalid repulsiveforce: {}", e)),
            "resolution" => value
                .parse::<f64>()
                .map(DotAttr::Resolution)
                .map_err(|e| format!("Invalid resolution: {}", e)),
            "root" => Ok(DotAttr::Root(value.to_string())),
            "rotate" => value
                .parse::<i32>()
                .map(DotAttr::Rotate)
                .map_err(|e| format!("Invalid rotate: {}", e)),
            "rotation" => value
                .parse::<f64>()
                .map(DotAttr::Rotation)
                .map_err(|e| format!("Invalid rotation: {}", e)),
            "samehead" => Ok(DotAttr::Samehead(value.to_string())),
            "sametail" => Ok(DotAttr::Sametail(value.to_string())),
            "samplepoints" => value
                .parse::<i32>()
                .map(DotAttr::Samplepoints)
                .map_err(|e| format!("Invalid samplepoints: {}", e)),
            "scale" => value
                .parse::<f64>()
                .map(DotAttr::Scale)
                .map_err(|e| format!("Invalid scale: {}", e)),
            "searchsize" => value
                .parse::<i32>()
                .map(DotAttr::Searchsize)
                .map_err(|e| format!("Invalid searchsize: {}", e)),
            "sep" => value
                .parse::<f64>()
                .map(DotAttr::Sep)
                .map_err(|e| format!("Invalid sep: {}", e)),
            "shape" => value
                .parse::<Shape>()
                .map(DotAttr::Shape)
                .map_err(|e| format!("Invalid shape: {}", e)),
            "shapefile" => Ok(DotAttr::Shapefile(value.to_string())),
            "showboxes" => value
                .parse::<i32>()
                .map(DotAttr::Showboxes)
                .map_err(|e| format!("Invalid showboxes: {}", e)),
            "sides" => value
                .parse::<i32>()
                .map(DotAttr::Sides)
                .map_err(|e| format!("Invalid sides: {}", e)),
            "size" => value
                .parse::<f64>()
                .map(DotAttr::Size)
                .map_err(|e| format!("Invalid size: {}", e)),
            "skew" => value
                .parse::<f64>()
                .map(DotAttr::Skew)
                .map_err(|e| format!("Invalid skew: {}", e)),
            "smoothing" => Ok(DotAttr::Smoothing(value.to_string())),
            "sortv" => value
                .parse::<i32>()
                .map(DotAttr::Sortv)
                .map_err(|e| format!("Invalid sortv: {}", e)),
            "splines" => Ok(DotAttr::Splines(value.to_string())),
            "start" => Ok(DotAttr::Start(value.to_string())),
            "style" => value
                .parse::<Style>()
                .map(DotAttr::Style)
                .map_err(|e| format!("Invalid style: {}", e)),
            "stylesheet" => Ok(DotAttr::Stylesheet(value.to_string())),
            "tail_lp" => value
                .parse::<Point>()
                .map(DotAttr::Tail_lp)
                .map_err(|e| format!("Invalid tail_lp: {}", e)),
            "tailclip" => parse_bool(value).map(DotAttr::Tailclip),
            "tailhref" => Ok(DotAttr::Tailhref(value.to_string())),
            "taillabel" => Ok(DotAttr::Taillabel(value.to_string())),
            "tailport" => Ok(DotAttr::Tailport(value.to_string())),
            "tailtarget" => Ok(DotAttr::Tailtarget(value.to_string())),
            "tailtooltip" => Ok(DotAttr::Tailtooltip(value.to_string())),
            "tailurl" => Ok(DotAttr::TailURL(value.to_string())),
            "target" => Ok(DotAttr::Target(value.to_string())),
            "tbbalance" => Ok(DotAttr::TBbalance(value.to_string())),
            "tooltip" => Ok(DotAttr::Tooltip(value.to_string())),
            "truecolor" => parse_bool(value).map(DotAttr::Truecolor),
            "vertices" => Ok(DotAttr::Vertices(value.to_string())),
            "viewport" => Ok(DotAttr::Viewport(value.to_string())),
            "voro_margin" => value
                .parse::<f64>()
                .map(DotAttr::Voro_margin)
                .map_err(|e| format!("Invalid voro_margin: {}", e)),
            "weight" => value
                .parse::<f64>()
                .map(DotAttr::Weight)
                .map_err(|e| format!("Invalid weight: {}", e)),
            "width" => value
                .parse::<f64>()
                .map(DotAttr::Width)
                .map_err(|e| format!("Invalid width: {}", e)),
            "xdotversion" => Ok(DotAttr::Xdotversion(value.to_string())),
            "xlabel" => Ok(DotAttr::Xlabel(value.to_string())),
            "xlp" => value
                .parse::<Point>()
                .map(DotAttr::Xlp)
                .map_err(|e| format!("Invalid xlp: {}", e)),
            "z" => value
                .parse::<f64>()
                .map(DotAttr::Z)
                .map_err(|e| format!("Invalid z: {}", e)),
            _ => Err(format!("Unknown attribute: {}", name)),
        }
    }

    /// Returns the canonical attribute name
    pub fn name(&self) -> &'static str {
        match self {
            DotAttr::_background(_) => "_background",
            DotAttr::URL(_) => "URL",
            DotAttr::Area(_) => "area",
            DotAttr::Arrowhead(_) => "arrowhead",
            DotAttr::Arrowsize(_) => "arrowsize",
            DotAttr::Arrowtail(_) => "arrowtail",
            DotAttr::Bb(_) => "bb",
            DotAttr::Beautify(_) => "beautify",
            DotAttr::Bgcolor(_) => "bgcolor",
            DotAttr::Center(_) => "center",
            DotAttr::Charset(_) => "charset",
            DotAttr::Class(_) => "class",
            DotAttr::Cluster(_) => "cluster",
            DotAttr::Clusterrank(_) => "clusterrank",
            DotAttr::Color(_) => "color",
            DotAttr::Colorscheme(_) => "colorscheme",
            DotAttr::Comment(_) => "comment",
            DotAttr::Compound(_) => "compound",
            DotAttr::Concentrate(_) => "concentrate",
            DotAttr::Constraint(_) => "constraint",
            DotAttr::Damping(_) => "Damping",
            DotAttr::Decorate(_) => "decorate",
            DotAttr::Defaultdist(_) => "defaultdist",
            DotAttr::Dim(_) => "dim",
            DotAttr::Dimen(_) => "dimen",
            DotAttr::Dir(_) => "dir",
            DotAttr::Diredgeconstraints(_) => "diredgeconstraints",
            DotAttr::Distortion(_) => "distortion",
            DotAttr::Dpi(_) => "dpi",
            DotAttr::Edgehref(_) => "edgehref",
            DotAttr::Edgetarget(_) => "edgetarget",
            DotAttr::Edgetooltip(_) => "edgetooltip",
            DotAttr::EdgeURL(_) => "edgeURL",
            DotAttr::Epsilon(_) => "epsilon",
            DotAttr::Esep(_) => "esep",
            DotAttr::Fillcolor(_) => "fillcolor",
            DotAttr::Fixedsize(_) => "fixedsize",
            DotAttr::Fontcolor(_) => "fontcolor",
            DotAttr::Fontname(_) => "fontname",
            DotAttr::Fontnames(_) => "fontnames",
            DotAttr::Fontpath(_) => "fontpath",
            DotAttr::Fontsize(_) => "fontsize",
            DotAttr::Forcelabels(_) => "forcelabels",
            DotAttr::Gradientangle(_) => "gradientangle",
            DotAttr::Group(_) => "group",
            DotAttr::Head_lp(_) => "head_lp",
            DotAttr::Headclip(_) => "headclip",
            DotAttr::Headhref(_) => "headhref",
            DotAttr::Headlabel(_) => "headlabel",
            DotAttr::Headport(_) => "headport",
            DotAttr::Headtarget(_) => "headtarget",
            DotAttr::Headtooltip(_) => "headtooltip",
            DotAttr::HeadURL(_) => "headURL",
            DotAttr::Height(_) => "height",
            DotAttr::Href(_) => "href",
            DotAttr::Id(_) => "id",
            DotAttr::Image(_) => "image",
            DotAttr::Imagepath(_) => "imagepath",
            DotAttr::Imagepos(_) => "imagepos",
            DotAttr::Imagescale(_) => "imagescale",
            DotAttr::Inputscale(_) => "inputscale",
            DotAttr::K(_) => "K",
            DotAttr::Label(_) => "label",
            DotAttr::Label_scheme(_) => "label_scheme",
            DotAttr::Labelangle(_) => "labelangle",
            DotAttr::Labeldistance(_) => "labeldistance",
            DotAttr::Labelfloat(_) => "labelfloat",
            DotAttr::Labelfontcolor(_) => "labelfontcolor",
            DotAttr::Labelfontname(_) => "labelfontname",
            DotAttr::Labelfontsize(_) => "labelfontsize",
            DotAttr::Labelhref(_) => "labelhref",
            DotAttr::Labeljust(_) => "labeljust",
            DotAttr::Labelloc(_) => "labelloc",
            DotAttr::Labeltarget(_) => "labeltarget",
            DotAttr::Labeltooltip(_) => "labeltooltip",
            DotAttr::LabelURL(_) => "labelURL",
            DotAttr::Landscape(_) => "landscape",
            DotAttr::Layer(_) => "layer",
            DotAttr::Layerlistsep(_) => "layerlistsep",
            DotAttr::Layers(_) => "layers",
            DotAttr::Layerselect(_) => "layerselect",
            DotAttr::Layersep(_) => "layersep",
            DotAttr::Layout(_) => "layout",
            DotAttr::Len(_) => "len",
            DotAttr::Levels(_) => "levels",
            DotAttr::Levelsgap(_) => "levelsgap",
            DotAttr::Lhead(_) => "lhead",
            DotAttr::Lheight(_) => "lheight",
            DotAttr::Linelength(_) => "linelength",
            DotAttr::Lp(_) => "lp",
            DotAttr::Ltail(_) => "ltail",
            DotAttr::Lwidth(_) => "lwidth",
            DotAttr::Margin(_) => "margin",
            DotAttr::Maxiter(_) => "maxiter",
            DotAttr::Mclimit(_) => "mclimit",
            DotAttr::Mindist(_) => "mindist",
            DotAttr::Minlen(_) => "minlen",
            DotAttr::Mode(_) => "mode",
            DotAttr::Model(_) => "model",
            DotAttr::Newrank(_) => "newrank",
            DotAttr::Nodesep(_) => "nodesep",
            DotAttr::Nojustify(_) => "nojustify",
            DotAttr::Normalize(_) => "normalize",
            DotAttr::Notranslate(_) => "notranslate",
            DotAttr::Nslimit(_) => "nslimit",
            DotAttr::Nslimit1(_) => "nslimit1",
            DotAttr::Oneblock(_) => "oneblock",
            DotAttr::Ordering(_) => "ordering",
            DotAttr::Orientation(_) => "orientation",
            DotAttr::Outputorder(_) => "outputorder",
            DotAttr::Overlap(_) => "overlap",
            DotAttr::Overlap_scaling(_) => "overlap_scaling",
            DotAttr::Overlap_shrink(_) => "overlap_shrink",
            DotAttr::Pack(_) => "pack",
            DotAttr::Packmode(_) => "packmode",
            DotAttr::Pad(_) => "pad",
            DotAttr::Page(_) => "page",
            DotAttr::Pagedir(_) => "pagedir",
            DotAttr::Pencolor(_) => "pencolor",
            DotAttr::Penwidth(_) => "penwidth",
            DotAttr::Peripheries(_) => "peripheries",
            DotAttr::Pin(_) => "pin",
            DotAttr::Pos(_) => "pos",
            DotAttr::Quadtree(_) => "quadtree",
            DotAttr::Quantum(_) => "quantum",
            DotAttr::Radius(_) => "radius",
            DotAttr::Rank(_) => "rank",
            DotAttr::Rankdir(_) => "rankdir",
            DotAttr::Ranksep(_) => "ranksep",
            DotAttr::Ratio(_) => "ratio",
            DotAttr::Rects(_) => "rects",
            DotAttr::Regular(_) => "regular",
            DotAttr::Remincross(_) => "remincross",
            DotAttr::Repulsiveforce(_) => "repulsiveforce",
            DotAttr::Resolution(_) => "resolution",
            DotAttr::Root(_) => "root",
            DotAttr::Rotate(_) => "rotate",
            DotAttr::Rotation(_) => "rotation",
            DotAttr::Samehead(_) => "samehead",
            DotAttr::Sametail(_) => "sametail",
            DotAttr::Samplepoints(_) => "samplepoints",
            DotAttr::Scale(_) => "scale",
            DotAttr::Searchsize(_) => "searchsize",
            DotAttr::Sep(_) => "sep",
            DotAttr::Shape(_) => "shape",
            DotAttr::Shapefile(_) => "shapefile",
            DotAttr::Showboxes(_) => "showboxes",
            DotAttr::Sides(_) => "sides",
            DotAttr::Size(_) => "size",
            DotAttr::Skew(_) => "skew",
            DotAttr::Smoothing(_) => "smoothing",
            DotAttr::Sortv(_) => "sortv",
            DotAttr::Splines(_) => "splines",
            DotAttr::Start(_) => "start",
            DotAttr::Style(_) => "style",
            DotAttr::Stylesheet(_) => "stylesheet",
            DotAttr::Tail_lp(_) => "tail_lp",
            DotAttr::Tailclip(_) => "tailclip",
            DotAttr::Tailhref(_) => "tailhref",
            DotAttr::Taillabel(_) => "taillabel",
            DotAttr::Tailport(_) => "tailport",
            DotAttr::Tailtarget(_) => "tailtarget",
            DotAttr::Tailtooltip(_) => "tailtooltip",
            DotAttr::TailURL(_) => "tailURL",
            DotAttr::Target(_) => "target",
            DotAttr::TBbalance(_) => "TBbalance",
            DotAttr::Tooltip(_) => "tooltip",
            DotAttr::Truecolor(_) => "truecolor",
            DotAttr::Vertices(_) => "vertices",
            DotAttr::Viewport(_) => "viewport",
            DotAttr::Voro_margin(_) => "voro_margin",
            DotAttr::Weight(_) => "weight",
            DotAttr::Width(_) => "width",
            DotAttr::Xdotversion(_) => "xdotversion",
            DotAttr::Xlabel(_) => "xlabel",
            DotAttr::Xlp(_) => "xlp",
            DotAttr::Z(_) => "z",
        }
    }
}

impl fmt::Display for DotAttr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DotAttr::_background(v) => write!(f, "_background={}", v),
            DotAttr::URL(v) => write!(f, "URL={}", v),
            DotAttr::Area(v) => write!(f, "area={}", v),
            DotAttr::Arrowhead(v) => write!(f, "arrowhead={}", v),
            DotAttr::Arrowsize(v) => write!(f, "arrowsize={}", v),
            DotAttr::Arrowtail(v) => write!(f, "arrowtail={}", v),
            DotAttr::Bb(v) => write!(f, "bb={}", v),
            DotAttr::Beautify(v) => write!(f, "beautify={}", v),
            DotAttr::Bgcolor(v) => write!(f, "bgcolor={}", v),
            DotAttr::Center(v) => write!(f, "center={}", v),
            DotAttr::Charset(v) => write!(f, "charset={}", v),
            DotAttr::Class(v) => write!(f, "class={}", v),
            DotAttr::Cluster(v) => write!(f, "cluster={}", v),
            DotAttr::Clusterrank(v) => write!(f, "clusterrank={}", v),
            DotAttr::Color(v) => write!(f, "color={}", v),
            DotAttr::Colorscheme(v) => write!(f, "colorscheme={}", v),
            DotAttr::Comment(v) => write!(f, "comment={}", v),
            DotAttr::Compound(v) => write!(f, "compound={}", v),
            DotAttr::Concentrate(v) => write!(f, "concentrate={}", v),
            DotAttr::Constraint(v) => write!(f, "constraint={}", v),
            DotAttr::Damping(v) => write!(f, "Damping={}", v),
            DotAttr::Decorate(v) => write!(f, "decorate={}", v),
            DotAttr::Defaultdist(v) => write!(f, "defaultdist={}", v),
            DotAttr::Dim(v) => write!(f, "dim={}", v),
            DotAttr::Dimen(v) => write!(f, "dimen={}", v),
            DotAttr::Dir(v) => write!(f, "dir={}", v),
            DotAttr::Diredgeconstraints(v) => write!(f, "diredgeconstraints={}", v),
            DotAttr::Distortion(v) => write!(f, "distortion={}", v),
            DotAttr::Dpi(v) => write!(f, "dpi={}", v),
            DotAttr::Edgehref(v) => write!(f, "edgehref={}", v),
            DotAttr::Edgetarget(v) => write!(f, "edgetarget={}", v),
            DotAttr::Edgetooltip(v) => write!(f, "edgetooltip={}", v),
            DotAttr::EdgeURL(v) => write!(f, "edgeURL={}", v),
            DotAttr::Epsilon(v) => write!(f, "epsilon={}", v),
            DotAttr::Esep(v) => write!(f, "esep={}", v),
            DotAttr::Fillcolor(v) => write!(f, "fillcolor={}", v),
            DotAttr::Fixedsize(v) => write!(f, "fixedsize={}", v),
            DotAttr::Fontcolor(v) => write!(f, "fontcolor={}", v),
            DotAttr::Fontname(v) => write!(f, "fontname={}", v),
            DotAttr::Fontnames(v) => write!(f, "fontnames={}", v),
            DotAttr::Fontpath(v) => write!(f, "fontpath={}", v),
            DotAttr::Fontsize(v) => write!(f, "fontsize={}", v),
            DotAttr::Forcelabels(v) => write!(f, "forcelabels={}", v),
            DotAttr::Gradientangle(v) => write!(f, "gradientangle={}", v),
            DotAttr::Group(v) => write!(f, "group={}", v),
            DotAttr::Head_lp(v) => write!(f, "head_lp={}", v),
            DotAttr::Headclip(v) => write!(f, "headclip={}", v),
            DotAttr::Headhref(v) => write!(f, "headhref={}", v),
            DotAttr::Headlabel(v) => write!(f, "headlabel={}", v),
            DotAttr::Headport(v) => write!(f, "headport={}", v),
            DotAttr::Headtarget(v) => write!(f, "headtarget={}", v),
            DotAttr::Headtooltip(v) => write!(f, "headtooltip={}", v),
            DotAttr::HeadURL(v) => write!(f, "headURL={}", v),
            DotAttr::Height(v) => write!(f, "height={}", v),
            DotAttr::Href(v) => write!(f, "href={}", v),
            DotAttr::Id(v) => write!(f, "id={}", v),
            DotAttr::Image(v) => write!(f, "image={}", v),
            DotAttr::Imagepath(v) => write!(f, "imagepath={}", v),
            DotAttr::Imagepos(v) => write!(f, "imagepos={}", v),
            DotAttr::Imagescale(v) => write!(f, "imagescale={}", v),
            DotAttr::Inputscale(v) => write!(f, "inputscale={}", v),
            DotAttr::K(v) => write!(f, "K={}", v),
            DotAttr::Label(v) => write!(f, "label={}", v),
            DotAttr::Label_scheme(v) => write!(f, "label_scheme={}", v),
            DotAttr::Labelangle(v) => write!(f, "labelangle={}", v),
            DotAttr::Labeldistance(v) => write!(f, "labeldistance={}", v),
            DotAttr::Labelfloat(v) => write!(f, "labelfloat={}", v),
            DotAttr::Labelfontcolor(v) => write!(f, "labelfontcolor={}", v),
            DotAttr::Labelfontname(v) => write!(f, "labelfontname={}", v),
            DotAttr::Labelfontsize(v) => write!(f, "labelfontsize={}", v),
            DotAttr::Labelhref(v) => write!(f, "labelhref={}", v),
            DotAttr::Labeljust(v) => write!(f, "labeljust={}", v),
            DotAttr::Labelloc(v) => write!(f, "labelloc={}", v),
            DotAttr::Labeltarget(v) => write!(f, "labeltarget={}", v),
            DotAttr::Labeltooltip(v) => write!(f, "labeltooltip={}", v),
            DotAttr::LabelURL(v) => write!(f, "labelURL={}", v),
            DotAttr::Landscape(v) => write!(f, "landscape={}", v),
            DotAttr::Layer(v) => write!(f, "layer={}", v),
            DotAttr::Layerlistsep(v) => write!(f, "layerlistsep={}", v),
            DotAttr::Layers(v) => write!(f, "layers={}", v),
            DotAttr::Layerselect(v) => write!(f, "layerselect={}", v),
            DotAttr::Layersep(v) => write!(f, "layersep={}", v),
            DotAttr::Layout(v) => write!(f, "layout={}", v),
            DotAttr::Len(v) => write!(f, "len={}", v),
            DotAttr::Levels(v) => write!(f, "levels={}", v),
            DotAttr::Levelsgap(v) => write!(f, "levelsgap={}", v),
            DotAttr::Lhead(v) => write!(f, "lhead={}", v),
            DotAttr::Lheight(v) => write!(f, "lheight={}", v),
            DotAttr::Linelength(v) => write!(f, "linelength={}", v),
            DotAttr::Lp(v) => write!(f, "lp={}", v),
            DotAttr::Ltail(v) => write!(f, "ltail={}", v),
            DotAttr::Lwidth(v) => write!(f, "lwidth={}", v),
            DotAttr::Margin(v) => write!(f, "margin={}", v),
            DotAttr::Maxiter(v) => write!(f, "maxiter={}", v),
            DotAttr::Mclimit(v) => write!(f, "mclimit={}", v),
            DotAttr::Mindist(v) => write!(f, "mindist={}", v),
            DotAttr::Minlen(v) => write!(f, "minlen={}", v),
            DotAttr::Mode(v) => write!(f, "mode={}", v),
            DotAttr::Model(v) => write!(f, "model={}", v),
            DotAttr::Newrank(v) => write!(f, "newrank={}", v),
            DotAttr::Nodesep(v) => write!(f, "nodesep={}", v),
            DotAttr::Nojustify(v) => write!(f, "nojustify={}", v),
            DotAttr::Normalize(v) => write!(f, "normalize={}", v),
            DotAttr::Notranslate(v) => write!(f, "notranslate={}", v),
            DotAttr::Nslimit(v) => write!(f, "nslimit={}", v),
            DotAttr::Nslimit1(v) => write!(f, "nslimit1={}", v),
            DotAttr::Oneblock(v) => write!(f, "oneblock={}", v),
            DotAttr::Ordering(v) => write!(f, "ordering={}", v),
            DotAttr::Orientation(v) => write!(f, "orientation={}", v),
            DotAttr::Outputorder(v) => write!(f, "outputorder={}", v),
            DotAttr::Overlap(v) => write!(f, "overlap={}", v),
            DotAttr::Overlap_scaling(v) => write!(f, "overlap_scaling={}", v),
            DotAttr::Overlap_shrink(v) => write!(f, "overlap_shrink={}", v),
            DotAttr::Pack(v) => write!(f, "pack={}", v),
            DotAttr::Packmode(v) => write!(f, "packmode={}", v),
            DotAttr::Pad(v) => write!(f, "pad={}", v),
            DotAttr::Page(v) => write!(f, "page={}", v),
            DotAttr::Pagedir(v) => write!(f, "pagedir={}", v),
            DotAttr::Pencolor(v) => write!(f, "pencolor={}", v),
            DotAttr::Penwidth(v) => write!(f, "penwidth={}", v),
            DotAttr::Peripheries(v) => write!(f, "peripheries={}", v),
            DotAttr::Pin(v) => write!(f, "pin={}", v),
            DotAttr::Pos(v) => write!(f, "pos={}", v),
            DotAttr::Quadtree(v) => write!(f, "quadtree={}", v),
            DotAttr::Quantum(v) => write!(f, "quantum={}", v),
            DotAttr::Radius(v) => write!(f, "radius={}", v),
            DotAttr::Rank(v) => write!(f, "rank={}", v),
            DotAttr::Rankdir(v) => write!(f, "rankdir={}", v),
            DotAttr::Ranksep(v) => write!(f, "ranksep={}", v),
            DotAttr::Ratio(v) => write!(f, "ratio={}", v),
            DotAttr::Rects(v) => write!(f, "rects={}", v),
            DotAttr::Regular(v) => write!(f, "regular={}", v),
            DotAttr::Remincross(v) => write!(f, "remincross={}", v),
            DotAttr::Repulsiveforce(v) => write!(f, "repulsiveforce={}", v),
            DotAttr::Resolution(v) => write!(f, "resolution={}", v),
            DotAttr::Root(v) => write!(f, "root={}", v),
            DotAttr::Rotate(v) => write!(f, "rotate={}", v),
            DotAttr::Rotation(v) => write!(f, "rotation={}", v),
            DotAttr::Samehead(v) => write!(f, "samehead={}", v),
            DotAttr::Sametail(v) => write!(f, "sametail={}", v),
            DotAttr::Samplepoints(v) => write!(f, "samplepoints={}", v),
            DotAttr::Scale(v) => write!(f, "scale={}", v),
            DotAttr::Searchsize(v) => write!(f, "searchsize={}", v),
            DotAttr::Sep(v) => write!(f, "sep={}", v),
            DotAttr::Shape(v) => write!(f, "shape={}", v),
            DotAttr::Shapefile(v) => write!(f, "shapefile={}", v),
            DotAttr::Showboxes(v) => write!(f, "showboxes={}", v),
            DotAttr::Sides(v) => write!(f, "sides={}", v),
            DotAttr::Size(v) => write!(f, "size={}", v),
            DotAttr::Skew(v) => write!(f, "skew={}", v),
            DotAttr::Smoothing(v) => write!(f, "smoothing={}", v),
            DotAttr::Sortv(v) => write!(f, "sortv={}", v),
            DotAttr::Splines(v) => write!(f, "splines={}", v),
            DotAttr::Start(v) => write!(f, "start={}", v),
            DotAttr::Style(v) => write!(f, "style={}", v),
            DotAttr::Stylesheet(v) => write!(f, "stylesheet={}", v),
            DotAttr::Tail_lp(v) => write!(f, "tail_lp={}", v),
            DotAttr::Tailclip(v) => write!(f, "tailclip={}", v),
            DotAttr::Tailhref(v) => write!(f, "tailhref={}", v),
            DotAttr::Taillabel(v) => write!(f, "taillabel={}", v),
            DotAttr::Tailport(v) => write!(f, "tailport={}", v),
            DotAttr::Tailtarget(v) => write!(f, "tailtarget={}", v),
            DotAttr::Tailtooltip(v) => write!(f, "tailtooltip={}", v),
            DotAttr::TailURL(v) => write!(f, "tailURL={}", v),
            DotAttr::Target(v) => write!(f, "target={}", v),
            DotAttr::TBbalance(v) => write!(f, "TBbalance={}", v),
            DotAttr::Tooltip(v) => write!(f, "tooltip={}", v),
            DotAttr::Truecolor(v) => write!(f, "truecolor={}", v),
            DotAttr::Vertices(v) => write!(f, "vertices={}", v),
            DotAttr::Viewport(v) => write!(f, "viewport={}", v),
            DotAttr::Voro_margin(v) => write!(f, "voro_margin={}", v),
            DotAttr::Weight(v) => write!(f, "weight={}", v),
            DotAttr::Width(v) => write!(f, "width={}", v),
            DotAttr::Xdotversion(v) => write!(f, "xdotversion={}", v),
            DotAttr::Xlabel(v) => write!(f, "xlabel={}", v),
            DotAttr::Xlp(v) => write!(f, "xlp={}", v),
            DotAttr::Z(v) => write!(f, "z={}", v),
        }
    }
}

/// Helper function to parse boolean values
fn parse_bool(s: &str) -> Result<bool, String> {
    // Try parsing as integer first - any nonzero integer is true
    if let Ok(n) = s.parse::<i64>() {
        return Ok(n != 0);
    }

    // Fall back to text-based parsing
    match s.to_lowercase().as_str() {
        "true" | "yes" => Ok(true),
        "false" | "no" => Ok(false),
        _ => Err(format!("Invalid boolean value: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_types() {
        let attr = DotAttr::parse("fontsize", "12").unwrap();
        assert_eq!(attr.name(), "fontsize");
        assert_eq!(attr.to_string(), "fontsize=12");

        let attr = DotAttr::parse("label", "Hello").unwrap();
        assert_eq!(attr.name(), "label");
        assert_eq!(attr.to_string(), "label=Hello");
    }

    #[test]
    fn test_parse_bool() {
        let attr = DotAttr::parse("center", "true").unwrap();
        assert_eq!(attr.name(), "center");
        assert_eq!(attr.to_string(), "center=true");

        let attr = DotAttr::parse("center", "false").unwrap();
        assert_eq!(attr.to_string(), "center=false");
    }

    #[test]
    fn test_parse_color() {
        let attr = DotAttr::parse("color", "red").unwrap();
        assert_eq!(attr.name(), "color");

        let attr = DotAttr::parse("bgcolor", "#ff0000").unwrap();
        assert_eq!(attr.name(), "bgcolor");
    }

    #[test]
    fn test_parse_shape() {
        let attr = DotAttr::parse("shape", "box").unwrap();
        assert_eq!(attr.name(), "shape");
        assert!(attr.to_string().contains("box"));
    }

    #[test]
    fn test_parse_arrow_type() {
        let attr = DotAttr::parse("arrowhead", "diamond").unwrap();
        assert_eq!(attr.name(), "arrowhead");
    }

    #[test]
    fn test_parse_dir_type() {
        let attr = DotAttr::parse("dir", "forward").unwrap();
        assert_eq!(attr.name(), "dir");
    }

    #[test]
    fn test_parse_unknown_attribute() {
        let result = DotAttr::parse("unknown_attr", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_value() {
        let result = DotAttr::parse("fontsize", "not_a_number");
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
            let attr = DotAttr::parse(name, value).unwrap();
            let formatted = attr.to_string();
            assert!(formatted.starts_with(name));
            assert!(formatted.contains(value) || formatted.contains(&value.replace(".", ".")));
        }
    }
}
