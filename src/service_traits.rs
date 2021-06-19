//! Traits for creating services.

use super::access::{Access, IAccess};
use super::pointers::ISharedPointer;
use crate::Resolver;
use std::rc::Rc;

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// A type that can be used as a shared service.
///
/// # Examples
///
/// Using a type from the same crate as a shared service:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IShared, Resolver};
///
/// struct MyService(u32);
///
/// impl IShared for MyService {
///     type Pointer = Arc<Mutex<Self>>;
///     type Target = Self;
///     type Error = ();
///
///     fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
///         Ok(Arc::new(Mutex::new(MyService(123))))
///     }
/// }
/// ```
///
/// Using a type from another crate as a shared service without a newtype
/// wrapper:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IShared, Resolver};
///
/// enum VecService {}
///
/// impl IShared for VecService {
///     // Note that `Pointer` dereferences to a different type than `Self` ..
///     type Pointer = Arc<Mutex<Vec<u32>>>;
///     type Target = Vec<u32>;
///     type Error = ();
///
///     // ... and that `construct` returns `Self::Pointer` instead of `Self`.
///     fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
///         Ok(Arc::new(Mutex::new(vec![1, 2, 3])))
///     }
/// }
/// ```
///
/// Using a type as a service, that has a dependency on another service:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IShared, Resolver, Shared};
///
/// enum VecService {}
/// impl IShared for VecService {
/// # type Pointer = Arc<Mutex<Vec<u32>>>; type Target = Vec<u32>; type Error = ();
/// # fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
/// #     Ok(Arc::new(Mutex::new(vec![1, 2, 3])))
/// # }
///     // ...
/// }
///
/// struct MyService {
///     numbers: Shared<VecService>,
/// };
///
/// impl IShared for MyService {
///     type Pointer = Arc<Mutex<Self>>;
///     type Target = Self;
///     type Error = ();
///
///     fn construct(mut resolver: Resolver) -> Result<Self::Pointer, Self::Error> {
///         let this = MyService {
///             numbers: resolver.shared()?,
///         };
///         Ok(Arc::new(Mutex::new(this)))
///     }
/// }
/// ```
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

/// A type that can be used as an owned service.
///
/// # Examples
///
/// Using a type from the same crate as an owned service:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IOwned, Resolver};
///
/// struct MyService(u32);
///
/// impl IOwned for MyService {
///     type Instance = Self;
///     type Parameters = u32;
///     type Error = ();
///
///     fn construct(
///         _: Resolver, 
///         value: Self::Parameters
///     ) -> Result<Self::Instance, Self::Error> {
///         Ok(MyService(value))
///     }
/// }
/// ```
///
/// Using a type from a different crate as an owned service:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IOwned, Resolver};
///
/// enum VecService {}
///
/// impl IOwned for VecService {
///     // Note that `Instance` is a different type than `Self` ..
///     type Instance = Vec<u32>;
///     type Parameters = (u32, usize);
///     type Error = ();
///
///     // .. and that `construct` returns `Self::Instance` instead of `Self`.
///     fn construct(
///         _: Resolver, 
///         (value, repeat): Self::Parameters
///     ) -> Result<Self::Instance, Self::Error> {
///         Ok(vec![value; repeat])
///     }
/// }
/// ```
///
/// Using a type with a dependency as an owned service:
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use rscontainer::{IOwned, Resolver};
///
/// enum VecService {}
/// impl IOwned for VecService {
/// # type Instance = Vec<u32>; type Parameters = (u32, usize); type Error = ();
/// # fn construct(_: Resolver, p: Self::Parameters) -> Result<Self::Instance, Self::Error> {
/// #     Ok(vec![p.0; p.1])
/// # }
///     // ...
/// }
///
/// struct MyService {
///     numbers: Vec<u32>,
/// }
///
/// impl IOwned for MyService {
///     type Instance = Self;
///     type Parameters = u32;
///     type Error = ();
///
///     fn construct(
///         mut resolver: Resolver, 
///         value: Self::Parameters
///     ) -> Result<Self::Instance, Self::Error> {
///         Ok(MyService {
///             numbers: resolver.owned::<VecService>((value, 4))?,
///         })
///     }
/// }
/// ```

pub trait IOwned {
    /// The type of the owned service.
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

impl IOwned for () {
    type Instance = ();
    type Parameters = ();
    type Error = ();

    fn construct(_: Resolver, _: Self::Parameters) -> Result<Self, Self::Error> {
        Ok(())
    }
}
