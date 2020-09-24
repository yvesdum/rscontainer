//! The traits to be implemented on foreign types by users of this crate.

use crate::ServiceContainer;
use crate::pointer::IPointer;

/// An object that can be constructed through the service container.
///
/// Implement this trait on a type to be able to use it as a static service.
///
/// # Examples
///
/// ```
/// use rscontainer::{ServiceContainer, IService};
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
    /// Typically this would be `Self`.
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