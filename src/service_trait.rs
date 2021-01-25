//! Traits for creating services.

use super::access::{Access, IAccess};
use super::container::ServiceContainer;
use super::getters::{Local, Singleton};
use super::pointers::ISingletonPointer;
use std::rc::Rc;

///////////////////////////////////////////////////////////////////////////////
// Trait
///////////////////////////////////////////////////////////////////////////////

/// A type that can be used as a service.
pub trait IService {
    /// The type of the smart pointer that holds the singleton instance.
    ///
    /// This can be any of the following:
    ///
    /// * `Rc<Access<T>>` (see [`Access`])
    /// * `Rc<RefCell<T>>`
    /// * `Rc<Cell<T>>`
    /// * `Arc<Access<T>>`
    /// * `Arc<Mutex<T>>`
    /// * `Arc<RwLock<T>>`
    ///
    /// [`Access`]: crate::Access
    type Pointer: ISingletonPointer + IAccess;

    /// The type of a local instance of the service.
    type Instance;

    // /// The kind of instance that will be resolved when an [`Instance<S>`] of
    // /// this service is requested with the [`resolve`] method.
    // ///
    // /// This is the default behaviour, a user can always request a specific
    // /// kind of instance with the appropriate methods.
    // ///
    // /// This can be any of the following:
    // ///
    // /// * [`Singleton<Self>`]
    // /// * [`Local<Self>`]
    // ///
    // /// [`Instance<S>`]: crate::Instance
    // /// [`resolve`]: crate::ServiceContainer::resolve
    // /// [`Singleton<Self>`]: crate::Singleton
    // /// [`Local<Self>`]: crate::Local
    // type DefaultInstance: IResolve<Error = Self::Error> + IIntoInstance<Service = Self>;

    /// Parameters that users can supply when resolving a local instance.
    ///
    /// It is recommended that this implements [`Default`], but it is not
    /// required.
    ///
    /// [`Default`]: std::default::Default
    type Params;

    /// The type of the error that is returned when resolving this service.
    type Error;

    /// Creates a new singleton instance.
    fn new_singleton(ctn: &mut ServiceContainer) -> Result<Singleton<Self>, Self::Error>;

    /// Creates a new local instance.
    fn new_local(
        ctn: &mut ServiceContainer,
        params: Self::Params,
    ) -> Result<Local<Self>, Self::Error>;

    /// Called after the singleton instance is resolved from the container.
    ///
    /// Use this for example to inject cyclic dependencies.
    fn resolved_singleton(_this: &Singleton<Self>, _ctn: &mut ServiceContainer) {}

    /// Called after the local instance is resolved from the container.
    ///
    /// Use this for example to inject cyclic dependencies.
    fn resolved_local(_this: &mut Local<Self>, _ctn: &mut ServiceContainer) {}
}

///////////////////////////////////////////////////////////////////////////////
// Dummy Service
///////////////////////////////////////////////////////////////////////////////

impl IService for () {
    type Pointer = Rc<Access<()>>;
    type Instance = ();
    type Params = ();
    type Error = ();

    fn new_singleton(_ctn: &mut ServiceContainer) -> Result<Singleton<Self>, ()> {
        Ok(Singleton::new(Rc::new(Access::new(()))))
    }

    fn new_local(_ctn: &mut ServiceContainer, _params: Self::Params) -> Result<Local<Self>, ()> {
        Ok(Local::new(()))
    }
}