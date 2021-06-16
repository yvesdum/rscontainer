//! Traits for creating services.

use super::access::{Access, IAccess};
use super::container::ServiceContainer;
use super::getters::{Local, Shared};
use super::pointers::ISharedPointer;
use std::rc::Rc;

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// A type that can be used as a shared service.
pub trait IShared {
    /// The type of the smart pointer to the service. Supported by default:
    ///
    /// * `Rc<Access<T>>`
    /// * `Rc<Cell<T>>`
    /// * `Rc<RefCell<T>>`
    /// * `Arc<Access<T>>`
    /// * `Arc<Mutex<T>>`
    /// * `Arc<RwLock<T>>`
    ///
    /// Where `T` is equal to `Self::Target`.
    ///
    /// Use the [`Access`] wrapper if the type is read-only or already
    /// implements interior mutability.
    ///
    /// [`Access`]: crate::access::Access
    type Pointer: ISharedPointer + IAccess<Target = Self::Target>;

    /// The type that is used to access the shared instance.
    ///
    /// This should be the type that the pointer eventually dereferences to.
    type Target;

    /// The type of the error that can occur when constructing or resolving
    /// this service.
    type Error;

    /// Constructs an instance of the shared service.
    fn construct(ctn: &mut ServiceContainer) -> Result<Shared<Self>, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &Shared<Self>, _ctn: &mut ServiceContainer) {}
}

/// A type that can be used as a local service.
pub trait ILocal {
    /// The type of the local service.
    type Instance;

    /// Optional parameters for the `construct` method.
    type Parameters;

    /// The type of the error that can occur when constructing or resolving
    /// this service.
    type Error;

    /// Constructs an instance of the shared service.
    fn construct(
        ctn: &mut ServiceContainer,
        params: Self::Parameters,
    ) -> Result<Local<Self>, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &mut Self::Instance, _ctn: &mut ServiceContainer) {}
}

/// A service that can be used as both a local and shared instance.
pub trait IInstance: ILocal + IShared {}

///////////////////////////////////////////////////////////////////////////////
// Implementations
///////////////////////////////////////////////////////////////////////////////

/// IInstance is implemented for every service that implements `ILocal` and
/// `IGlobal`. `ILocal::Instance` must be the same as `IGlobal::Target`.
impl<S> IInstance for S where S: ILocal + IShared<Target = <S as ILocal>::Instance> {}

impl IShared for () {
    type Pointer = Rc<Access<()>>;
    type Target = ();
    type Error = ();

    fn construct(_: &mut ServiceContainer) -> Result<Shared<Self>, Self::Error> {
        Ok(Shared::new(Rc::new(Access::new(()))))
    }
}

impl ILocal for () {
    type Instance = ();
    type Parameters = ();
    type Error = ();

    fn construct(
        _: &mut ServiceContainer,
        _: Self::Parameters,
    ) -> Result<Local<Self>, Self::Error> {
        Ok(Local::new(()))
    }
}
