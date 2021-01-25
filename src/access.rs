//! Access to the data of services.

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// Provides access to a singleton.
pub trait IAccess {
    /// The actual type of the instance.
    type Target;

    /// Tries to get access to the singleton through a closure.
    ///
    /// Returns `None` if the access failed, for example if the singleton is
    /// already locked or mutably borrowed.
    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U>;

    /// Get access to the singleton through a closure.
    ///
    /// Panics if the singleton is poisoned or already mutably borrowed.
    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.try_access(f).unwrap()
    }
}

/// Provides mutable access to a singleton.
pub trait IAccessMut: IAccess {
    /// Tries to get mutable access to the singleton through a closure.
    ///
    /// Returns `None` if the access failed, for example if the singleton is
    /// already locked or mutably borrowed.
    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U>;

    /// Get mutable access to the singleton through a closure.
    ///
    /// Panics if the singleton is poisoned or already mutably borrowed.
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.try_access_mut(f).unwrap()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Helper Types
///////////////////////////////////////////////////////////////////////////////

/// Wrapper to make a type accessable through the `IAccess` trait.
///
/// Note: this makes the type read-only.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Access<T>(T);

impl<T> Access<T> {
    /// Creates a new `Access` wrapper around some value.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Removes the `Access` wrapper and returns the original value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.0
    }
}

///////////////////////////////////////////////////////////////////////////////
// IAccess Implementations
///////////////////////////////////////////////////////////////////////////////

impl<T> IAccess for Access<T> {
    type Target = T;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(self.inner())
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        Some(self.access(f))
    }
}

impl<T> IAccess for RefCell<T> {
    type Target = T;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.borrow())
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow() {
            Ok(bor) => Some(f(&bor)),
            Err(..) => None
        }
    }
}

impl<T: Copy> IAccess for Cell<T> {
    type Target = T;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.get())
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        Some(self.access(f))
    }
}

impl<T> IAccess for Mutex<T> {
    type Target = T;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.lock().unwrap())
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(lock) => Some(f(&lock)),
            Err(..) => None
        }
    }
}

impl<T> IAccess for RwLock<T> {
    type Target = T;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.read().unwrap())
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_read() {
            Ok(read) => Some(f(&read)),
            Err(..) => None
        }
    }
}

impl<T: IAccess> IAccess for Rc<T> {
    type Target = T::Target;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }
}

impl<T: IAccess> IAccess for Arc<T> {
    type Target = T::Target;

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }

    fn try_access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// IAccessMut Implementations
///////////////////////////////////////////////////////////////////////////////

impl<T> IAccessMut for RefCell<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.borrow_mut())
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow_mut() {
            Ok(mut bor) => Some(f(&mut bor)),
            Err(..) => None
        }
    }
}

impl<T: Copy> IAccessMut for Cell<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        let mut value = self.get();
        let output = f(&mut value);
        self.set(value);
        output
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        Some(self.access_mut(f))
    }
}

impl<T> IAccessMut for Mutex<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.lock().unwrap())
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(mut lock) => Some(f(&mut lock)),
            Err(..) => None
        }
    }
}

impl<T> IAccessMut for RwLock<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.write().unwrap())
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        match self.try_write() {
            Ok(mut write) => Some(f(&mut write)),
            Err(..) => None
        }
    }
}

impl<T: IAccessMut> IAccessMut for Rc<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }
}

impl<T: IAccessMut> IAccessMut for Arc<T> {
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }

    fn try_access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }
}
