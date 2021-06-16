//! Object dependency management for Rust.
//!
//! # Features
//!
//! * Automatically construct objects and their dependencies recursively
//! * Multiple crates can resolve the same global objects
//! * Override default constructors to customize behaviour
//! * Get access to many objects while only copying one reference
//! * Inversion of Control without generic type parameters
//! * Setup is optional, not required

mod access;
mod builder;
mod container;
mod getters;
mod internal_helpers;
mod pointers;
mod resolver;
mod service_traits;

pub use self::access::{Access, Poisoning};
pub use self::builder::ContainerBuilder;
pub use self::container::ServiceContainer;
pub use self::getters::{Instance, Shared};
pub use self::resolver::Resolver;
pub use self::service_traits::{ILocal, IShared};

/// Types for extending the functionality of rscontainer.
pub mod internals {
    pub use crate::access::{IAccess, IAccessMut};
    pub use crate::pointers::ISharedPointer;
}

// #[cfg(test)]
// mod tests;
