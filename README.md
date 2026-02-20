This crate contains a proc macro for writing a suite of unit tests as methods of an object which can be instantiated via a new macro to run each method as an individual test.

# Example

Consider this test suite in `src/mytrait_tests.rs`:

```rust
pub struct TestSuite<T> {
  // Data needed for each test in the suite.
  param1: usize,
  param2: String,
}

#[generate_test_macro(mytrait_tests)]
impl<T: MyTrait> TestSuite<T> {
  fn new(param1: usize, param2: String) -> Self {
    Self { param1, param2 }
  }

  #[test]
  fn my_test(&self) {
    todo!("run a test");
  }
}
```

The expansion of the impl looks like this:

```rust
impl<T: MyTrait> TestSuite<T> {
  #[doc(hidden)]
  pub fn new(param1: usize, param2: String) -> Self {
    Self { param1, param2 }
  }

  #[doc(hidden)]
  pub fn my_test(&self) {
    todo!("run a test");
  }
}

#[macro_export]
macro_rules! mytrait_tests {
  (mod_name:$ident, $T:ty, $param1:expr, $param2:expr) => {
    mod $mod_name {
      use super::*;

      #[test]
      fn my_test() {
        $crate::mytrait_tests::TestSuite::<$T>::new($param1, $param2).my_test();
      }
    }
  }
}
```

If the "quickcheck" feature is enabled, quickcheck tests are also supported.  Consider this implementation of `TestSuite<T>`:

```rust
#[generate_test_macro(mytrait_tests)]
impl<T: MyTrait> TestSuite<T> {
  #[quickcheck]
  fn test_result_prop(data: MyTestData<T>) -> TestResult {
    todo!("generate a TestResult")
  }

  #[quickcheck]
  fn boolean_prop(data: MyTestData<T>) -> bool {
    todo!("generate a boolean result")
  }
}
```

The expansion looks like this:

```rust
impl<T: MyTrait> TestSuite<T> {
  #[doc(hidden)]
  pub fn test_result_prop(data: MyTestData<T>) -> TestResult {
    todo!("generate a TestResult")
  }

  #[doc(hidden)]
  pub fn boolean_prop(data: MyTestData<T>) -> bool {
    todo!("generate a boolean result")
  }
}

#[macro_export]
macro_rules! mytrait_tests {
  (mod_name:$ident, $T:ty) => {
    mod $mod_name {
      use super::*;

      #[test]
      pub fn test_result_prop() {
        quickcheck::quickcheck($crate::mytrait_tests::TestSuite::<$T>::test_result_prop as fn(_) -> _);
      }


      #[test]
      pub fn boolean_prop() {
        quickcheck::quickcheck($crate::mytrait_tests::TestSuite::<$T>::boolean_prop as fn(_) -> _);
      }
    }
  }
}
```

Other than a `new` method and methods annotated with `#[test]` or `#[quickcheck]`, all methods are copied as-is to the result.