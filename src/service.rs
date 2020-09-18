//! The IService trait.

use crate::pointer::IPointer;
use crate::ServiceContainer;

/// An object that can be resolved through the service container.
pub trait IService {
    /// The type that is returned by the construct() method.
    type Instance;

    /// Creates a new instance of this service.
    fn construct(ctn: &mut ServiceContainer) -> Self::Instance;

    /// Stores the service in the container.
    ///
    /// Note: there is no use implementing this, because only singletons can be
    /// stored in the container and this method is automatically implemented
    /// for singletons.
    fn store(_me: Self::Instance, _ctn: &mut ServiceContainer) {}
}

/// An object that can be resolved through and stored in the service container.
///
/// A singleton is automatically a service.
pub trait ISingleton {
    /// The type of the smart pointer that contains the service.
    ///
    /// Pointer can be:
    /// * `Arc<T>`
    /// * `Arc<RwLock<T>>`
    /// * `Arc<Mutex<T>>`
    /// * `Rc<T>`
    /// * `Rc<RefCell<T>>`
    /// * `Rc<Cell<T>>`
    /// * A custom reference counted pointer that implements `IPointer`.
    type Pointer: IPointer;

    /// Creates a new instance of this service.
    fn construct(ctn: &mut ServiceContainer) -> Self::Pointer;
}

/// Every singleton is a service, so implement `IService` automatically.
impl<T: ISingleton> IService for T {
    type Instance = T::Pointer;

    fn construct(ctn: &mut ServiceContainer) -> Self::Instance {
        T::construct(ctn)
    }

    fn store(me: Self::Instance, ctn: &mut ServiceContainer) {
        ctn.store::<T>(me)
    }
}