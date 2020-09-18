//! A generized smart pointer to a service.

use crate::service::IService;

/// A generalized smart pointer to a service that is resolved from the service
/// container.
///
/// Use a `Dependency` if you want to store a pointer to a service for a longer
/// time, for example, as a field of a struct.
///
/// A `Dependency` can also be used to send services across threads, as long as
/// `T` and it's pointer type implements `Send` and `Sync`.
pub struct Dependency<T>
where
    T: ?Sized + IService
{
    /// The instance of the service.
    ///
    /// For a normal service, this is its `Instance` type. For a singleton,
    /// this is its `Pointer` type.
    instance: T::Instance
}

impl<T> Dependency<T>
where
    T: ?Sized + IService
{
    /// Create a new dependency.
    pub(crate) fn new(instance: T::Instance) -> Self {
        Self { instance }
    }
}