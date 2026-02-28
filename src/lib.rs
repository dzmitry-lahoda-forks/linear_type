#![doc = include_str!("../README.md")]

use core::any::type_name;
use std::{fmt::Debug, marker::PhantomData, mem::ManuallyDrop};

/// A linear type that must be destructured to access the inner value.
///
/// Linear types are a way to enforce that a value is used exactly once.  This is useful in
/// cases where you want to ensure that a value propagates into a final state which eventually
/// gets consumed.
///
/// Linear types are unique and enforce continuity every state transformation creates a new
/// type that is tagged with the type signature it is created from. This is the `U` generic
/// parameter.  Thus means one can not make up linear typed values from thin air and use them
/// as substitutes for a destroyed value in a chain of linear evaluation.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub struct Linear<T, U>(
    ManuallyDrop<T>,
    NoDrop,
    PhantomData<U>,
    core::cell::Cell<()>,
); // Cell<()> is just stable !Sync

/// Hashes only inner value.
impl<T: core::hash::Hash, U> core::hash::Hash for Linear<T, U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Custom debug outputing value and lifetime type
impl<T: Debug, U> core::fmt::Debug for Linear<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Linear")
            .field(&self.0)
            .field(&type_name::<U>())
            .finish()
    }
}

/// Must use type to be occasionally used in function boundaries
pub type MustUse<T> = Linear<T, UniqueType<fn()>>;

/// Type based must_use equivalent
pub fn must_use<T>(val: T) -> MustUse<T> {
    MustUse::new(val, unique_type!())
}

/// A marker struct that is constructed with unique closure types.
pub struct UniqueType<F: Fn()>(pub ManuallyDrop<F>);

// Returns a value with a unique type for every call.
#[doc(hidden)]
#[macro_export]
macro_rules! unique_type {
    () => {
        $crate::UniqueType(std::mem::ManuallyDrop::new(|| ()))
    };
}

/// Wraps a value of type `T` in `Linear<T>`. This macro ensures that every new instance has a
/// unique type.
#[macro_export]
macro_rules! new_linear {
    ($t:expr) => {
        $crate::Linear::new($t, $crate::unique_type!())
    };
}

#[doc(hidden)]
impl<T, F: Fn()> Linear<T, UniqueType<F>> {
    // to be called by the new_linear macro
    #[doc(hidden)]
    pub const fn new(inner: T, _: UniqueType<F>) -> Self {
        Linear(
            ManuallyDrop::new(inner),
            NoDrop,
            PhantomData,
            core::cell::Cell::new(()),
        )
    }
}

// This must not compile because foo and bar are distinct types
// Its a hack and gives a rather cryptic error messages about "expected closure, found a different closure"
// #[test]
// fn uniqueness() {
//     let foo = new_linear!("test");
//     let mut bar = new_linear!("test");
//     bar = foo;
// }

impl<T, U> Linear<T, U> {
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
    /// # use linear_type::*;
    /// let linear = new_linear!(123);
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
    /// # use linear_type::*;
    /// let linear = new_linear!(123);
    /// let inner = linear.into_inner();
    /// assert_eq!(inner, 123);
    /// ```
    pub fn into_inner(self) -> T {
        let Linear(t, n, _, _) = self;
        std::mem::forget(n);
        ManuallyDrop::into_inner(t)
    }

    /// Consumes and destroys the wrapped value. This is like `into_inner()` and them dropping
    /// the returned value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let linear = new_linear!(123);
    /// linear.destroy();
    /// ```
    #[inline]
    pub fn destroy(mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.0);
        }
        let Linear(_, n, _, _) = self;
        std::mem::forget(n);
    }

    /// Transforms one linear type to another linear type. The inner value is passed to the
    /// closure and the return value is wrapped in a `Linear`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let number = new_linear!(123);
    /// let string = number.map(|x| x.to_string());
    /// assert_eq!(string.into_inner(), "123");
    /// ```
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Linear<R, Self> {
        Self::transpose(f(self.into_inner()))
    }

    const fn transpose<R>(r: R) -> Linear<R, Self> {
        Linear(
            ManuallyDrop::new(r),
            NoDrop,
            PhantomData,
            core::cell::Cell::new(()),
        )
    }
}

