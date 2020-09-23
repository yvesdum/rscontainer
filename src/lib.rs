//! Dependency Injection for Rust
//!
//! Requirements:
//! * Provides Inverion of Control.
//! * Services can be aliased.
//! * All foreign types can be used.

mod dependency;
mod errors;
mod pointer;
mod read_write;
mod service;

pub use crate::dependency::{DynInstance, DynSingleton, Instance, Singleton};
pub use crate::errors::{MakeError, ReadError, WriteError};
pub use crate::read_write::{ReadService, WriteService};
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
