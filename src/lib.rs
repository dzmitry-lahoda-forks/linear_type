#![doc = include_str!("../README.md")]

use core::mem::ManuallyDrop;

/// Linearity holder. Carries the unique type marker and ensures a linear value is not dropped.
#[doc(hidden)]
pub struct Linearity<U>(
    NoDrop,
    core::marker::PhantomData<U>,
    core::cell::Cell<()>, // Cell<()> is just stable !Sync
);

impl<U> PartialEq for Linearity<U> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<U> Eq for Linearity<U> {}

impl<U> PartialOrd for Linearity<U> {
    fn partial_cmp(&self, _other: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<U> Ord for Linearity<U> {
    fn cmp(&self, _other: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

#[doc(hidden)]
pub const fn __linearity<U>() -> Linearity<U> {
    Linearity(
        NoDrop,
        core::marker::PhantomData,
        core::cell::Cell::new(()),
    )
}

#[doc(hidden)]
pub const fn __linear_from_parts<T, U>(value: T) -> Linear<T, U> {
    Linear(::core::mem::ManuallyDrop::new(value), __linearity::<U>())
}

/// Generates linear newtype from newtype name and inner value type.
/// `Linear<T, U>`` is just generated generic variant with some added extra helpers for uniquness
#[macro_export]
macro_rules! linear {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$t:ident, $u:ident>($inner:ty);
    ) => {
        $(#[$meta])*
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        #[must_use]
        $vis struct $name<$t, $u>(
            ::core::mem::ManuallyDrop<$inner>,
            $crate::Linearity<$u>,
        );

        /// Hashes only inner value.
        impl<$t: ::core::hash::Hash, $u> ::core::hash::Hash for $name<$t, $u> {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        /// Custom debug outputing value and lifetime type
        impl<$t: ::core::fmt::Debug, $u> ::core::fmt::Debug for $name<$t, $u> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .field(&::core::any::type_name::<$u>())
                    .finish()
            }
        }

        #[doc(hidden)]
        impl<$t, F: Fn()> $name<$t, $crate::UniqueType<F>> {
            // to be called by the new_linear macro
            #[doc(hidden)]
            pub const fn new(inner: $inner, _: $crate::UniqueType<F>) -> Self {
                $name(
                    ::core::mem::ManuallyDrop::new(inner),
                    $crate::__linearity::<$crate::UniqueType<F>>(),
                )
            }
        }

        impl<$t, $u> $name<$t, $u> {
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
            /// # linear.into();
            /// ```
            pub fn get_ref(&self) -> &$inner {
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
            /// let inner = linear.into();
            /// assert_eq!(inner, 123);
            /// ```
            pub fn into(self) -> $inner {
                let $name(t, linearity) = self;
                ::core::mem::forget(linearity);
                ::core::mem::ManuallyDrop::into_inner(t)
            }

            /// Consumes and destroys the wrapped value. This is like `into()` and them dropping
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
                    ::core::mem::ManuallyDrop::drop(&mut self.0);
                }
                let $name(_, linearity) = self;
                ::core::mem::forget(linearity);
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
            /// assert_eq!(string.into(), "123");
            /// ```
            pub fn map<F: FnOnce($inner) -> R, R>(self, f: F) -> $name<R, Self> {
                Self::transpose(f(self.into()))
            }

            const fn transpose<R>(r: R) -> $name<R, Self> {
                $name(
                    ::core::mem::ManuallyDrop::new(r),
                    $crate::__linearity::<Self>(),
                )
            }
        }

        /// Additional map methods for `Linear<Result<R,E>>`
        impl<$t, E, $u> $name<::core::result::Result<$t, E>, $u> {
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
            /// assert!(mapped.unwrap_ok().into().contains("linear_type"));
            /// ```
            pub fn map_ok<F: FnOnce($t) -> ::core::result::Result<R, E>, R>(
                self,
                f: F,
            ) -> $name<::core::result::Result<R, E>, Self> {
                match self.into() {
                    Ok(t) => Self::transpose(f(t)),
                    Err(e) => Self::transpose(Err(e)),
                }
            }

            /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
            /// to the `Err` value.  Retains a `Ok` value.
            pub fn map_err<F: FnOnce(E) -> ::core::result::Result<$t, R>, R>(
                self,
                f: F,
            ) -> $name<::core::result::Result<$t, R>, Self> {
                match self.into() {
                    Ok(t) => Self::transpose(Ok(t)),
                    Err(e) => Self::transpose(f(e)),
                }
            }
        }

        /// Additional `unwrap_ok()` method for `Linear<Result<T,E>>` where E is `Debug`.
        impl<$t, E: ::core::fmt::Debug, $u> $name<::core::result::Result<$t, E>, $u> {
            /// Unwraps a `Linear<Result<T,E>>` into a `Linear<T>`.
            ///
            /// # Panics
            ///
            /// When the value is an `Err`.
            pub fn unwrap_ok(self) -> $name<$t, Self> {
                $name::transpose(self.into().unwrap())
            }
        }

        /// Additional `unwrap_err()` method for `Linear<Result<T,E>>` where T is `Debug`.
        impl<$t: ::core::fmt::Debug, E, $u> $name<::core::result::Result<$t, E>, $u> {
            /// Unwraps a `Linear<Result<T,E>>` into a `Linear<E>`.
            ///
            /// # Panics
            ///
            /// When the value is an `Ok`.
            pub fn unwrap_err(self) -> $name<E, Self> {
                $name::transpose(self.into().unwrap_err())
            }
        }

        /// Additional methods for `Linear<Option<T>>`, only fundamental methods are supported.
        /// Anything beyond that needs to be handled manually.
        impl<$t, $u> $name<::core::option::Option<$t>, $u> {
            /// Transforms a `Linear<Option<T>>` into `Linear<Option<R>>` by applying a function
            /// to the `Some` value.  Retains a `None` value.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// let option = new_linear!(Some(123));
            /// let mapped = option.map_some(|x| Some(x.to_string()));
            /// assert_eq!(mapped.unwrap_some().into(), "123");
            /// ```
            pub fn map_some<F: FnOnce($t) -> ::core::option::Option<R>, R>(
                self,
                f: F,
            ) -> $name<::core::option::Option<R>, Self> {
                match self.into() {
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
            /// assert_eq!(mapped.unwrap_some().into(), 123);
            /// ```
            pub fn or_else<F: FnOnce() -> ::core::option::Option<$t>>(
                self,
                f: F,
            ) -> $name<::core::option::Option<$t>, Self> {
                match self.into() {
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
            /// assert_eq!(unwrapped.into(), 123);
            /// ```
            pub fn unwrap_some(self) -> $name<$t, Self> {
                $name::transpose(self.into().unwrap())
            }
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident($inner:ty);
    ) => {
        $(#[$meta])*
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        #[must_use]
        $vis struct $name(
            ::core::mem::ManuallyDrop<$inner>,
            $crate::Linearity<$crate::UniqueType<fn()>>,
        );

        /// Hashes only inner value.
        impl ::core::hash::Hash for $name
        where
            $inner: ::core::hash::Hash,
        {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        /// Custom debug outputing value and lifetime type
        impl ::core::fmt::Debug for $name
        where
            $inner: ::core::fmt::Debug,
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .field(&::core::any::type_name::<$crate::UniqueType<fn()>>())
                    .finish()
            }
        }

        impl $name {
            /// Constructs a new value with the fixed `U` type.
            pub const fn new(inner: $inner) -> Self {
                $name(
                    ::core::mem::ManuallyDrop::new(inner),
                    $crate::__linearity::<$crate::UniqueType<fn()>>(),
                )
            }

            #[cfg(any(doc, feature = "semipure"))]
            /// Returns a reference to the inner value.
            pub fn get_ref(&self) -> &$inner {
                &self.0
            }

            /// Destructures the linear type and returns the inner type.  This must eventually be called on
            /// any linear type, failing to do so will panic when the linear type is dropped.
            pub fn into(self) -> $inner {
                let $name(t, linearity) = self;
                ::core::mem::forget(linearity);
                ::core::mem::ManuallyDrop::into_inner(t)
            }

            /// Consumes and destroys the wrapped value. This is like `into()` and them dropping
            /// the returned value.
            #[inline]
            pub fn destroy(mut self) {
                unsafe {
                    ::core::mem::ManuallyDrop::drop(&mut self.0);
                }
                let $name(_, linearity) = self;
                ::core::mem::forget(linearity);
            }

            /// Transforms one linear type to another linear type. The inner value is passed to the
            /// closure and the return value is wrapped in a `Linear`.
            pub fn map<F: FnOnce($inner) -> R, R>(self, f: F) -> $crate::Linear<R, Self> {
                $crate::__linear_from_parts::<R, Self>(f(self.into()))
            }
        }

        // No Result/Option extensions for the fully concrete variant.
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident<$t:ident>($inner:ty);
    ) => {
        $(#[$meta])*
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        #[must_use]
        $vis struct $name<$t>(
            ::core::mem::ManuallyDrop<$inner>,
            $crate::Linearity<$crate::UniqueType<fn()>>,
        );

        /// Hashes only inner value.
        impl<$t: ::core::hash::Hash> ::core::hash::Hash for $name<$t> {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        /// Custom debug outputing value and lifetime type
        impl<$t: ::core::fmt::Debug> ::core::fmt::Debug for $name<$t> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.0)
                    .field(&::core::any::type_name::<$crate::UniqueType<fn()>>())
                    .finish()
            }
        }

        impl<$t> $name<$t> {
            /// Constructs a new value with the fixed `U` type.
            pub const fn new(inner: $inner) -> Self {
                $name(
                    ::core::mem::ManuallyDrop::new(inner),
                    $crate::__linearity::<$crate::UniqueType<fn()>>(),
                )
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
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let linear = Example::new(123);
            /// # #[cfg(any(doc, feature = "semipure"))]
            /// assert_eq!(linear.get_ref(), &123);
            /// # linear.into();
            /// ```
            pub fn get_ref(&self) -> &$inner {
                &self.0
            }

            /// Destructures the linear type and returns the inner type.  This must eventually be called on
            /// any linear type, failing to do so will panic when the linear type is dropped.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let linear = Example::new(123);
            /// let inner = linear.into();
            /// assert_eq!(inner, 123);
            /// ```
            pub fn into(self) -> $inner {
                let $name(t, linearity) = self;
                ::core::mem::forget(linearity);
                ::core::mem::ManuallyDrop::into_inner(t)
            }

            /// Consumes and destroys the wrapped value. This is like `into()` and them dropping
            /// the returned value.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let linear = Example::new(123);
            /// linear.destroy();
            /// ```
            #[inline]
            pub fn destroy(mut self) {
                unsafe {
                    ::core::mem::ManuallyDrop::drop(&mut self.0);
                }
                let $name(_, linearity) = self;
                ::core::mem::forget(linearity);
            }

            /// Transforms one linear type to another linear type. The inner value is passed to the
            /// closure and the return value is wrapped in a `Linear`.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let number = Example::new(123);
            /// let string = number.map(|x| x.to_string());
            /// assert_eq!(string.into(), "123");
            /// ```
            pub fn map<F: FnOnce($inner) -> R, R>(self, f: F) -> $name<R> {
                $name::<R>::transpose(f(self.into()))
            }

            const fn transpose<R>(r: R) -> $name<R> {
                $name(
                    ::core::mem::ManuallyDrop::new(r),
                    $crate::__linearity::<$u_ty>(),
                )
            }
        }

        /// Additional map methods for `Linear<Result<R,E>>`
        impl<$t, E> $name<::core::result::Result<$t, E>> {
            /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<R,E>>` by applying a function
            /// to the `Ok` value.  Retains a `Err` value.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # use std::io::Read;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let result = Example::new(std::fs::File::open("Cargo.toml"));
            /// let mapped = result.map_ok(|mut file| { let mut s = String::new(); file.read_to_string(&mut s)?; Ok(s)});
            /// assert!(mapped.unwrap_ok().into().contains("linear_type"));
            /// ```
            pub fn map_ok<F: FnOnce($t) -> ::core::result::Result<R, E>, R>(
                self,
                f: F,
            ) -> $name<::core::result::Result<R, E>> {
                match self.into() {
                    Ok(t) => $name::<::core::result::Result<R, E>>::transpose(f(t)),
                    Err(e) => $name::<::core::result::Result<R, E>>::transpose(Err(e)),
                }
            }

            /// Transforms a `Linear<Result<T,E>>` into `Linear<Result<T, R>>` by applying a function
            /// to the `Err` value.  Retains a `Ok` value.
            pub fn map_err<F: FnOnce(E) -> ::core::result::Result<$t, R>, R>(
                self,
                f: F,
            ) -> $name<::core::result::Result<$t, R>> {
                match self.into() {
                    Ok(t) => $name::<::core::result::Result<$t, R>>::transpose(Ok(t)),
                    Err(e) => $name::<::core::result::Result<$t, R>>::transpose(f(e)),
                }
            }
        }

        /// Additional `unwrap_ok()` method for `Linear<Result<T,E>>` where E is `Debug`.
        impl<$t, E: ::core::fmt::Debug> $name<::core::result::Result<$t, E>> {
            /// Unwraps a `Linear<Result<T,E>>` into a `Linear<T>`.
            ///
            /// # Panics
            ///
            /// When the value is an `Err`.
            pub fn unwrap_ok(self) -> $name<$t> {
                $name::<$t>::transpose(self.into().unwrap())
            }
        }

        /// Additional `unwrap_err()` method for `Linear<Result<T,E>>` where T is `Debug`.
        impl<$t: ::core::fmt::Debug, E> $name<::core::result::Result<$t, E>> {
            /// Unwraps a `Linear<Result<T,E>>` into a `Linear<E>`.
            ///
            /// # Panics
            ///
            /// When the value is an `Ok`.
            pub fn unwrap_err(self) -> $name<E> {
                $name::<E>::transpose(self.into().unwrap_err())
            }
        }

        /// Additional methods for `Linear<Option<T>>`, only fundamental methods are supported.
        /// Anything beyond that needs to be handled manually.
        impl<$t> $name<::core::option::Option<$t>> {
            /// Transforms a `Linear<Option<T>>` into `Linear<Option<R>>` by applying a function
            /// to the `Some` value.  Retains a `None` value.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let option = Example::new(Some(123));
            /// let mapped = option.map_some(|x| Some(x.to_string()));
            /// assert_eq!(mapped.unwrap_some().into(), "123");
            /// ```
            pub fn map_some<F: FnOnce($t) -> ::core::option::Option<R>, R>(
                self,
                f: F,
            ) -> $name<::core::option::Option<R>> {
                match self.into() {
                    Some(t) => $name::<::core::option::Option<R>>::transpose(f(t)),
                    None => $name::<::core::option::Option<R>>::transpose(None),
                }
            }

            /// Transforms a `Linear<Option<T>>` into `Linear<Option<T>>` by applying a function
            /// to the `None` value.  Retains a `Some` value.
            ///
            /// # Example
            ///
            /// ```rust
            /// # use linear_type::*;
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let option = Example::new(None);
            /// let mapped = option.or_else(|| Some(123));
            /// assert_eq!(mapped.unwrap_some().into(), 123);
            /// ```
            pub fn or_else<F: FnOnce() -> ::core::option::Option<$t>>(
                self,
                f: F,
            ) -> $name<::core::option::Option<$t>> {
                match self.into() {
                    inner @ Some(_) => $name::<::core::option::Option<$t>>::transpose(inner),
                    None => $name::<::core::option::Option<$t>>::transpose(f()),
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
            /// # linear! { pub struct Example<T>(T, UniqueType<fn()>); }
            /// let option = Example::new(Some(123));
            /// let unwrapped = option.unwrap_some();
            /// assert_eq!(unwrapped.into(), 123);
            /// ```
            pub fn unwrap_some(self) -> $name<$t> {
                $name::<$t>::transpose(self.into().unwrap())
            }
        }
    };
}

linear! {
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
    pub struct Linear<T, U>(T);
}

/// Must use type to be occasionally used in function boundaries
pub type MustUse<T> = Linear<T, UniqueType<fn()>>;

/// Type based must_use equivalent
pub fn must_use<T>(val: T) -> MustUse<T> {
    MustUse::new(val, unique!())
}

/// A marker struct that is constructed with unique closure types.
pub struct UniqueType<F: Fn()>(pub ManuallyDrop<F>);

// Returns a value with a unique type for every call.
#[doc(hidden)]
#[macro_export]
macro_rules! unique {
    () => {
        $crate::UniqueType(std::mem::ManuallyDrop::new(|| ()))
    };
}

/// Wraps a value of type `T` in `Linear<T>`. This macro ensures that every new instance has a
/// unique type.
#[macro_export]
macro_rules! new_linear {
    ($t:expr) => {
        $crate::Linear::new($t, $crate::unique!())
    };
}

/// ```compile_fail
/// use linear_type::new_linear;
///
/// // This must not compile because `foo` and `bar` are distinct types.
/// let foo = new_linear!("test");
/// let mut bar = new_linear!("test");
/// bar = foo;
/// ```

/// A marker type that can not be dropped.
///
/// # Panics or Aborts
///
/// When the `drop_unchecked` feature is not enabled, this type will panic in tests when dropped
/// or abort in non-test builds. This is to ensure that linear types are not dropped and must be
/// destructured manually. Dropping a linear type is considered a programming error and
/// must not happen. The panic in test builds is only there to permit completion of the test suite.
///
#[doc(hidden)]
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

linear! {
    /// Linear string.
    pub struct LinearString(String);
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic]
    fn panics() {
        let _ = crate::LinearString::new("Hello".to_string());
    }


    linear!(pub struct Foo(u32););    
} 
