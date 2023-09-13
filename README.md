This crate defines a `Linear<T>` type. This is a type which contents must be eventually
consumed with the 'into_inner()' method. Failing to do so and dropping a non-consumed
`Linear<T>` will panic. Future versions may implement dropping a non consumed linear type as
compile error.
