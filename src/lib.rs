#![doc = include_str!("../README.md")]

/// A linear type that must be destructured to access the inner value.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct Linear<T>(T, NoDrop);

impl<T> Linear<T> {
    /// Creates a new linear type.
    pub const fn new(inner: T) -> Self {
        Self(inner, NoDrop)
    }

    /// Destructures the linear type and returns the inner type.  This must eventually be called on
    /// any linear type, failing to do so will panic.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let linear = Linear::new(123);
    /// let inner = linear.into_inner();
    /// assert_eq!(inner, 123);
    /// ```
    pub fn into_inner(self) -> T {
        let Linear(t, n) = self;
        std::mem::forget(n);
        t
    }

    /// Transforms one linear type to another linear type. The inner value is passed to the
    /// closure and the return value is wrapped in a `Linear`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let number = Linear::new(123);
    /// let string = number.map(|x| x.to_string());
    /// assert_eq!(string.into_inner(), "123");
    /// ```
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Linear<R> {
        Linear::new(f(self.into_inner()))
    }
}

/// A marker type that can not be dropped.
///
/// # Panics
///
/// When the `drop_unchecked` feature is not enabled, this type will panic when dropped.
/// This is to ensure that linear types are not dropped and must be destructured manually.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
struct NoDrop;

/// Drop is only implemented when either `debug_assertions` are enabled or the
/// `drop_unchecked` feature is not enabled.
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
impl Drop for NoDrop {
    fn drop(&mut self) {
        // Avoid double panic when we are already panicking
        #[allow(clippy::manual_assert)]
        if !std::thread::panicking() {
            panic!("linear type dropped");
        }
    }
}

#[test]
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
#[should_panic(expected = "linear type dropped")]
fn test_failed_destructure() {
    let _linear = Linear::new(123);
}
