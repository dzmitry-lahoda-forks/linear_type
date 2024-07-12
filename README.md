# About

This crate defines a `Linear<T>` type. This is a type whose values cannot be dropped and must
be eventually consumed with the `into_inner()` method. Failing to do so will result in a panic
or compile error.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed unless they are forgotten with `mem::forget()`. 


## Status

This crate started with a discussion on IRC. It does not implement pure linear-type theory as
this would require language support. Consider it as proof-of-concept. It may have some use and
should be safe (in the Rust sense) to use. Improvements and PR's are welcome. This crate will
be somewhat in flux before a 1.0 version is released.


## Features

When this crate is compiled with the 'compile_error' feature flag, it will use the
'no-panic' crate to generate compile errors whenever `Linear<T>` will be dropped.

Please read [https://github.com/dtolnay/no-panic#caveats][1] for details.


[1]: https://github.com/dtolnay/no-panic#caveats
