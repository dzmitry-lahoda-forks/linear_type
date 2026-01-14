# About

This crate strives to implement unique [linear
types](https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems).

The [`Linear<T>`] type that wraps a `T`. Linear types must not be dropped but eventualy consumed.
There are only a few methods you can use on a linear type.

* `new_linear!()` creates a new linear type. Must be a macro for unique types.
* `into_inner()` destructures a object returning the inner `T`.
* `destroy()` manual drop method, consumes and destroys the wrapped inner type.
* `map()` applies a `FnOnce` with the destructured inner type as parameter yielding another
  linear type.
* Some variants of `map()` to handle `Linear<Result<T,E>>` and `Linear<Option<T>>`.
* `Linear<Result<T,E>>` and `Linear<Option<T>>` support few forms of `unwrap()`.
* `splice()` and `merge()` to divert and join the program flow of unique types.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed and consumed.

Strict linearity is enforced by creating unique types for each linear type which track the
operations that are applied on the values. They are tagged `#[must_use]`, when one still tries
to drop a linear type by `let _ = ...` or `drop()` a runtime panic happens. Unfortunally there
is no way to detect this at compile time in stable rust.


### Dropping Policy and Panics

Panicking is inherently incompatible with a linear type system. Still rust relies on the fact
that panics can basically happen at any time. We face the situation that we have to mix
standard rust with a linear type system.

Linear types are ordinary rust types. When panics happen then the all state is properly
unwinded and destructors are called, even for the inners of linear types. This is only
modestly sound but the best we can do.


## Status

This crate started with a discussion on IRC. It does not completey implement pure linear-type
theory as this would require language support. Consider it as proof-of-concept. Most notably
using `mem::forget()` on a linear type will break the linear type semantics. This crate can
not prevent this. It may have some use and should be safe (in the Rust sense) to
use. Improvements and PR's are welcome. This crate will be somewhat in flux before a 1.0
version is released.

Creating unique types for each new linear type and each operation on it is expected to be
heavy on the type system. Don't expect it to be fast and slim to compile. If possible enable
LTO in release builds since it may reduce the footprint of monomorphized functions
significantly.


## Feature Flags

* **`drop_unchecked`**

  When this crate is compiled with the `drop_unchecked` feature flag, then, in release builds,
  dropping a linear type will not panic as intended. The linear-type semantic is not
  enforced. This defeats the purpose of this crate. It adds only a small space and performance
  improvement. It should considered to be UB and should only be enabled on programs that are
  thoroughly validated and tested when required.

* **`semipure`**

  When this crate is compiled with the `semipure` feature flag, then the `Linear<T>` type will
  implement the `get_ref()` method. This method will return a reference to the inner
  value. This is useful when you want to borrow the inner value without consuming the linear
  type. This feature is not recommended as it may break the linear type semantics.

# Example

While any type can be wraped in a `Linear<T>`, it is recommended to use it with unique newtypes
which transitioning into a final state. The state transitions can be functions or closures.

```rust
use linear_type::*;
use std::fs::File;
use std::io::{Read, Result};

// define some newtypes for the states
struct Filename(&'static str);
#[derive(Debug)]
struct ReadonlyFile(File);
#[derive(Debug)]
struct FileContent(String);

// define functions that transition from one state to the next.
fn open_file(Filename(name): Filename) -> Result<ReadonlyFile> {
    Ok(ReadonlyFile(File::open(name)?))
}

fn read_text(ReadonlyFile(mut file): ReadonlyFile) -> Result<FileContent> {
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(FileContent(text))
}

fn main() {
    // Create a linear type and transition through the states
    let file_content = new_linear!(Filename("README.md"))
        .map(open_file)
        .map_ok(read_text)
        .unwrap_ok();

    // destructure the file content
    let FileContent(text) = file_content.into_inner();
    assert!(text.contains("# Example"));
}
```


