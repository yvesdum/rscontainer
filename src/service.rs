//! The IService trait.

use crate::ServiceContainer;
use crate::pointer::IPointer;

/// An object that can be resolved through the service container.
pub trait IService {
    /// The smart pointer to the service instance in case its a singleton.
    type Pointer: IPointer + Clone;

    /// The type of the service in case its a non-shared instance.
    type Instance;

    /// Creates an instance of this service.
    fn construct(ctn: &mut ServiceContainer) -> Self::Instance;

    /// Creates a singleton instance of this service.
    fn construct_singleton(ctn: &mut ServiceContainer) -> Self::Pointer;
}

/// A trait object that can be used as a dynamic service.
///
/// Implement this on the trait object itself with 
/// `impl IDynService for dyn X {}`.
pub trait IDynService {
    /// A ref counted smart pointer to this trait object.
    type Pointer: IPointer + Clone;

    /// A boxed instance of this trait object.
    type Instance;
}