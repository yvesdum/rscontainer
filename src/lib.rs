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
//! # use rscontainer::{IOwned, Resolver};
//! # struct MyService(u32);
//! # impl IOwned for MyService {
//! #   type Instance = MyService;
//! #   type Parameters = u32;
//! #   type Error = ();
//! #   fn construct(_: Resolver, val: u32) -> Result<MyService, ()> {
//! #       Ok(MyService(val))
//! #   }
//! # }
//! use rscontainer::ServiceContainer;
//! let mut container = ServiceContainer::builder()
//!     .with_owned_constructor::<MyService>(|_resolver, value| {
//!         Ok(MyService(value))
//!     })
//!     .build();
//! ```
//!
//! ## Resolving instances
//!
//! There are different kind of instances:
//!
//! * **Owned instances**: a fresh instance to be used in an owned scope. This
//!   instance will not be stored in the service container, you will get a new
//!   instance each time you resolve an owned instance. See
//!   [`Resolver::owned()`].
//! * **Shared instances**: an instance behind a smart pointer that is stored
//!   in the service container. You will get the same instance each time you
//!   resolve a shared service. See [`Resolver::shared()`] and [`Shared<T>`].
//! * **Some instances**: an enum over owned and shared instances. Use this in a
//!   type when you want the user of your type to decide what kind of instance
//!   they want to supply. See [`Instance`], [`Resolver::shared_instance()`] and
//!   [`Resolver::owned_instance()`].
//!
//! To resolve instances, you first need to acquire a [`Resolver`].
//!
//! ```rust
//! # use rscontainer::ServiceContainer;
//! # let mut container = ServiceContainer::new();
//! let mut resolver = container.resolver();
//! ```
//!
//! To get an instance, you use one of the resolver methods. To resolve an
//! **owned instance**, use the [`Resolver::owned()`] method. An owned service
//! can define parameters that need to be supplied to the `owned()` method.
//!
//! ```rust
//! # use rscontainer::{IOwned, Resolver, ServiceContainer};
//! # struct MyService(u32);
//! # impl IOwned for MyService {
//! #   type Instance = MyService;
//! #   type Parameters = u32;
//! #   type Error = ();
//! #   fn construct(_: Resolver, val: u32) -> Result<MyService, ()> {
//! #       Ok(MyService(val))
//! #   }
//! # }
//! # fn main() -> Result<(), ()> {
//! # let mut container = ServiceContainer::new();
//! # let mut resolver = container.resolver();
//! let mut owned_service = resolver.owned::<MyService>(120)?;
//! # Ok(()) }
//! ```
//!
//! To resolve a **shared instance**, use the [`Resolver::shared()`] method.
//! The first time that this service is resolved, it will be contructed, stored
//! in the container and the pointer is returned. Every other time the service
//! is resolved that same pointer will be cloned and returned. Therefore it is
//! not possible to supply parameters.
//!
//! ```rust
//! # use rscontainer::{IShared, Resolver, ServiceContainer};
//! # use std::sync::{Arc, Mutex};
//! # struct MyService(u32);
//! # impl IShared for MyService {
//! #   type Pointer = Arc<Mutex<MyService>>;
//! #   type Target = MyService;
//! #   type Error = ();
//! #   fn construct(_: Resolver) -> Result<Arc<Mutex<MyService>>, ()> {
//! #       Ok(Arc::new(Mutex::new(MyService(543))))
//! #   }
//! # }
//! # fn main() -> Result<(), ()> {
//! # let mut container = ServiceContainer::new();
//! # let mut resolver = container.resolver();
//! let shared_service = resolver.shared::<MyService>()?;
//! # Ok(()) }
//! ```
//!
//! ## Working with instances
//!
//! An owned instance is just a normal, owned instance, therefore you can do
//! with it whatever you want. But a shared instance is always behind a smart
//! pointer and a locking or borrowing mechanism. To use the instance, you need
//! to use one of the access methods: [`Shared::access()`],
//! [`Shared::access_mut()`], [`Shared::try_access()`] and
//! [`Shared::try_access_mut()`], which borrow or lock the instance for the
//! lifetime of the supplied closure. These access methods take into account
//! that the service may be poisoned. See [`Poisoning`] for more information.
//!
//! ```rust
//! # use rscontainer::{IShared, Resolver, ServiceContainer};
//! # use std::sync::{Arc, Mutex};
//! # struct MyService(u32);
//! # impl MyService { fn get_value(&self) -> u32 { self.0 } }
//! # impl IShared for MyService {
//! #   type Pointer = Arc<Mutex<MyService>>;
//! #   type Target = MyService;
//! #   type Error = ();
//! #   fn construct(_: Resolver) -> Result<Arc<Mutex<MyService>>, ()> {
//! #       Ok(Arc::new(Mutex::new(MyService(543))))
//! #   }
//! # }
//! # fn main() -> Result<(), ()> {
//! # let mut container = ServiceContainer::new();
//! # let mut resolver = container.resolver();
//! # let shared_service = resolver.shared::<MyService>()?;
//! let value = shared_service.access(|service| {
//!     let service = service.assert_healthy();
//!     service.get_value()
//! });
//! # Ok(()) }
//! ```
//!
//! ## Using a type as a service
//!
//! To be able to resolve a type through the service container, there needs to
//! be an implementation of [`IShared`] and/or [`IOwned`] for it. These traits
//! define a constructor method. For an owned service it will be called each
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
//!     type Target = Instant;
//!     type Error = ();
//!
//!     fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
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
pub use self::service_traits::{IOwned, IShared};

/// Types for extending the functionality of rscontainer.
pub mod internals {
    pub use crate::access::{IAccess, IAccessMut};
    pub use crate::pointers::ISharedPointer;
}

///////////////////////////////////////////////////////////////////////////////
// Tests for README.md
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    #[test]
    fn readme_example() {
        use super::{IOwned, IShared, Resolver, ServiceContainer, Shared};
        use std::sync::{Arc, Mutex};
        use std::time::Instant;

        enum LogService {}

        impl IOwned for LogService {
            type Instance = Vec<Instant>;
            type Parameters = ();
            type Error = ();

            fn construct(_: Resolver, _: Self::Parameters) -> Result<Self::Instance, Self::Error> {
                Ok(Vec::new())
            }
        }

        struct Counter {
            value: u32,
            log: Vec<Instant>,
        }

        impl Counter {
            fn increase(&mut self) {
                self.value += 1;
                self.log.push(Instant::now());
            }
        }

        impl IShared for Counter {
            type Pointer = Arc<Mutex<Counter>>;
            type Target = Counter;
            type Error = ();

            fn construct(mut r: Resolver) -> Result<Self::Pointer, Self::Error> {
                Ok(Arc::new(Mutex::new(Counter {
                    value: 0,
                    log: r.owned::<LogService>(())?,
                })))
            }
        }

        fn main() -> Result<(), ()> {
            let mut container = ServiceContainer::new();

            // Initialize the counter service and recursively intialize an owned
            // instance of the log service and inject it in the counter service.
            let counter: Shared<Counter> = container.resolver().shared()?;

            counter.access_mut(|instance| {
                instance.assert_healthy().increase();
            });

            let timestamps = counter.access(|instance| {
                let counter = instance.assert_healthy();
                assert_eq!(counter.value, 1);
                counter.log.clone()
            });

            println!("Timestamps: {:?}", timestamps);

            Ok(())
        }

        main().unwrap();
    }
}
