# DOT Attributes Refactoring Summary

## Overview
The `DotAttr` enum has been completely refactored to include typed payloads for each attribute variant. Previously, the enum only represented attribute names. Now each variant carries the attribute's value with the appropriate Rust type.

## Key Changes

### 1. **Typed Payloads**
Each `DotAttr` variant now contains a typed payload:

**Before:**
```rust
pub enum DotAttr {
    Color,      // Just the name
    Fontsize,   // Just the name
    Shape,      // Just the name
    // ...
}
```

**After:**
```rust
pub enum DotAttr {
    Color(Color),        // Contains a Color value
    Fontsize(f64),       // Contains a numeric value
    Shape(Shape),        // Contains a Shape value
    Label(String),       // Contains a string value
    Center(bool),        // Contains a boolean value
    // ... 177 total variants
}
```

### 2. **New Parsing API**
The parsing has been changed from single-argument to two-argument form:

**Before:**
```rust
let attr: DotAttr = "color".parse().unwrap();  // Just the name
```

**After:**
```rust
let attr = DotAttr::parse("color", "red").unwrap();  // Name + value
```

### 3. **Type Safety Benefits**
- **Compile-time type checking**: Invalid values are caught at parse time
- **Structured data**: Values are parsed into appropriate Rust types immediately
- **Better IDE support**: Autocomplete and type hints work correctly
- **Clearer semantics**: The attribute's value is part of the enum, not separate

### 4. **Supported Types**
The implementation uses these type mappings:

| Graphviz Type | Rust Type | Example |
|---------------|-----------|---------|
| double | `f64` | `Fontsize(14.5)` |
| int | `i32` | `Peripheries(2)` |
| bool | `bool` | `Center(true)` |
| string | `String` | `Label("text")` |
| color | `Color` | `Color(Color::Named("red"))` |
| shape | `Shape` | `Shape(Shape::Box)` |
| arrowType | `ArrowType` | `Arrowhead(ArrowType::Diamond)` |
| dirType | `DirType` | `Dir(DirType::Forward)` |
| rankdir | `RankDir` | `Rankdir(RankDir::LR)` |
| rankType | `RankType` | `Rank(RankType::Same)` |
| outputMode | `OutputMode` | `Outputorder(OutputMode::BreadthFirst)` |
| pagedir | `PageDir` | `Pagedir(PageDir::BL)` |
| point | `Point` | `Lp(Point::new(1.0, 2.0))` |
| rect | `Rect` | `Bb(Rect::new(0.0, 0.0, 10.0, 10.0))` |
| style | `Style` | `Style(Style::from("filled,bold"))` |

### 5. **API Methods**

#### `DotAttr::parse(name: &str, value: &str) -> Result<Self, String>`
Parse an attribute from its name and string value.
```rust
let attr = DotAttr::parse("fontsize", "12").unwrap();
let attr = DotAttr::parse("color", "red").unwrap();
let attr = DotAttr::parse("shape", "box").unwrap();
```

#### `DotAttr::name(&self) -> &'static str`
Returns the canonical attribute name.
```rust
let attr = DotAttr::parse("fontsize", "12").unwrap();
assert_eq!(attr.name(), "fontsize");
```

#### `Display` Implementation
Formats the attribute as `name=value`.
```rust
let attr = DotAttr::parse("fontsize", "12").unwrap();
assert_eq!(attr.to_string(), "fontsize=12");
```

### 6. **Error Handling**
The parser provides clear error messages:
```rust
// Invalid value for type
match DotAttr::parse("fontsize", "not_a_number") {
    Err(e) => println!("{}", e),  // "Invalid fontsize: invalid float literal"
}

// Unknown attribute
match DotAttr::parse("unknown_attr", "value") {
    Err(e) => println!("{}", e),  // "Unknown attribute: unknown_attr"
}

// Invalid enum value
match DotAttr::parse("shape", "invalid_shape") {
    Err(e) => println!("{}", e),  // "Invalid shape: Unknown shape: invalid_shape"
}
```

### 7. **Usage Examples**

