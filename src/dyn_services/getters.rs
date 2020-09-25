//! Types to resolve and store dynamic services.

use crate::dyn_services::service_traits::IDynService;
use crate::helpers::IResolve;
use crate::ServiceContainer;

/// A generalized smart pointer to a singleton trait object that is resolved
/// from the service container.
pub struct DynSingleton<T>
where
    T: ?Sized + IDynService,
{
    _pointer: T::SingletonPointer,
}

impl<T> IResolve for DynSingleton<T>
where
    T: ?Sized + IDynService + 'static,
{
    fn resolve(ctn: &mut ServiceContainer) -> Self {
        ctn.resolve_dyn_singleton()
    }
}

/// An instance of a service trait object that is resolved through the service
/// container.
pub struct DynInstance<T>
where
    T: ?Sized + IDynService,
{
    _pointer: T::InstancePointer,
}

impl<T> IResolve for DynInstance<T>
where
    T: ?Sized + IDynService + 'static,
{
    fn resolve(ctn: &mut ServiceContainer) -> Self {
        ctn.resolve_dyn_instance()
    }
}
