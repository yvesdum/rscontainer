//! Object dependency management for Rust.
//!
//! # Features
//!
//! * Automatic construction of objects which depend on other objects
//! * Dependency Injection
//! * Inversion Of Control
//! * Storage and management of Singletons
//! * Compatible with multiple smart pointer types, locking mechanisms and
//!   interiour mutability mechanisms.
//!
//! # Using the Service Container with static services
//!
//! **Singletons** are instances which are shared throughout your application.
//! Each time you resolve a singleton, you will get the same instance. A 
//! singleton is always behind a shared smart pointer, such as `Arc` or `Rc`,
//!  and may have an access mechanism such as `RefCell` or `Mutex`. Each 
//! service decides for themselve which kind of pointer and mechanism they use.
//! The first time you ask for an instance of a certain singleton, the 
//! container constructs it and recursively constructs and injects the 
//! neccessary dependencies. The instance is than stored in the container.
//!
//! To read from or mutate a singleton, you use the `read()` and `write()`
//! methods. This might lock the singleton, so immediately use the result
//! of these methods or keep the results in a scope that is as short as
//! possible.
//!
//! ```
//! use rscontainer::{ServiceContainer, Singleton};
//!
//! let mut container = ServiceContainer::new();
//! let singleton: Singleton<MyService> = container.resolve();
//!
//! singleton.write().set_something("something");
//! let something = singleton.read().get_something();
//! ```
//!
//! **Instances** are instances which are different each time you resolve them
//! from the container. They are not behind a pointer and access mechanism. 
//! The container will still take care of injecting the neccessary 
//! dependencies.
//!
//! Because instances are not behind a pointer, you don't need `read()` or
//! `write()` to interact with them. Instances implement `Deref` and 
//! `DerefMut` instead.
//!
//! ```
//! use rscontainer::{ServiceContainer, Instance};
//!
//! let mut container = ServiceContainer::new();
//! let instance: Instance<MyService> = container.resolve();
//! ```
//!
//! `Singleton<T>` and `Instance<T>` do not carry a lifetime parameter, 
//! therefore they can be stored in structs and enums very easily.
//!
//! # Creating a service
//!
//! To enable a type to be resolved through the service container, you need to
//! implement the `IService` trait on it. See the documentation of this trait
//! for more information.
//!
//! If your service depends on other services, you need to store these services
//! as fields in your struct as `Singleton<T>` or `Instance<T>`. 

mod dependency;
mod errors;
mod pointer;
mod read_write;
mod service;

pub use crate::dependency::{DynInstance, DynSingleton, Instance, Singleton};
// pub use crate::errors::{MakeError, ReadError, WriteError};
// pub use crate::read_write::{ReadService, WriteService};
pub use crate::service::{IDynService, IService};

use crate::dependency::IResolve;
use crate::pointer::ServicePointer;
use log::trace;
use std::any::type_name;
use std::any::TypeId;
use std::collections::HashMap;

//////////////////////////////////////////////////////////////////////////////
// Main Service Container
//////////////////////////////////////////////////////////////////////////////

/// A container for services.
///
/// Manages dependencies between these services.
pub struct ServiceContainer {
    singletons: HashMap<TypeId, ServicePointer>,
    dyn_singletons: HashMap<TypeId, ServicePointer>,
}

impl ServiceContainer {
    /// Creates a new, empty service container.
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            dyn_singletons: HashMap::new(),
        }
    }

    /// Creates a new service container with the specified reserved capacity.
    pub fn with_capacity(singletons: usize, dyn_singletons: usize) -> Self {
        Self {
            singletons: HashMap::with_capacity(singletons),
            dyn_singletons: HashMap::with_capacity(dyn_singletons),
        }
    }

    /// Resolve an object from the service container.
    ///
    /// This method can be used to resolve a service in any form, be it
    /// a singleton, instance, etc..
    #[inline]
    pub fn resolve<T>(&mut self) -> T
    where
        T: IResolve,
    {
        T::resolve(self)
    }

    /// Resolves or constructs a singleton.
    pub fn resolve_singleton<T>(&mut self) -> Singleton<T>
    where
        T: IService + 'static,
    {
        trace!("Resolving singleton {}", type_name::<T>());
        let key = TypeId::of::<T>();

        if let Some(service) = self.singletons.get(&key) {
            // If the key (which is the type id of `T`) exists, we can
            // guarantee that the service at this key has the same type as
            // `T`, so this is safe.
            let pointer = unsafe { service.as_pointer_unchecked() };
            return Singleton { pointer };
        }

        let pointer = T::construct_singleton(self);
        let service = ServicePointer::from_pointer(pointer.clone());
        self.singletons.insert(key, service);

        Singleton { pointer }
    }

    /// Constructs an instance through the service container.
    pub fn resolve_instance<T>(&mut self) -> Instance<T>
    where
        T: IService,
    {
        trace!("Resolving instance {}", type_name::<T>());
        let instance = T::construct(self);
        Instance { instance }
    }

    pub fn resolve_dyn_singleton<T>(&mut self) -> DynSingleton<T>
    where
        T: ?Sized + IDynService + 'static,
    {
        let key = TypeId::of::<T>();

        if let Some(_service) = self.dyn_singletons.get(&key) {
            unimplemented!()
        }

        panic!(
            "Resolve error: no implementor defined for dyn singleton {}",
            type_name::<T>()
        );
    }

    pub fn resolve_dyn_instance<T>(&mut self) -> DynInstance<T>
    where
        T: ?Sized + IDynService,
    {
        panic!(
            "Resolve error: no implementor defined for dyn instance {}",
            type_name::<T>()
        );
    }
}
