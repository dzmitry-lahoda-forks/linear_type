#![doc = include_str!("../README.md")]
#![warn(clippy::cargo_common_metadata)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::missing_panics_doc)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

use std::ops::{Deref, DerefMut};

#[allow(unused_imports)]
use std::mem::ManuallyDrop;

/// A Cell like struct that wraps a T and can be derefernced to &T.  This cell can not be
/// dropped. For destruction of the inner value one has to destructure the linear type with
/// `.into_inner()`. Usually this is done in manual destructors.
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Linear<T>(Option<T>);

#[doc(hidden)]
#[cfg(all(not(debug_assertions), feature = "drop_unchecked"))]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Linear<T>(ManuallyDrop<T>);

#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
impl<T> Linear<T> {
    /// Creates a new `Linear<T>` wrapping the supplied value.
    #[must_use]
    pub fn new(t: T) -> Self {
        Linear(Some(t))
    }

    /// Destructures a `Linear<T>` and returns the contained value.  This must eventually be
    /// called on any `Linear<T>`, failing to do so will panic or show a compile error.
    #[must_use]
    pub fn into_inner(mut self) -> T {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.take().unwrap_unchecked()
        }
    }

    #[inline]
    fn get(&self) -> &T {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.as_ref().unwrap_unchecked()
        }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.as_mut().unwrap_unchecked()
        }
    }
}

#[cfg(all(not(debug_assertions), feature = "drop_unchecked"))]
impl<T> Linear<T> {
    /// Creates a new `Linear<T>` wrapping the supplied value.
    #[must_use]
    pub fn new(t: T) -> Self {
        Linear(ManuallyDrop::new(t))
    }

    /// Destructures a `Linear<T>` and returns the contained value.  This must eventually be
    /// called on any `Linear<T>`, failing to do so will panic or show a compile error.
    #[must_use]
    pub fn into_inner(mut self) -> T {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains invalid data
            // because '.into_inner()' consuming 'self'.
            ManuallyDrop::take(&mut self.0)
        }
    }

    #[inline]
    fn get(&self) -> &T {
        &*self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        &mut *self.0
    }
}

impl<T> Deref for Linear<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Linear<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T: AsRef<U>, U> AsRef<U> for Linear<T> {
    fn as_ref(&self) -> &U {
        self.get().as_ref()
    }
}

impl<T: AsMut<U>, U> AsMut<U> for Linear<T> {
    fn as_mut(&mut self) -> &mut U {
        self.get_mut().as_mut()
    }
}

/// Drop is only implemented when either `debug_assertions` are enabled or the
/// `drop_unchecked` feature is not enabled.
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
impl<T> Drop for Linear<T> {
    #[cfg_attr(feature = "compile_error", no_panic::no_panic)]
    fn drop(&mut self) {
        // Avoid double panic when we are already panicking
        if self.0.is_some() && !std::thread::panicking() {
            panic!("linear type dropped");
        }
    }
}

#[test]
fn test_destructure() {
    let linear = Linear::new(123);
    let _ = linear.into_inner();
}

#[cfg(not(feature = "compile_error"))]
#[cfg(any(debug_assertions, not(feature = "drop_unchecked")))]
#[test]
#[should_panic]
fn test_failed_destructure() {
    let _linear = Linear::new(123);
}
