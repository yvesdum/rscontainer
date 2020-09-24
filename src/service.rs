//! The traits to be implemented on foreign types by users of this crate.

use crate::pointer::IPointer;
use crate::ServiceContainer;

/// An object that can be constructed through the service container.
///
/// Implement this trait on a type to be able to use it as a static service.
///
/// # Using foreign types as services
///
/// To use a foreign type as a service, follow these steps:
///
/// 1. Create a Zero Sized Type, for example `enum DatabaseService {}`.
/// 2. Implement `IService` on this type.
/// 3. Set the `Instance` associated type to the foreign type.
/// 4. Set the `Pointer` associated type to a smart pointer to the foreign
///    type, for example `Arc<Mutex<Database>>`.
/// 5. Implement `construct()` and `construct_singleton()` to construct the
///    foreign type.
/// 6. When you want to resolve the service, you use `Singleton<>` with your
///    Zero Sized Type, and you will get a pointer to the foreign type.
///
/// # Examples
///
/// Primary use case:
///
/// ```
/// use rscontainer::{ServiceContainer, IService, Singleton};
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// struct Account {
///     database: Singleton<Database>,
///     auth: Singleton<Authentication>
/// }
///
/// impl IService for Account {
///     type Instance = Self;
///     type Pointer = Rc<RefCell<Self>>;
///
///     fn construct(ctn: &mut ServiceContainer) -> Self::Instance {
///         Self {
///             database: ctn.resolve(),
///             auth: ctn.resolve()
///         }
///     }
///
///     fn construct_singleton(ctn: &mut ServiceContainer) -> Self::Pointer {
///         Rc::new(RefCell::new(Self::construct(ctn)))
///     }
/// }
///
/// // Use as follows:
/// let mut container = ServiceContainer::new();
/// let account: Singleton<Account> = container.resolve();
/// ```
///
/// Foreign services:
///
/// ```
/// use rscontainer::{ServiceContainer, IService};
/// use std::rc::Rc;
/// use std::cell::RefCell;
/// use diesel::pg::PgConnection;
///
/// enum PgConnectionService {}
/// impl IService for PgConnectionService {
///     type Instance = PgConnection;
///     type Pointer = Rc<RefCell<PgConnection>>;
///
///     fn construct(ctn: &mut ServiceContainer) -> Self::Instance {
///         PgConnection::establish()
///     }
///
///     fn construct_singleton(ctn: &mut ServiceContainer) -> Self::Pointer {
///         Rc::new(RefCell::new(Self::construct(ctn)))
///     }
/// }
///
/// // Use as follows:
/// let mut container = ServiceContainer::new();
/// let account: Singleton<PgConnectionService> = container.resolve();
/// ```
pub trait IService {
    /// The smart pointer to the service instance in case its a singleton.
    ///
    /// This can be:
    /// * `Rc<T>`
    /// * `Rc<RefCell<T>>`
    /// * `Arc<T>`
    /// * `Arc<Mutex<T>>`
    /// * `Arc<RwLock<T>>`
    type Pointer: IPointer + Clone;

    /// The type of the service in case its a non-shared instance.
    ///
    /// Typically this would be `Self`. However, it is also possible to return
    /// a completely different type. Therefore it is possible to use any
    /// existing type as a service.
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
