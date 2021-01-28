//! Internal storage helpers.

use crate::container::ServiceContainer;
use crate::getters::{Global, Local};
use crate::pointers::IGlobalPointer;
use crate::service_traits::{IGlobal, ILocal};
use std::fmt;
use std::ptr::NonNull;

/// A raw pointer to a singleton instance with drop logic.
/// This is a type-erased `Rc` or `Arc` that implements `ISingletonPointer`.
#[derive(Debug)]
pub(crate) struct GlobalPtr {
    pub ptr: NonNull<()>,
    dtor: unsafe fn(NonNull<()>),
}

impl Drop for GlobalPtr {
    fn drop(&mut self) {
        #[cfg(test)]
        println!("Dropping SingletonPtr {:p}", self);

        unsafe { (self.dtor)(self.ptr) }
    }
}

impl GlobalPtr {
    pub fn new<P: IGlobalPointer>(instance: P) -> Self {
        GlobalPtr {
            ptr: unsafe { instance.into_ptr() },
            dtor: P::drop_from_ptr,
        }
    }
}

/// A custom constructor for a global instance.
pub(crate) type GlobalCtor<S> =
    fn(&mut ServiceContainer) -> Result<Global<S>, <S as IGlobal>::Error>;

/// A custom constructor for a local instance.
pub(crate) type LocalCtor<S> =
    fn(&mut ServiceContainer, <S as ILocal>::Parameters) -> Result<Local<S>, <S as ILocal>::Error>;

/// A service in the container that is type erased.
#[derive(Default)]
pub(crate) struct TypeErasedService {
    /// A raw pointer to the global instance.
    pub global_ptr: Option<GlobalPtr>,
    /// Custom constructor for a global instance.
    pub global_ctor: Option<GlobalCtor<()>>,
    /// Custom constructor for a local instance.
    pub local_ctor: Option<LocalCtor<()>>,
}

impl fmt::Debug for TypeErasedService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeErasedService")
            .field("global_ptr", &self.global_ptr)
            .field("global_ctor", &self.global_ctor.is_some())
            .field("local_ctor", &self.local_ctor.is_some())
            .finish()
    }
}
