//! Internal storage helpers.

use crate::container::ServiceContainer;
use crate::getters::{Shared, Local};
use crate::pointers::ISharedPointer;
use crate::service_traits::{IShared, ILocal};
use std::fmt;
use std::ptr::NonNull;

/// A raw pointer to a shared instance with drop logic.
/// This is a type-erased `Rc` or `Arc` that implements `ISharedPointer`.
#[derive(Debug)]
pub(crate) struct SharedPtr {
    pub ptr: NonNull<()>,
    dtor: unsafe fn(NonNull<()>),
}

impl Drop for SharedPtr {
    fn drop(&mut self) {
        unsafe { (self.dtor)(self.ptr) }
    }
}

impl SharedPtr {
    pub fn new<P: ISharedPointer>(instance: P) -> Self {
        SharedPtr {
            ptr: unsafe { instance.into_ptr() },
            dtor: P::drop_from_ptr,
        }
    }
}

/// A custom constructor for a shared instance.
pub(crate) type SharedCtor<S> =
    fn(&mut ServiceContainer) -> Result<Shared<S>, <S as IShared>::Error>;

/// A custom constructor for a local instance.
pub(crate) type LocalCtor<S> =
    fn(&mut ServiceContainer, <S as ILocal>::Parameters) -> Result<Local<S>, <S as ILocal>::Error>;

/// A service in the container that is type erased.
#[derive(Default)]
pub(crate) struct TypeErasedService {
    /// A raw pointer to the shared instance.
    pub shared_ptr: Option<SharedPtr>,
    /// Custom constructor for a shared instance.
    pub shared_ctor: Option<SharedCtor<()>>,
    /// Custom constructor for a local instance.
    pub local_ctor: Option<LocalCtor<()>>,
}

impl fmt::Debug for TypeErasedService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeErasedService")
            .field("shared_ptr", &self.shared_ptr)
            .field("shared_ctor", &self.shared_ctor.is_some())
            .field("local_ctor", &self.local_ctor.is_some())
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn shared_ptr_new() {
        let thing = Rc::new(100);
        let thing_clone = Rc::clone(&thing);
        let ptr = SharedPtr::new(thing);
        assert_eq!(Rc::strong_count(&thing_clone), 2);
        assert_eq!(
            Rc::as_ptr(&thing_clone) as *const (),
            ptr.ptr.as_ptr() as *const ()
        );
    }

    #[test]
    fn shared_ptr_drop() {
        let thing = Rc::new(100);
        let thing_clone = Rc::clone(&thing);
        let ptr = SharedPtr::new(thing);
        drop(ptr);
        assert_eq!(Rc::strong_count(&thing_clone), 1);
    }
}