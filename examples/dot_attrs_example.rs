//! Example demonstrating typed DOT attributes.

#[cfg(feature = "dot")]
fn main() {
    use graphitude::DotAttr;

    println!("=== Graphviz DOT Attributes with Typed Payloads ===\n");

    // Parse various attribute types
    println!("1. Numeric attributes:");
    let fontsize = DotAttr::parse("fontsize", "14.5").unwrap();
    println!("   {} -> {}", fontsize.name(), fontsize);

    let width = DotAttr::parse("width", "2.0").unwrap();
    println!("   {} -> {}", width.name(), width);

    println!("\n2. Boolean attributes:");
    let center = DotAttr::parse("center", "true").unwrap();
    println!("   {} -> {}", center.name(), center);

    let constraint = DotAttr::parse("constraint", "false").unwrap();
    println!("   {} -> {}", constraint.name(), constraint);

    println!("\n3. String attributes:");
    let label = DotAttr::parse("label", "My Label").unwrap();
    println!("   {} -> {}", label.name(), label);

    let comment = DotAttr::parse("comment", "A comment").unwrap();
    println!("   {} -> {}", comment.name(), comment);

    println!("\n4. Color attributes:");
    let color = DotAttr::parse("color", "red").unwrap();
    println!("   {} -> {}", color.name(), color);

    let bgcolor = DotAttr::parse("bgcolor", "#ff00ff").unwrap();
    println!("   {} -> {}", bgcolor.name(), bgcolor);

    println!("\n5. Shape attributes:");
    let shape = DotAttr::parse("shape", "box").unwrap();
    println!("   {} -> {}", shape.name(), shape);

    let shape2 = DotAttr::parse("shape", "ellipse").unwrap();
    println!("   {} -> {}", shape2.name(), shape2);

    println!("\n6. Arrow type attributes:");
    let arrowhead = DotAttr::parse("arrowhead", "diamond").unwrap();
    println!("   {} -> {}", arrowhead.name(), arrowhead);

    let arrowtail = DotAttr::parse("arrowtail", "dot").unwrap();
    println!("   {} -> {}", arrowtail.name(), arrowtail);

    println!("\n7. Direction attributes:");
    let dir = DotAttr::parse("dir", "forward").unwrap();
    println!("   {} -> {}", dir.name(), dir);

    let rankdir = DotAttr::parse("rankdir", "LR").unwrap();
    println!("   {} -> {}", rankdir.name(), rankdir);

    println!("\n8. Style attributes:");
    let style = DotAttr::parse("style", "filled,bold").unwrap();
    println!("   {} -> {}", style.name(), style);

    println!("\n9. Error handling:");
    match DotAttr::parse("fontsize", "not_a_number") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Parse error: {}", e),
    }

    match DotAttr::parse("unknown_attr", "value") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Unknown attribute: {}", e),
    }

    println!("\n=== Type Safety Benefits ===");
    println!("Each attribute now has a typed payload:");
    println!("  - Numbers are f64 or i32");
    println!("  - Booleans are bool");
    println!("  - Colors are Color enum (Named, RGB, RGBA, HSV, Scheme)");
    println!("  - Shapes are Shape enum with 25+ variants");
    println!("  - Arrows are ArrowType enum with 15+ variants");
    println!("  - etc.");
}

#[cfg(not(feature = "dot"))]
fn main() {
    println!("This example requires the 'dot' feature to be enabled.");
    println!("Run with: cargo run --example dot_attrs_example --features dot");
}
