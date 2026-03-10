use linear_type::{linear, unique};

linear! {
    /// Custom linear wrapper that holds a `String`.
    pub struct ReturnResponseMustUse<T, U>(T);
}

fn main() {
    let foo = ReturnResponseMustUse::new(String::from("a"), unique!());
    let mut bar = ReturnResponseMustUse::new(String::from("b"), unique!());

    // This must not compile because `foo` and `bar` have distinct unique types.
    bar = foo;
}
