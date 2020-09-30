//! Types for internal functionality.

use crate::ServiceContainer;
use std::ptr::NonNull;

/// A custom contsructor that creates an instance or singleton.
pub(crate) type Constructor<T> = fn(&mut ServiceContainer) -> T;

/// Used for the `resolve()` method of the `ServiceContainer`.
pub trait IResolve {
    fn resolve(ctn: &mut ServiceContainer) -> Self;
}

/// Type-erased pointer to a service.
#[derive(Clone)]
pub(crate) struct SingletonPtr {
    /// The raw version of a ref counted smart pointer that implements 
    /// `IPointer`.
    pub ptr: NonNull<()>,
    /// The `drop_type_erased()` method of the `IPointer` trait implementation.
    pub dtor: unsafe fn(*const ()),
}

impl Drop for SingletonPtr {
    fn drop(&mut self) {
        unsafe { (self.dtor)(self.ptr.as_ptr()) }
    }
}

/// Custom constructors for a service.
pub(crate) struct Constructors {
    pub singleton: Option<Constructor<()>>,
    pub instance: Option<Constructor<()>>
}