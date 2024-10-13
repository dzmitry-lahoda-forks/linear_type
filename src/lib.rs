#![doc = include_str!("../README.md")]

use std::{fmt::Debug, mem::ManuallyDrop};

/// A linear type that must be destructured to access the inner value.
///
/// Linear types are a way to enforce that a value is used exactly once.
/// This is useful in cases where you want to ensure that a value propagates
/// into a final state which eventually gets consumed.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct Linear<T>(ManuallyDrop<T>, NoDrop);

impl<T> Linear<T> {
    /// Creates a new linear type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let linear = Linear::new(123);
    /// // a linear type must be used or destructured eventually
    /// let _ = linear.into_inner();
    /// ```
    pub const fn new(inner: T) -> Self {
        Self(ManuallyDrop::new(inner), NoDrop)
    }

    #[cfg(any(doc, feature = "semipure"))]
    /// Returns a reference to the inner value.
    ///
    /// In a pure linear type-system even immutable access to the inner value is not available
    /// because this may leak unwanted interior mutability or enable to clone the inner which
    /// would be impure (in a linear type system). When one doesn't do either then the rust
    /// type system/lifetimes are strong enough to be pure. This method is only available when
    /// one defines the `semipure` feature. It's then the decision of the programmer not to use
    /// any interior mutability/cloning or bend the rules and do something impure to make this
    /// crate more convenient to use.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let linear = Linear::new(123);
    /// # #[cfg(any(doc, feature = "semipure"))]
    /// assert_eq!(linear.get_ref(), &123);
    /// # linear.into_inner();
    /// ```
    pub fn get_ref(&self) -> &T {
        &self.0
    }

    /// Destructures the linear type and returns the inner type.  This must eventually be called on
    /// any linear type, failing to do so will panic when the linear type is dropped.
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

/// Additional map methods for `Linear<Result<R,E>>`
impl<T, E> Linear<Result<T, E>> {
    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<R,E>>` by applying a function
    /// to the `Ok` value.  Retains a `Err` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// # use std::io::Read;
    /// let result = Linear::new(std::fs::File::open("Cargo.toml"));
    /// let mapped = result.map_ok(|mut file| { let mut s = String::new(); file.read_to_string(&mut s)?; Ok(s)});
    /// assert!(mapped.unwrap_ok().into_inner().contains("linear_type"));
    /// ```
    pub fn map_ok<F: FnOnce(T) -> Result<R, E>, R>(self, f: F) -> Linear<Result<R, E>> {
        match self.into_inner() {
            Ok(t) => Linear::new(f(t)),
            Err(e) => Linear::new(Err(e)),
        }
    }

    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
    /// to the `Err` value.  Retains a `Ok` value.
    pub fn map_err<F: FnOnce(E) -> Result<T, R>, R>(self, f: F) -> Linear<Result<T, R>> {
        match self.into_inner() {
            Ok(t) => Linear::new(Ok(t)),
            Err(e) => Linear::new(f(e)),
        }
    }
}

/// Additional `unwrap_ok()` method for `Linear<Result<T,E>>` where E is `Debug`.
impl<T, E: Debug> Linear<Result<T, E>> {
    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<T>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Err`.
    pub fn unwrap_ok(self) -> Linear<T> {
        Linear::new(self.into_inner().unwrap())
    }
}

/// Additional `unwrap_err()` method for `Linear<Result<T,E>>` where T is `Debug`.
impl<T: Debug, E> Linear<Result<T, E>> {
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
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let option = Linear::new(Some(123));
    /// let mapped = option.map_some(|x| Some(x.to_string()));
    /// assert_eq!(mapped.unwrap_some().into_inner(), "123");
    /// ```
    pub fn map_some<F: FnOnce(T) -> Option<R>, R>(self, f: F) -> Linear<Option<R>> {
        match self.into_inner() {
            Some(t) => Linear::new(f(t)),
            None => Linear::new(None),
        }
    }

    /// Transforms a `Linear<Option<T>>` into `Linear<Option<T>>` by applying a function
    /// to the `None` value.  Retains a `Some` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let option = Linear::new(None);
    /// let mapped = option.or_else(|| Some(123));
    /// assert_eq!(mapped.unwrap_some().into_inner(), 123);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::Linear;
    /// let option = Linear::new(Some(123));
    /// let unwrapped = option.unwrap_some();
    /// assert_eq!(unwrapped.into_inner(), 123);
    /// ```
    pub fn unwrap_some(self) -> Linear<T> {
        Linear::new(self.into_inner().unwrap())
    }
}

/// A marker type that can not be dropped.
///
/// # Panics or Aborts
///
/// When the `drop_unchecked` feature is not enabled, this type will panic in tests when dropped
/// or abort in non-test builds. This is to ensure that linear types are not dropped and must be
/// destructured manually. Dropping a linear type is considered a programming error and
/// must not happen. The panic in test builds is only there to permit completion of the test suite.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
struct NoDrop;

/// Drop is only implemented when either `debug_assertions` are enabled or the
/// `drop_unchecked` feature is not enabled.
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
impl Drop for NoDrop {
    #[cfg(test)]
    fn drop(&mut self) {
        // Avoid double panic when we are already panicking
        #[allow(clippy::manual_assert)]
        if !std::thread::panicking() {
            panic!("linear type dropped");
        }
    }
    #[cfg(not(test))]
    fn drop(&mut self) {
        // be nice in debug builds and tell why we are aborting
        #[cfg(debug_assertions)]
        eprintln!("linear type dropped");
        std::process::abort();
    }
}

#[test]
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
#[should_panic(expected = "linear type dropped")]
fn test_failed_destructure() {
    let _linear = Linear::new(123);
}
