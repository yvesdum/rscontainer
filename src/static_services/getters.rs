//! A generized smart pointer to a service.
//!
//! Requirements:
//! * Can hold a service or a pointer to a singleton.
//! * Provides read access to all services.
//! * Provides writing access to services that support it.
//! * Does not carry a lifetime.
//! * Can be used with concrete instances, eg `Dependency<Database>`.
//! * Can be used with trait objects, eg. `Dependency<dyn IDatabase>`.
//! * Can be cloned, so it can be pushed to other objects.
//! * Can be send across threads if the service is `Send` and `Sync`.

use crate::static_services::pointers::{IReadPointer, IWritePointer};
use crate::static_services::service_traits::IService;
use crate::helpers::IResolve;
use crate::ServiceContainer;
use std::ops::{Deref, DerefMut};

///////////////////////////////////////////////////////////////////////////////
// Concrete Singleton
///////////////////////////////////////////////////////////////////////////////

/// A generalized smart pointer to a singleton that is resolved from the service
/// container.
///
/// To read from a singleton, use the `read()` method. To mutate a singleton,
/// use the `write()` method.
#[derive(Debug)]
pub struct Singleton<T>
where
    T: IService,
{
    pub(crate) pointer: T::Pointer,
}

impl<T> IResolve for Singleton<T>
where
    T: IService + 'static,
{
    fn resolve(ctn: &mut ServiceContainer) -> Self {
        ctn.resolve_singleton()
    }
}

impl<'a, T> Singleton<T>
where
    T: IService,
    T::Pointer: IReadPointer<'a>,
{
    /// Acquire read-only access to the singleton.
    ///
    /// Depending on the smart pointer, this might lock the singleton. Use this
    /// method in a scope that is as small as possible.
    pub fn read(&'a self) -> <T::Pointer as IReadPointer>::ReadGuard {
        self.pointer.read()
    }
}

impl<'a, T> Singleton<T>
where
    T: IService,
    T::Pointer: IWritePointer<'a>,
{
    /// Acquire read/write access to the singleton.
    ///
    /// Depending on the smart pointer, this might lock the singleton. Use this
    /// method in a scope that is as small as possible.
    pub fn write(&'a self) -> <T::Pointer as IWritePointer>::WriteGuard {
        self.pointer.write()
    }
}

impl<T> Clone for Singleton<T>
where
    T: IService,
    T::Pointer: Clone,
{
    /// Clones the pointer to the singleton.
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Concrete Instance
///////////////////////////////////////////////////////////////////////////////

/// An instance of a service that is resolved from the service container.
///
/// Instance implements `Deref` and `DerefMut` to be able to read and mutate
/// the service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instance<T>
where
    T: IService,
{
    pub(crate) instance: T::Instance,
}

impl<T> IResolve for Instance<T>
where
    T: IService + 'static,
{
    fn resolve(ctn: &mut ServiceContainer) -> Self {
        ctn.resolve_instance()
    }
}

impl<T> Deref for Instance<T>
where
    T: IService,
{
    type Target = T::Instance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl<T> DerefMut for Instance<T>
where
    T: IService,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}