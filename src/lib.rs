#![doc = include_str!("../README.md")]
#![warn(clippy::cargo_common_metadata)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::missing_panics_doc)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

use std::ops::Deref;
use std::ops::DerefMut;

/// A Cell like struct that wraps a T and can be derefernced to &T.  This cell must never be
/// dropped. For destruction of the inner value one has to call `.into_inner()`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Linear<T>(Option<T>);

impl<T> Linear<T> {
    /// Creates a new `Linear<T>` wrapping the supplied value.
    #[must_use]
    pub fn new(t: T) -> Self {
        Linear(Some(t))
    }

    /// Destructures a `Linear<T>` and returns the contained value.  This must eventually be
    /// called on any `Linear<T>`, failing to do so will panic.
    #[must_use]
    pub fn into_inner(mut self) -> T {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.take().unwrap_unchecked()
        }
    }
}

// Safety: since we do only a thin wrapper, just delegating Send+Sync is ok.
unsafe impl<T: Send> Send for Linear<T> {}
unsafe impl<T: Sync> Sync for Linear<T> {}

impl<T> Deref for Linear<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.as_ref().unwrap_unchecked()
        }
    }
}

impl<T> DerefMut for Linear<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY: A program will never see a `Linear<T>` that contains None because only
            // '.into_inner()' set it to 'None' while consuming 'self'.
            self.0.as_mut().unwrap_unchecked()
        }
    }
}

impl<T> Drop for Linear<T> {
    #[cfg_attr(feature = "compile_error", no_panic::no_panic)]
    fn drop(&mut self) {
        // Avoid double panic when we already panicking
        if self.0.is_some() && !std::thread::panicking() {
            panic!("linear type dropped")
        }
    }
}

#[test]
fn test_destructure() {
    let linear = Linear::new(123);
    let _ = linear.into_inner();
}

#[cfg(not(feature = "compile_error"))]
#[test]
#[should_panic]
fn test_failed_destructure() {
    let _linear = Linear::new(123);
}
