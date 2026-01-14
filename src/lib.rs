#![doc = include_str!("../README.md")]

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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct Linear<T, U>(T, PhantomData<U>);

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
        Linear(inner, PhantomData)
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
    /// assert_eq!(*linear.get_ref(), 123);
    /// # linear.into_inner();
    /// ```
    pub const fn get_ref(&self) -> &T {
        &self.0
    }

    /// Let a function inspect the inner value.
    ///
    /// This takes a plain function pointer (can be written as non capturing closure
    /// syntax). This is necessary because this function must not capture outside state and
    /// have any side effect except for diagnostic ones. This make inspect reasonably more
    /// sound so it wont use the `semipure` feature flag.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let linear = new_linear!(123).inspect(|i| assert_eq!(*i, 123));
    /// # linear.destroy();
    /// ```
    pub fn inspect(self, f: fn(&T)) -> Self {
        f(&self.0);
        self
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
        let this = ManuallyDrop::new(self);
        unsafe { std::ptr::read(&this.0) }
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
    pub fn destroy(self) {
        drop(self.into_inner());
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
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Linear<R, Map<F, Self>> {
        Self::transpose(f(self.into_inner()))
    }

    #[inline]
    const fn transpose<R, S>(r: R) -> Linear<R, S> {
        Linear(r, PhantomData)
    }

    /// Splices a linear type into a two-tuple of linear types. The caller has to ensure that
    /// the resulting values are independent and don't share any observable state.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let foobar = new_linear!("foobar".to_string());
    /// let (foo, bar) = foobar.splice(|mut foo| {let bar = foo.split_off(3); (foo, bar)});
    /// assert_eq!(foo.into_inner(), "foo");
    /// assert_eq!(bar.into_inner(), "bar");
    /// ```
    pub fn splice<F: FnOnce(T) -> (R, S), R, S>(
        self,
        f: F,
    ) -> (
        Linear<R, SpliceLeft<F, Self>>,
        Linear<S, SpliceRight<F, Self>>,
    ) {
        let (r, s) = f(self.into_inner());
        (Self::transpose(r), Self::transpose(s))
    }

    /// Merges two linear type into one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linear_type::*;
    /// let foo = new_linear!("foo".to_string());
    /// let bar = new_linear!("bar".to_string());
    /// let foobar = foo.merge(bar, |foo, bar| { foo + &bar });
    /// println!("{}", std::any::type_name_of_val(&foobar));
    /// assert_eq!(foobar.into_inner(), "foobar");
    /// ```
    pub fn merge<F: FnOnce(T, T2) -> R, R, T2, U2>(
        self,
        other: Linear<T2, U2>,
        f: F,
    ) -> Linear<R, Merge<F, Self, Linear<T2, U2>>> {
        Self::transpose(f(self.into_inner(), other.into_inner()))
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
    pub fn map_ok<F: FnOnce(T) -> Result<R, E>, R>(
        self,
        f: F,
    ) -> Linear<Result<R, E>, MapOk<F, Self>> {
        match self.into_inner() {
            Ok(t) => Self::transpose(f(t)),
            Err(e) => Self::transpose(Err(e)),
        }
    }

    /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
    /// to the `Err` value.  Retains a `Ok` value.
    pub fn map_err<F: FnOnce(E) -> Result<T, R>, R>(
        self,
        f: F,
    ) -> Linear<Result<T, R>, MapErr<F, Self>> {
        match self.into_inner() {
            Ok(t) => Self::transpose(Ok(t)),
            Err(e) => Self::transpose(f(e)),
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
    pub fn unwrap_ok(self) -> Linear<T, UnwrapOk<Self>> {
        Self::transpose(self.into_inner().unwrap())
    }
}

/// Additional `unwrap_err()` method for `Linear<Result<T,E>>` where T is `Debug`.
impl<T: Debug, E, U> Linear<Result<T, E>, U> {
    /// Unwraps a `Linear<Result<T,E>>` into a `Linear<E>`.
    ///
    /// # Panics
    ///
    /// When the value is an `Ok`.
    pub fn unwrap_err(self) -> Linear<E, UnwrapErr<Self>> {
        Self::transpose(self.into_inner().unwrap_err())
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
    pub fn map_some<F: FnOnce(T) -> Option<R>, R>(
        self,
        f: F,
    ) -> Linear<Option<R>, MapSome<F, Self>> {
        match self.into_inner() {
            Some(t) => Self::transpose(f(t)),
            None => Self::transpose(None),
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
    pub fn or_else<F: FnOnce() -> Option<T>>(self, f: F) -> Linear<Option<T>, OrElse<F, Self>> {
        match self.into_inner() {
            inner @ Some(_) => Self::transpose(inner),
            None => Self::transpose(f()),
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
    pub fn unwrap_some(self) -> Linear<T, UnwrapSome<Self>> {
        Self::transpose(self.into_inner().unwrap())
    }
}

// All operations have Markers to construct unique types
#[doc(hidden)]
pub struct Map<F, M>(PhantomData<F>, PhantomData<M>);
#[doc(hidden)]
pub struct MapOk<F, R>(PhantomData<F>, PhantomData<R>);
#[doc(hidden)]
pub struct MapErr<F, E>(PhantomData<F>, PhantomData<E>);
#[doc(hidden)]
pub struct MapSome<F, S>(PhantomData<F>, PhantomData<S>);
#[doc(hidden)]
pub struct UnwrapOk<R>(PhantomData<R>);
#[doc(hidden)]
pub struct UnwrapErr<E>(PhantomData<E>);
#[doc(hidden)]
pub struct UnwrapSome<S>(PhantomData<S>);
#[doc(hidden)]
pub struct OrElse<F, E>(PhantomData<F>, PhantomData<E>);
#[doc(hidden)]
pub struct SpliceLeft<F, L>(PhantomData<F>, PhantomData<L>);
#[doc(hidden)]
pub struct SpliceRight<F, R>(PhantomData<F>, PhantomData<R>);
#[doc(hidden)]
pub struct Merge<F, L, R>(PhantomData<F>, PhantomData<L>, PhantomData<R>);

impl<T, U> Drop for Linear<T, U> {
    #[cfg_attr(any(test, debug_assertions), track_caller)]
    fn drop(&mut self) {
        // Avoid double panic when we are already panicking
        #[cfg(not(feature = "drop_unchecked"))]
        #[allow(clippy::manual_assert)]
        if !std::thread::panicking() {
            panic!("linear type dropped {}", std::any::type_name::<Self>());
        }
    }
}
