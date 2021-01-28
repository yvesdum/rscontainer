//! Traits for creating services.

use super::access::{Access, IAccess};
use super::container::ServiceContainer;
use super::getters::{Global, Local};
use super::pointers::IGlobalPointer;
use std::rc::Rc;

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// A type that can be used as a global service.
pub trait IGlobal {
    /// The type of the smart pointer to the service.
    type Pointer: IGlobalPointer + IAccess<Target = Self::Access>;

    /// The type that is used to access the singleton.
    ///
    /// This should be the type that the pointer eventually dereferences to.
    type Access;

    /// The type of the error that can occur when constructing or resolving 
    /// this service.
    type Error;

    /// Constructs an instance of the global service.
    fn construct(ctn: &mut ServiceContainer) -> Result<Global<Self>, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &Global<Self>, _ctn: &mut ServiceContainer) {}
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

    /// Constructs an instance of the global service.
    fn construct(
        ctn: &mut ServiceContainer,
        params: Self::Parameters,
    ) -> Result<Local<Self>, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &mut Self::Instance, _ctn: &mut ServiceContainer) {}
}

/// A service that can be used as both a local and global instance.
pub trait IInstance: ILocal + IGlobal {}

///////////////////////////////////////////////////////////////////////////////
// Implementations
///////////////////////////////////////////////////////////////////////////////

/// IInstance is implemented for every service that implements `ILocal` and
/// `IGlobal`. `ILocal::Instance` must be the same as `IGlobal::Access`.
impl<S> IInstance for S where S: ILocal + IGlobal<Access = <S as ILocal>::Instance> {}

impl IGlobal for () {
    type Pointer = Rc<Access<()>>;
    type Access = ();
    type Error = ();

    fn construct(_: &mut ServiceContainer) -> Result<Global<Self>, Self::Error> {
        Ok(Global::new(Rc::new(Access::new(()))))
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
