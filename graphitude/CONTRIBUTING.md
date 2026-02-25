## Style
- Prefer importing modules from this crate using the highest-level package where the import is available.
- Avoid using fully qualified names, except in macros, or where a symbol is only used once in a file.
- Avoid prefixing names with _ except where needed to silence compiler warnings. For default method implementations, prefer using `let _ =` syntax instead.
- Prefer `where` syntax for bounds in traits and trait impls.