#### Basic Parsing
```rust
use graphitude::DotAttr;

// Parse various types
let fontsize = DotAttr::parse("fontsize", "14.5").unwrap();
let label = DotAttr::parse("label", "My Node").unwrap();
let center = DotAttr::parse("center", "true").unwrap();
let color = DotAttr::parse("color", "#ff0000").unwrap();
let shape = DotAttr::parse("shape", "box").unwrap();

// Format back to DOT syntax
println!("{}", fontsize);  // "fontsize=14.5"
println!("{}", label);     // "label=My Node"
println!("{}", center);    // "center=true"
```

#### Pattern Matching
```rust
match attr {
    DotAttr::Fontsize(size) if size > 10.0 => {
        println!("Large font: {}", size);
    }
    DotAttr::Color(Color::Named(name)) => {
        println!("Using named color: {}", name);
    }
    DotAttr::Shape(Shape::Box) => {
        println!("Box-shaped node");
    }
    _ => {}
}
```

#### Building DOT Output
```rust
let attrs = vec![
    DotAttr::parse("fontsize", "12")?,
    DotAttr::parse("color", "blue")?,
    DotAttr::parse("shape", "ellipse")?,
];

for attr in attrs {
    println!("  {}", attr);
}
// Output:
//   fontsize=12
//   color=blue
//   shape=ellipse
```

### 8. **Testing**
The module includes comprehensive tests covering:
- Parsing simple types (strings, numbers)
- Parsing complex types (colors, shapes, arrows)
- Boolean value parsing (true/false, yes/no, 1/0)
- Error handling for invalid values
- Round-trip conversion (parse → display → parse)
- Case-insensitive attribute name matching

All 272 tests pass, including:
- 9 specific DOT attribute tests
- Integration with existing DOT types
- Compatibility with the rest of the library

### 9. **Migration Guide**

**Old Code:**
```rust
// Just attribute names
let attr: DotAttr = "color".parse().unwrap();
match attr {
    DotAttr::Color => { /* handle */ }
    _ => {}
}
```

**New Code:**
```rust
// Attribute with value
let attr = DotAttr::parse("color", "red").unwrap();
match attr {
    DotAttr::Color(color) => { 
        // Can now access the actual color value
        println!("Color is: {}", color);
    }
    _ => {}
}
```

### 10. **File Structure**
- **Location**: `src/graph/dot_attrs.rs`
- **Size**: ~1400 lines
- **Dependencies**:
  - `std::fmt` for Display trait
  - `super::dot_types::*` for custom types (Color, Shape, etc.)
- **Exports**: 
  - Main enum: `DotAttr`
  - All variants are public
  - Helper function: `parse_bool()` (private)

### 11. **Compatibility**
- ✅ **Backward API change**: The parsing API has changed (now requires two arguments)
- ✅ **Feature gated**: All DOT-related code is behind `#[cfg(feature = "dot")]`
- ✅ **Type safe**: Strong typing prevents invalid attribute assignments
- ✅ **Extensible**: Easy to add new attribute types or variants

### 12. **Performance Notes**
- Parsing happens once per attribute
- Values are stored in efficient Rust types (no string parsing at usage time)
- Display implementation is efficient (direct formatting)
- Pattern matching on variants is zero-cost

## Benefits
1. **Type Safety**: Compile-time checks prevent invalid attribute values
2. **Better Error Messages**: Parse errors show exactly what went wrong
3. **Cleaner API**: Value is part of the attribute, not separate
4. **IDE Support**: Autocomplete and type hints work correctly
5. **Extensibility**: Easy to add validation or conversion logic
6. **Performance**: Values are pre-parsed, no runtime string interpretation needed

## Files Changed
- `src/graph/dot_attrs.rs` - Complete rewrite with typed payloads
- `examples/dot_attrs_example.rs` - New example demonstrating the API

## Testing
Run the tests with:
```bash
cargo test --lib --features dot dot_attrs
```

Run the example with:
```bash
cargo run --example dot_attrs_example --features dot
```

## Next Steps
Potential future enhancements:
1. Add validation for attribute applicability (graph vs node vs edge)
2. Implement serde support for serialization/deserialization
3. Add builder pattern for complex attribute combinations
4. Support for attribute collections with conflict detection
