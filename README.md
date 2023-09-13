# About

This crate defines a `Linear<T>` type. This is a type whose values can not be dropped and must
be eventually consumed with the `into_inner()` method. Failing to do so will result in a panic
or compile error.

Unlike `Pin` linear types can be moved and unlike `ManuallyDrop` linear types require to be
eventually deconstructed unless they are forgotten with `mem::forget()`. 

## Features

When this crates is compiled with the 'compile_error' feature flag then it will use the
'no-panic' crate to generate compile errors whenever `Linear<T>` will be dropped.

Please read [https://github.com/dtolnay/no-panic#caveats][1] for details.


[1]: https://github.com/dtolnay/no-panic#caveats
