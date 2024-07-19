#![doc = include_str!("../README.md")]

use std::{fmt::Debug, mem::ManuallyDrop};

/// A linear type that must be destructured to access the inner value.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct Linear<T>(ManuallyDrop<T>, NoDrop);

impl<T> Linear<T> {
    /// Creates a new linear type.
    pub const fn new(inner: T) -> Self {
        Self(ManuallyDrop::new(inner), NoDrop)
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
        ManuallyDrop::into_inner(t)
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

/// Additional methods for `Linear<Result<R,E>>`, only fundamental map and unwrap methods are
/// supported. Anything beyond that needs to be handled manually.
impl<T: Debug, E: Debug> Linear<Result<T, E>> {
    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<R,E>>` by applying a function
    /// to the `Ok` value.  Retains a `Err` value.
    pub fn map_ok<F: FnOnce(T) -> Result<R,E>, R>(self, f: F) -> Linear<Result<R, E>> {
        match self.into_inner() {
            Ok(t) => Linear::new(f(t)),
            Err(e) => Linear::new(Err(e)),
        }
    }

    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
    /// to the `Err` value.  Retains a `Ok` value.
    pub fn map_err<F: FnOnce(E) -> Result<T,R>, R>(self, f: F) -> Linear<Result<T, R>> {
        match self.into_inner() {
            Ok(t) => Linear::new(Ok(t)),
            Err(e) => Linear::new(f(e)),
        }
    }

    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<T>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Err`.
    pub fn unwrap_ok(self) -> Linear<T> {
        Linear::new(self.into_inner().unwrap())
    }

    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<E>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Ok`.
    pub fn unwrap_err(self) -> Linear<E> {
        Linear::new(self.into_inner().unwrap_err())
    }
}

/// Additional methods for `Linear<Option<T>>`, only fundamental methods are supported.
/// Anything beyond that needs to be handled manually.
impl<T> Linear<Option<T>> {
    /// Transforms a `Linear<Option<T>>` into `Linear<Option<R>>` by applying a function
    /// to the `Some` value.  Retains a `None` value.
    pub fn map_some<F: FnOnce(T) -> Option<R>, R>(self, f: F) -> Linear<Option<R>> {
        match self.into_inner() {
            Some(t) => Linear::new(f(t)),
            None => Linear::new(None),
        }
    }

    /// Transforms a `Linear<Option<T>>` into `Linear<Option<T>>` by applying a function
    /// to the `None` value.  Retains a `Some` value.
    pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Self {
        match self.into_inner() {
            inner @ Some(_) => Linear::new(inner),
            None => Linear::new(f()),
        }
    }

    /// Unwraps a `Linear<Some<T>>` into a `Linear<T>`.
    ///
    /// # Panics
    ///
    /// When the value is `None`.
    pub fn unwrap_some(self) -> Linear<T> {
        Linear::new(self.into_inner().unwrap())
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
