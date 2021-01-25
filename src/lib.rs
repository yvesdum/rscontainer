// //! Object dependency management for Rust.
// //!
// //! # Features
// //!
// //! * Automatic construction of objects which depend on other objects
// //! * Dependency Injection
// //! * Storage and management of Singletons
// //! * Compatible with multiple smart pointer types, locking mechanisms and
// //!   interiour mutability mechanisms
// //! * No setup step required
// //! * Works with any existing type without writing complicated wrapper types
// //! * Optional registration of custom constructors
// //!
// //! # Creating a Service Container
// //!
// //! To create a service container without any configuration, use the
// //! [`ServiceContainer::new()`] method.
// //!
// //! ```rust
// //! use rscontainer::ServiceContainer;
// //!
// //! fn main() {
// //!     let mut container = ServiceContainer::new();
// //! }
// //! ```
// //!
// //! To register custom constructors, use the [`ContainerBuilder`].
// //!
// //! ```rust
// //! use rscontainer::ContainerBuilder;
// //!
// //! fn main() {
// //!     let mut builder = ContainerBuilder::new();
// //!
// //!     builder.constructors::<MyService>(
// //!         |container| {
// //!             MyService::new_instance(container)
// //!         },
// //!         |container| {
// //!             MyService::new_singleton(container)
// //!         }
// //!     );
// //!
// //!     let mut container = builder.build();
// //! }
// //! ```
// //!
// //! # Using the Service Container with static services
// //!
// //! **Singletons** are instances which are shared throughout your
// //! application. Each time you resolve a singleton, you will get the same
// //! instance. See [`Singleton<T>`] for more information.
// //!
// //! ```
// //! use rscontainer::{ServiceContainer, Singleton};
// //!
// //! let mut container = ServiceContainer::new();
// //!
// //! let singleton: Singleton<MyService> = container.resolve();
// //! singleton.write().set_something("something");
// //! let something = singleton.read().get_something();
// //! ```
// //!
// //! **Instances** are instances which are different each time you resolve
// //! them from the container. See [`Instance<T>`] for more information.
// //!
// //! ```
// //! use rscontainer::{ServiceContainer, Instance};
// //!
// //! let mut container = ServiceContainer::new();
// //!
// //! let mut instance: Instance<MyService> = container.resolve();
// //! instance.set_something("something");
// //! let something = instance.get_something();
// //! ```
// //!
// //! [`Singleton<T>`] and [`Instance<T>`] do not carry a lifetime parameter,
// //! therefore they can be stored in structs and enums very easily.
// //!
// //! # Enabling a type to be used as a Service
// //!
// //! To enable a type to be resolved through the service container, you need to
// //! implement the [`IService`] trait on it. With a simple trick it's possible
// //! to use any existing type as a service. See the documentation of this trait
// //! for more information.
// //!
// //! [`ServiceContainer::new()`]: crate::ServiceContainer::new
// //! [`ContainerBuilder`]: crate::ContainerBuilder
// //! [`Singleton<T>`]: crate::Singleton
// //! [`Instance<T>`]: crate::Instance
// //! [`IService`]: crate::IService

mod access;
mod container;
mod getters;
mod pointers;
mod service_trait;

pub use self::access::Access;
pub use self::container::ServiceContainer;
pub use self::getters::{Instance, Local, Singleton};
pub use self::service_trait::IService;

/// Types for extending the functionality of rscontainer.
pub mod internals {
    pub use crate::access::{IAccess, IAccessMut};
    pub use crate::getters::{IResolve, IResolveLocal, IResolveSingleton};
    pub use crate::pointers::ISingletonPointer;
}

#[cfg(test)]
mod tests;