/// Additional map methods for `Linear<Result<R,E>>`
impl<T, E, U> Linear<Result<T, E>, U> {
    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<R,E>>` by applying a function
    /// to the `Ok` value.  Retains a `Err` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// # use std::io::Read;
    /// let result = new_linear!(std::fs::File::open("Cargo.toml"));
    /// let mapped = result.map_ok(|mut file| { let mut s = String::new(); file.read_to_string(&mut s)?; Ok(s)});
    /// assert!(mapped.unwrap_ok().into_inner().contains("linear_type"));
    /// ```
    pub fn map_ok<F: FnOnce(T) -> Result<R, E>, R>(self, f: F) -> Linear<Result<R, E>, Self> {
        match self.into_inner() {
            Ok(t) => Self::transpose(f(t)),
            Err(e) => Self::transpose(Err(e)),
        }
    }

    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
    /// to the `Err` value.  Retains a `Ok` value.
    pub fn map_err<F: FnOnce(E) -> Result<T, R>, R>(self, f: F) -> Linear<Result<T, R>, Self> {
        match self.into_inner() {
            Ok(t) => Linear::transpose(Ok(t)),
            Err(e) => Linear::transpose(f(e)),
        }
    }
}

/// Additional `unwrap_ok()` method for `Linear<Result<T,E>>` where E is `Debug`.
impl<T, E: Debug, U> Linear<Result<T, E>, U> {
    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<T>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Err`.
    pub fn unwrap_ok(self) -> Linear<T, Self> {
        Linear::transpose(self.into_inner().unwrap())
    }
}

/// Additional `unwrap_err()` method for `Linear<Result<T,E>>` where T is `Debug`.
impl<T: Debug, E, U> Linear<Result<T, E>, U> {
    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<E>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Ok`.
    pub fn unwrap_err(self) -> Linear<E, Self> {
        Linear::transpose(self.into_inner().unwrap_err())
    }
}

/// Additional methods for `Linear<Option<T>>`, only fundamental methods are supported.
/// Anything beyond that needs to be handled manually.
impl<T, U> Linear<Option<T>, U> {
    /// Transforms a `Linear<Option<T>>` into `Linear<Option<R>>` by applying a function
    /// to the `Some` value.  Retains a `None` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let option = new_linear!(Some(123));
    /// let mapped = option.map_some(|x| Some(x.to_string()));
    /// assert_eq!(mapped.unwrap_some().into_inner(), "123");
    /// ```
    pub fn map_some<F: FnOnce(T) -> Option<R>, R>(self, f: F) -> Linear<Option<R>, Self> {
        match self.into_inner() {
            Some(t) => Linear::transpose(f(t)),
            None => Linear::transpose(None),
        }
    }

    /// Transforms a `Linear<Option<T>>` into `Linear<Option<T>>` by applying a function
    /// to the `None` value.  Retains a `Some` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let option = new_linear!(None);
    /// let mapped = option.or_else(|| Some(123));
    /// assert_eq!(mapped.unwrap_some().into_inner(), 123);
    /// ```
    pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Linear<Option<T>, Self> {
        match self.into_inner() {
            inner @ Some(_) => Linear::transpose(inner),
            None => Linear::transpose(f()),
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
    /// # use linear_type::*;
    /// let option = new_linear!(Some(123));
    /// let unwrapped = option.unwrap_some();
    /// assert_eq!(unwrapped.into_inner(), 123);
    /// ```
    pub fn unwrap_some(self) -> Linear<T, Self> {
        Linear::transpose(self.into_inner().unwrap())
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
