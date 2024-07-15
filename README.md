# About

This crate defines a `Linear<T>` type. This is a type whose values can not be dropped and must
be eventually consumed with the `into_inner()` method. Failing to do so will result in a panic
or compile error.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed unless they are forgotten with `mem::forget()`. 


## Status

This crate started with a discussion on IRC. It does not implement pure linear-type theory as
this would require language support. Consider it as proof-of-concept. It may have some use and
should be safe (in the Rust sense) to use. Improvements and PR's are welcome. This crate will
be somewhat in flux before a 1.0 version is released.


## Feature Flags

* **`drop_unchecked`**  

  When this crate is compiled with the `drop_unchecked` feature flag, then, in release builds,
  the contained value is simply wrapped in a `ManuallyDrop` and the check if a value is
  consumed with `.into_inner()` is omitted. While this is still safe, the linear-type semantic
  is not enforced and programs may leak resources for objects that are not properly consumed
  with `.into_inner()`. This defeats the purpose of this crate and adds only a small space and
  performance improvement. It should only be enabled on programs that are thoroughly validated
  and tested when required.
