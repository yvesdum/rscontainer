//! Traits for creating services.

use super::access::{Access, IAccess};
use super::pointers::ISharedPointer;
use crate::Resolver;
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
    fn construct(ctn: Resolver) -> Result<Self::Pointer, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &mut Self::Pointer, _ctn: Resolver) {}
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
    fn construct(ctn: Resolver, params: Self::Parameters) -> Result<Self::Instance, Self::Error>;

    /// Called each time after the service is resolved from the container.
    fn resolved(_this: &mut Self::Instance, _ctn: Resolver) {}
}

///////////////////////////////////////////////////////////////////////////////
// Implementations
///////////////////////////////////////////////////////////////////////////////

impl IShared for () {
    type Pointer = Rc<Access<()>>;
    type Target = ();
    type Error = ();

    fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
        Ok(Rc::new(Access::new(())))
    }
}

impl ILocal for () {
    type Instance = ();
    type Parameters = ();
    type Error = ();

    fn construct(_: Resolver, _: Self::Parameters) -> Result<Self, Self::Error> {
        Ok(())
    }
}
