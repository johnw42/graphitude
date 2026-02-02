//! Example demonstrating typed DOT attributes.

#[cfg(feature = "dot")]
fn main() {
    use graphitude::dot::attr::Attr;

    println!("=== Graphviz DOT Attributes with Typed Payloads ===\n");

    // Parse various attribute types
    println!("1. Numeric attributes:");
    let fontsize = Attr::parse("fontsize", "14.5").unwrap();
    println!("   {} -> {}", fontsize.name(), fontsize);

    let width = Attr::parse("width", "2.0").unwrap();
    println!("   {} -> {}", width.name(), width);

    println!("\n2. Boolean attributes:");
    let center = Attr::parse("center", "true").unwrap();
    println!("   {} -> {}", center.name(), center);

    let constraint = Attr::parse("constraint", "false").unwrap();
    println!("   {} -> {}", constraint.name(), constraint);

    println!("\n3. String attributes:");
    let label = Attr::parse("label", "My Label").unwrap();
    println!("   {} -> {}", label.name(), label);

    let comment = Attr::parse("comment", "A comment").unwrap();
    println!("   {} -> {}", comment.name(), comment);

    println!("\n4. Color attributes:");
    let color = Attr::parse("color", "red").unwrap();
    println!("   {} -> {}", color.name(), color);

    let bgcolor = Attr::parse("bgcolor", "#ff00ff").unwrap();
    println!("   {} -> {}", bgcolor.name(), bgcolor);

    println!("\n5. Shape attributes:");
    let shape = Attr::parse("shape", "box").unwrap();
    println!("   {} -> {}", shape.name(), shape);

    let shape2 = Attr::parse("shape", "ellipse").unwrap();
    println!("   {} -> {}", shape2.name(), shape2);

    println!("\n6. Arrow type attributes:");
    let arrowhead = Attr::parse("arrowhead", "diamond").unwrap();
    println!("   {} -> {}", arrowhead.name(), arrowhead);

    let arrowtail = Attr::parse("arrowtail", "dot").unwrap();
    println!("   {} -> {}", arrowtail.name(), arrowtail);

    println!("\n7. Direction attributes:");
    let dir = Attr::parse("dir", "forward").unwrap();
    println!("   {} -> {}", dir.name(), dir);

    let rankdir = Attr::parse("rankdir", "LR").unwrap();
    println!("   {} -> {}", rankdir.name(), rankdir);

    println!("\n8. Style attributes:");
    let style = Attr::parse("style", "filled,bold").unwrap();
    println!("   {} -> {}", style.name(), style);

    println!("\n9. Error handling:");
    match Attr::parse("fontsize", "not_a_number") {
        Ok(_) => println!("   Unexpected success!"),
        Err(e) => println!("   Parse error: {}", e),
    }

    match Attr::parse("unknown_attr", "value") {
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
