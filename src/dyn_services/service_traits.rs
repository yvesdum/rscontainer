//! Traits that are neccessary to create dynamic services.

use crate::dyn_services::pointers::IDynSharedPointer;
use crate::ServiceContainer;

/// Defines a dynamic service.
///
/// This trait needs to be implemented on the trait object of the dynamic
/// service. 
pub trait IDynService {
    /// The type of the shared smart pointer in case the dynamic service is
    /// resolved as a singleton.
    type SingletonPointer: IDynSharedPointer + Clone;

    /// The type of the unique smart pointer in case the dynamic service is
    /// resolved as a local instance.
    type InstancePointer;
}

/// Defines an implementor of a dynamic service.
pub trait IDynImpl<TDynService> 
where
    TDynService: ?Sized + IDynService
{
    /// Constructs a singleton of the service.
    fn construct_singleton(ctn: &mut ServiceContainer) -> TDynService::SingletonPointer;

    /// Constructs a local instance of the service.
    fn construct(ctn: &mut ServiceContainer) -> TDynService::InstancePointer;
}

