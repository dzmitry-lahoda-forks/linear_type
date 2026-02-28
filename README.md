# Important

Fork of https://seed.pipapo.org/nodes/seed.pipapo.org/rad:z2HSXU1YYpbzTLp4LWXXL9j19rSnw and all credit goes to author of it.

Not sure I ready to use radicle this time. Hope author will examine changes and incorporate some.

# About

This crate strives to implement [linear
types](https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems).

The [`Linear<T>`] type that wraps a `T`. Linear types must not be dropped but eventualy consumed.
There are only a few methods you can use on a linear type.

* `new()` creates a new linear type.
* `into_inner()` destructures a object returning the inner value as non linear type.
* `destroy()` manual drop method, consumes and destroys the wrapped `T`.
* `map()` applies a `FnOnce` with the destructured inner type as parameter yielding another
  linear type.
* Some variants of `map()` to handle `Linear<Result<T,E>>` and `Linear<Option<T>>`.
* `Linear<Result<T,E>>` and `Linear<Option<T>>` support few forms of `unwrap()`.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed and consumed.


## Status

This crate started with a discussion on IRC. It does not implement pure linear-type theory as
this would require language support. Consider it as proof-of-concept. Most notably using
`mem::forget()` on a linear type will break the linear type semantics. This crate can not
prevent this. It may have some use and should be safe (in the Rust sense) to use. Improvements
and PR's are welcome. This crate will be somewhat in flux before a 1.0 version is released.


## Feature Flags

* **`drop_unchecked`**

  When this crate is compiled with the `drop_unchecked` feature flag, then, in release builds,
  dropping a linear type will not panic as intended. The linear-type semantic is not
  enforced. This defeats the purpose of this crate. it adds only a small space and performance
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


