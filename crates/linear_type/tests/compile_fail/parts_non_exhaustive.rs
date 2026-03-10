use linear_type::parts;

struct Abc {
    a: String,
}

parts! {
    #[non_exhaustive]
    impl Abc {
        a: String,
    }
}

fn main() {}
