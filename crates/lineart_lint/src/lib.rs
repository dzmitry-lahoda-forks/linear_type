//! Forbids
//! 
//! let Abc { a, .. } = abc; 
//! let Abc { a, .. } = abc; 
//! let Abc { a, b, c: _ } = abc; 
//! 
//! 
//! enables
//! 
//! #[deny(unused_variables)]
//! #[deny(clippy::rest_pat_in_fully_bound_structs)]
//! #[deny(clippy::unneeded_wildcard_pattern)]
//! #[deny(let_underscore_drop)] 
//! 