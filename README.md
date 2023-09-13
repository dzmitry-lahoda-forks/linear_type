# About

This crate defines a `Linear<T>` type. This is a type which contents must be eventually
consumed with the `into_inner()` method. Failing to do so and dropping a non-consumed
`Linear<T>` will result in a panic or compile error.

# Features

When this crates is compile with '--feature compile_error' then it will use the 'no-panic'
crate to generate compile errors whenever a non-consumed `Linear<T>` will be dropped.

Please read https://github.com/dtolnay/no-panic#caveats for details.
