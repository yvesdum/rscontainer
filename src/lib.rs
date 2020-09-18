//! Dependency Injection for Rust

mod dependency;
mod errors;
mod pointer;
mod read_write;
mod service;

pub use dependency::Dependency;
pub use errors::{MakeError, ReadError, WriteError};
pub use read_write::{ReadService, WriteService};
pub use service::{IService, ISingleton};

use std::any::TypeId;
use std::collections::HashMap;

///////////////////////////////////////////////////////////////////////////////
// Internal storage of a service in the container
///////////////////////////////////////////////////////////////////////////////

struct Service {
    type_id: TypeId,
    ptr: *const u8,
    dtor: unsafe fn(TypeId, *const u8),
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { (self.dtor)(self.type_id, self.ptr) }
    }
}

//////////////////////////////////////////////////////////////////////////////
// Main Service Container
//////////////////////////////////////////////////////////////////////////////

/// A container for services.
///
/// Manages dependencies between these services.
pub struct ServiceContainer {
    singletons: HashMap<TypeId, Service>
}

impl ServiceContainer {
    /// Creates a new, empty service container.
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new()
        }
    }

    /// Stores a singleton in the container.
    pub fn store<'a, T>(&'a mut self, pointer: T::Pointer)
    where
        T: ?Sized + ISingleton,
    {
        unimplemented!()
    }

    /// Creates a fresh instance of the specified service.
    pub fn fresh<'a, T>(&'a mut self) -> Dependency<T>
    where
        T: ?Sized + IService,
    {
        unimplemented!()
    }

    /// Acquire read-only access to the specified service.
    pub fn read<'a, T>(&'a mut self) -> Result<ReadService<'a, T>, ReadError>
    where
        T: ?Sized + IService,
    {
        unimplemented!()
    }

    /// Acquire read-write access to the specified service.
    pub fn write<'a, T>(&'a mut self) -> Result<WriteService<'a, T>, WriteError>
    where
        T: ?Sized + IService,
    {
        unimplemented!()
    }

    /// Makes a Dependency object for the specified service.
    pub fn make<T>(&mut self) -> Result<Dependency<T>, MakeError>
    where
        T: ?Sized + IService,
    {
        unimplemented!()
    }
}
