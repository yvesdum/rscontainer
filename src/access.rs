//! Access to the data of services.

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock, TryLockError};

///////////////////////////////////////////////////////////////////////////////
// Poisoning Support
///////////////////////////////////////////////////////////////////////////////

/// Indicates whether an instance is poisoned or not.
pub enum Poisoning<S> {
    /// The instance is not poisoned.
    Healthy(S),
    /// The instance is poisoned.
    Poisoned(S),
}

impl<S> Poisoning<S> {
    /// Returns the instance if it is not poisoned.
    ///
    /// Panics if the instance is poisoned.
    #[track_caller]
    pub fn assert_healthy(self) -> S {
        match self {
            Self::Healthy(value) => value,
            Self::Poisoned(..) => panic!("Global instance is poisoned"),
        }
    }

    /// Returns the instance in all cases without panicking.
    pub fn into_inner(self) -> S {
        match self {
            Self::Healthy(value) => value,
            Self::Poisoned(value) => value,
        }
    }

    /// Returns true if the instance is poisoned.
    pub fn is_poisoned(&self) -> bool {
        match self {
            Self::Healthy(..) => false,
            Self::Poisoned(..) => true
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// Provides access to a global instance.
pub trait IAccess {
    /// The actual type of the instance.
    type Target;

    /// Tries to get access to the global instance through a closure.
    ///
    /// Returns `None` if the access failed, for example if the global instance 
    /// is already locked or mutably borrowed.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U>;

    /// Get access to the global instance through a closure.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        self.try_access(f).unwrap()
    }

    /// Get access to the global instance through a closure.
    ///
    /// Panics if the global instance is poisoned or already mutably borrowed.
    #[track_caller]
    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.access_poisoned(|poisoned| f(poisoned.assert_healthy()))
    }
}

/// Provides mutable access to a global instance.
pub trait IAccessMut: IAccess {
    /// Tries to get mutable access to the global instance through a closure.
    ///
    /// Returns `None` if the access failed, for example if the global instance is
    /// already locked or mutably borrowed.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U>;

    /// Get mutable access to the global instance through a closure.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        self.try_access_mut(f).unwrap()
    }

    /// Get mutable access to the global instance through a closure.
    ///
    /// Panics if the global instance is poisoned or already mutably borrowed.
    #[track_caller]
    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.access_poisoned_mut(|poisoned| f(poisoned.assert_healthy()))
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

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        Some(f(Poisoning::Healthy(self.inner())))
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(self.inner()))
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(self.inner())
    }
}

impl<T> IAccess for RefCell<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow() {
            Ok(bor) => Some(f(Poisoning::Healthy(&bor))),
            Err(..) => None,
        }
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&self.borrow()))
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.borrow())
    }
}

impl<T: Copy> IAccess for Cell<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        Some(self.access_poisoned(f))
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&self.get()))
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.get())
    }
}

impl<T> IAccess for Mutex<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(lock) => Some(f(Poisoning::Healthy(&lock))),
            Err(TryLockError::Poisoned(lock)) => Some(f(Poisoning::Poisoned(&lock.into_inner()))),
            Err(..) => None,
        }
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        match self.lock() {
            Ok(lock) => f(Poisoning::Healthy(&lock)),
            Err(poison) => f(Poisoning::Poisoned(&poison.into_inner())),
        }
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.lock().unwrap())
    }
}

impl<T> IAccess for RwLock<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_read() {
            Ok(read) => Some(f(Poisoning::Healthy(&read))),
            Err(TryLockError::Poisoned(lock)) => Some(f(Poisoning::Poisoned(&lock.into_inner()))),
            Err(..) => None,
        }
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        match self.read() {
            Ok(read) => f(Poisoning::Healthy(&read)),
            Err(poison) => f(Poisoning::Poisoned(&poison.into_inner())),
        }
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        f(&self.read().unwrap())
    }
}

impl<T: IAccess> IAccess for Rc<T> {
    type Target = T::Target;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_poisoned(f)
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }
}

impl<T: IAccess> IAccess for Arc<T> {
    type Target = T::Target;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }

    fn access_poisoned<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_poisoned(f)
    }

    fn access<U, F: FnOnce(&Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// IAccessMut Implementations
///////////////////////////////////////////////////////////////////////////////

impl<T> IAccessMut for RefCell<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow_mut() {
            Ok(mut bor) => Some(f(Poisoning::Healthy(&mut bor))),
            Err(..) => None,
        }
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&mut self.borrow_mut()))
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.borrow_mut())
    }
}

impl<T: Copy> IAccessMut for Cell<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        let mut value = self.get();
        let output = f(Poisoning::Healthy(&mut value));
        self.set(value);
        Some(output)
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        let mut value = self.get();
        let output = f(Poisoning::Healthy(&mut value));
        self.set(value);
        output
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        let mut value = self.get();
        let output = f(&mut value);
        self.set(value);
        output
    }
}

impl<T> IAccessMut for Mutex<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(mut lock) => Some(f(Poisoning::Healthy(&mut lock))),
            Err(TryLockError::Poisoned(lock)) => {
                Some(f(Poisoning::Poisoned(&mut lock.into_inner())))
            }
            Err(..) => None,
        }
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        match self.lock() {
            Ok(mut lock) => f(Poisoning::Healthy(&mut lock)),
            Err(poison) => f(Poisoning::Poisoned(&mut poison.into_inner())),
        }
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.lock().unwrap())
    }
}

impl<T> IAccessMut for RwLock<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_write() {
            Ok(mut write) => Some(f(Poisoning::Healthy(&mut write))),
            Err(TryLockError::Poisoned(poison)) => {
                Some(f(Poisoning::Poisoned(&mut poison.into_inner())))
            }
            Err(..) => None,
        }
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        match self.write() {
            Ok(mut write) => f(Poisoning::Healthy(&mut write)),
            Err(poison) => f(Poisoning::Poisoned(&mut poison.into_inner())),
        }
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        f(&mut self.write().unwrap())
    }
}

impl<T: IAccessMut> IAccessMut for Rc<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_poisoned_mut(f)
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }
}

impl<T: IAccessMut> IAccessMut for Arc<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }

    fn access_poisoned_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_poisoned_mut(f)
    }

    fn access_mut<U, F: FnOnce(&mut Self::Target) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }
}
