#![allow(missing_docs)]

#[test]
fn ui_compile_fail() {
    let rustflags = match std::env::var("RUSTFLAGS") {
        Ok(existing) if existing.is_empty() => "-D warnings".to_string(),
        Ok(existing) => format!("{existing} -D warnings"),
        Err(_) => "-D warnings".to_string(),
    };
    unsafe { std::env::set_var("RUSTFLAGS", rustflags); }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
