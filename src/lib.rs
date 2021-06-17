//! Manager for dependencies between objects for the Rust language.
//!
//! # How to use
//!
//! The main type of this crate is the [`ServiceContainer`]. There are multiple
//! ways to initialize the container. See the documentation on ServiceContainer
//! for more information. The easiest way is with `new()`.
//!
//! ```rust
//! use rscontainer::ServiceContainer;
//! let mut container = ServiceContainer::new();
//! ```
//!
//! To configure the container, such as overriding the default constructors,
//! use the [`ContainerBuilder`]. 
//!
//! ```rust
//! use rscontainer::ServiceContainer;
//! let mut container = ServiceContainer::builder()
//!     .with_local_constructor::<u32>(|_resolver, params| Ok(params.value))
//!     .build();
//! ```
//!
//! ## Resolving instances
//!
//! There are different kind of instances:
//!
//! * **Local instances**: a fresh instance to be used in a local scope. This
//!   instance will not be stored in the service container, you will get a new
//!   instance each time you resolve a local instance. See 
//!   [`Resolver::local()`].
//! * **Shared instances**: an instance behind a smart pointer that is stored
//!   in the service container. You will get the same instance each time you
//!   resolve a shared service. See [`Resolver::shared()`] and [`Shared<T>`].
//!
//! To resolve instances, you first need to acquire a [`Resolver`].
//!
//! ```rust
//! let mut resolver = container.resolver();
//! ```
//!
//! To get an instance, you use one of the resolver methods. To resolve a 
//! **local instance**, use the [`Resolver::local()`] method. A local service 
//! can define parameters that need to be supplied to the `local()` method.
//!
//! ```rust
//! let params = (100, "hi!");
//! let mut local_service = resolver.resolve::<LocalService>(params).unwrap();
//! ```
//!
//! To resolve a **shared instance**, use the [`Resolver::shared()`] method.
//! The first time that this service is resolved, it will be contructed, stored
//! in the container and the pointer is returned. Every other time the service
//! is resolved that same pointer will be cloned and returned. Therefore it is
//! not possible to supply parameters.
//!
//! ```rust
//! let mut shared_service = resolver.shared::<SharedService>().unwrap();
//! ```
//!
//! ## Working with instances
//! 
//! A local instance is just a normal, owned instance, therefore you can do
//! with it whatever you want. But a shared instance is always behind a smart
//! pointer and a locking or borrowing mechanism. To use the instance, you need
//! to use one of the access methods: [`Shared::access()`], 
//! [`Shared::access_mut()`], [`Shared::try_access()`] and 
//! [`Shared::try_access_mut()`], which borrow or lock the instance for the
//! lifetime of the supplied closure. These access methods take into account
//! that the service may be poisoned. See [`Poisoning`] for more information.
//!
//! ```rust
//! let value = shared_service.access(|service| {
//!     let service = service.assert_healthy();
//!     service.get_value()
//! });
//! ```
//!
//! ## Using a type as a service
//!
//! To be able to resolve a type through the service container, there needs to
//! be an implementation of [`IShared`] and/or [`ILocal`] for it. These traits
//! define a constructor method. For a local service it will be called each
//! time it is resolved. For a shared service it will only be called the first
//! time.
//!
//! The constructors do not return `Self`, but rather an associated type
//! defined on the traits. This makes it possible to resolve every type through
//! the container without having to create a newtype wrapper.
//!
//! The constructors also receive a [`Resolver`], which can be used to 
//! recursively construct dependencies of the service. This is rscontainer's
//! implementation of *dependency injection*.
//!
//! # Example
//!
//! ```rust
//! use std::time::Instant;
//! use std::rc::Rc;
//! use std::cell::RefCell;
//! use rscontainer::{IShared, Resolver, ServiceContainer};
//!
//! struct InstantService;
//! impl IShared for InstantService {
//!     type Pointer = Rc<RefCell<Instant>>;
//!     type Instance = Instant;
//!     type Error = ();
//!
//!     fn construct(_: Resolver) -> Result<Rc<RefCell<Instant>>, ()> {
//!         Ok(Rc::new(RefCell::new(Instant::now())))
//!     }
//! }
//! 
//! fn main() {
//!     let mut container = ServiceContainer::new();
//!     let instant = container.resolver().shared::<InstantService>().unwrap();
//!     instant.access(|instant| {
//!         let instant = instant.assert_healthy();
//!         println!("{:?}", instant);
//!     });
//! }
//! ```

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