# Important

Fork of `linear_type` and all credit goes to author of it.

Not sure I ready to use radicle this time. Hope author will examine changes and incorporate some.

# About

This crate strives to implement [linear
types](https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems).

Combines `#[must_use]` attribute with runtime `panic!` in type level. 

The [`Linear<T>`] type that wraps a `T`. Linear types must not be dropped but eventualy consumed.
There are only a few methods you can use on a linear type.

* `new()` creates a new linear type.
* `into()` destructures a object returning the inner value as non linear type.
* `destroy()` manual drop method, consumes and destroys the wrapped `T`.
* `map()` applies a `FnOnce` with the destructured inner type as parameter yielding another
  linear type.
* Some variants of `map()` to handle `Linear<Result<T,E>>` and `Linear<Option<T>>`.
* `Linear<Result<T,E>>` and `Linear<Option<T>>` support few forms of `unwrap()`.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed and consumed.

## linear! macro

```rust
use linear_ty::linear;

linear!(pub struct LinearResult(String););
```

This generates the full linear wrapper implementation, including `map`, `into`, and the
`Result`/`Option` extensions.

`linear!` also rejects `#[non_exhaustive]` targets:

```compile_fail
use linear_ty::linear;

linear! {
  #[non_exhaustive]
  pub struct LinearResult(String);
}
```


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
use linear_ty::*;
use std::fs::File;
use std::io::{Read, Result};
use std::path::PathBuf;

// define some newtypes for the states
struct Filename(PathBuf);
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
    let file_content = new_linear!(Filename(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../README.md"),
    ))
        .map(open_file)
        .map_ok(read_text)
        .unwrap_ok();

    // destructure the file content
    let FileContent(text) = file_content.into();
    assert!(text.contains("# Example"));
}
```

### Parts macro

Consider struct 

```rust
struct Abc {
  a: String,
  b: u8,
  c: Vec<()>
}
```

On caller side to ensure all fields used one can do:

```rust
struct Abc {
  a: String,
  b: u8,
  c: Vec<()>
}

let abc = Abc {
  a: "hi".to_string(),
  b: 7,
  c: vec![()],
};
let Abc { a, b, c} = abc;
```


If struct market non_exaustive, or caller use `..` or `a: _`, easy to miss return.

Also field setters could be private.

so `parts!` macro allows to access all fileds like:

```rust
use linear_ty::parts;

struct Abc {
  a: String,
  b: u8,
  c: Vec<()>
}

parts! {
  impl Abc {
    a: String,
    b: u8,
    c: Vec<()>,
  }
}

let abc = Abc {
  a: "hi".to_string(),
  b: 7,
  c: vec![()],
};

let (a,b,c) = abc.parts_ref();
let (a,b,c) = abc.parts();
```

Both methods of parts and parts_ref marcked as must_use.

`parts!` also rejects `#[non_exhaustive]` targets:

```compile_fail
use linear_ty::parts;

struct Abc {
  a: String,
}

parts! {
  #[non_exhaustive]
  impl Abc {
    a: String,
  }
}
```
