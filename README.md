# About

This crate strives to implement [linear
types](https://en.wikipedia.org/wiki/Substructural_type_system#Linear_type_systems).

* The [`Linear<T>`] type that wraps a `T`. Linear types must not be dropped but eventualy consumed.
  There are only 3 methods you can use on a linear type:
  1. [`Linear::new()`] creates a new linear type. Note the
  2. [`Linear::into_inner()`] destructures a object returning the inner value as non
     linear type.
  3. [`Linear::map()`] applies a `FnOnce` with the destructured inner type as parameter
     yielding another linear type.

Unlike `Pin`, linear types can be moved, and unlike `ManuallyDrop`, linear types are required to be
eventually deconstructed and consumed.


## Status

This crate started with a discussion on IRC. It does not implement pure linear-type theory as
this would require language support. Consider it as proof-of-concept. It may have some use and
should be safe (in the Rust sense) to use. Improvements and PR's are welcome. This crate will
be somewhat in flux before a 1.0 version is released.


## Feature Flags

* **`drop_unchecked`**  

  When this crate is compiled with the `drop_unchecked` feature flag, then, in release builds,
  dropping a linear type will not panic as intended. The linear-type semantic is not
  enforced. This defeats the purpose of this crate and adds only a small space and performance
  improvement. It should only be enabled on programs that are thoroughly validated and tested
  when required.


# Example

While any type can be wraped in a `Linear<T>`, it is recommended to use it with unique newtypes
which transitioning into a final state. The state trasitions can be functions or closures.

```rust
use linear_type::Linear;
use std::fs::File;
use std::io::Read;

// define some newtypes for the states
struct Filename(String);
struct ReadonlyFile(File);
struct FileContent(String);

// define functions that transitions from one state to the next
fn open_file(Filename(name): Filename) -> ReadonlyFile {
    ReadonlyFile(File::open(name).unwrap())
}

fn read_text(ReadonlyFile(mut file): ReadonlyFile) -> FileContent {
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    FileContent(text)
}

// Create a linear type and transition through the states
let file_content = Linear::new(Filename("README.md".to_string()))
    .map(open_file)
    .map(read_text);

// destructure the file content
let FileContent(text) = file_content.into_inner();
assert!(text.contains("# Example"));
```

