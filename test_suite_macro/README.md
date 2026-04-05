# test_suite_macro

A tool for testing implementations of a trait.

The normal way to test implementations of a trait is to write the tests in a
giant macro, which is then expanded for each implemention. This process is messy
and interferes with IDE tools, so this crate provides an alterantive where test
suites are written as ordinary structs with `#[test]` methods. A macro is then
provided to generate an ordinary `#[test]` function for each `#[test]` method.
This macro can be expanded multiple times to generate packages of test functions
for different instantiations of the test suite type.

## Basic Usage

To define a type as a test suite, use the `test_suite_macro` on its `impl`
block, passing the name of a new macro to be generated.  The `impl` block should
contain methods annotated with `#[test]`:

```rust
struct ExampleSuite<T> {...}

#[test_suite_macro(example_suite)]
impl ExampleSuite {
  #[test]
  pub fn instance_method_test(&self) {...}
}
```

This generates a new macro which generates a package containing a `#[test]`
function for each `#[test]` method of the type.  The calling convention of the
macro is

```
example_suite($package_name: $test_type = $test_instance);
```

(See below for abbreivated froms of this signature.)

For example, invoking the macro like this

```rust
example_suite(test1: ExampleSuite<i32> = ExampleSuite {...});

```

will expand (roughly) to this module definition:

```rust
mod test1 {
  use super::*;

  #[test]
  fn instance_method_test() {
    let instance: ExampleSuite<i32> = ExampleSuite {...};
    instance.instance_method_test();
  }
}
```

## Advanced Usage

### QuickCheck

In addition to methods marked as `#[test]`, static methods can be marked with
`#[quickcheck]`.  This operates like the `quickcheck` macro provided by the
`quickcheck_macros` crate. Due to limitations of QuickCheck, a method
annotated with `#[quickcheck]` cannot accept a `self` parameter.

### The `self` Parameter

The `self` parameter of test methods may be passed by value, reference, or mut
reference.  The distinction is largely irrelevant, because a new instance of the
test suite type is created for each test.

### Static Methods

Test methods may be static. If all test methods are static, there is no need to
pass a `$test_instance` parameter to the generated macro.  Instances of the test
suite type are not created for static test methods.

### Abbreviated Signatures

The `$test_type` parameter may be omitted when it is obvious from the
`$test_instance` expression.  The type is considered obvious when the expression
starts with the unqualified name of the test type, possibly followed by type
arguments using turbofish syntax.

The `$test_instance` paremeter may be omitted if the test suite type implments `Default`.