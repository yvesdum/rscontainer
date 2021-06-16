//! Access to the data of services.

use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock, TryLockError};

///////////////////////////////////////////////////////////////////////////////
// Poisoning Support
///////////////////////////////////////////////////////////////////////////////

/// Indicates whether an instance is poisoned or not.
///
/// More information about poisoning: 
/// [https://doc.rust-lang.org/nomicon/poisoning.html].
///
/// How to use this:
/// * For pointer types that don't support poisoning, use [`assert_healthy`].
/// * When it's a hard bug if the value is poisoned, use [`assert_healthy`].
/// * When poisoning status doesn't matter, use [`assume_healthy`].
/// * When you need different logic for poisoned or not, use a match statement.
pub enum Poisoning<S> {
    /// The instance is not poisoned, program flow can continue as usual.
    Healthy(S),
    /// The instance is poisoned, extra care should be taken when handling the
    /// value.
    Poisoned(S),
}

impl<S> Poisoning<S> {
    /// Returns the instance if it is not poisoned, panics if it is.
    #[track_caller]
    pub fn assert_healthy(self) -> S {
        match self {
            Self::Healthy(value) => value,
            Self::Poisoned(..) => panic!("Shared instance is poisoned"),
        }
    }

    /// Always returns the instance, whether it's poisoned or not.
    ///
    /// For pointer types that don't support poisoning, prefer 
    /// [`assert_healthy`], as this won't introduce hidden bugs when the 
    /// pointer type is changed at a later time.
    ///
    /// Only use this if you're certain that it doesn't matter if the value
    /// is poisoned.
    pub fn assume_healthy(self) -> S {
        match self {
            Self::Healthy(value) => value,
            Self::Poisoned(value) => value,
        }
    }

    /// Returns `true` if the instance is [`Healthy`].
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy(..))
    }

    /// Returns `true` if the instance is [`Poisoned`].
    pub const fn is_poisoned(&self) -> bool {
        matches!(self, Self::Poisoned(..))
    }
}

///////////////////////////////////////////////////////////////////////////////
// Traits
///////////////////////////////////////////////////////////////////////////////

/// Provides access to a shared instance.
pub trait IAccess {
    /// The actual type of the instance.
    type Target: ?Sized;

    /// Tries to get access to the shared instance through a closure.
    ///
    /// Returns `None` if the access failed, for example if the shared instance 
    /// is already locked or mutably borrowed.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U>;

    /// Get access to the shared instance through a closure.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U;
}

/// Provides mutable access to a shared instance.
pub trait IAccessMut: IAccess {
    /// Tries to get mutable access to the shared instance through a closure.
    ///
    /// Returns `None` if the access failed, for example if the shared instance is
    /// already locked or mutably borrowed.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U>;

    /// Get mutable access to the shared instance through a closure.
    ///
    /// The parameter of the closure contains the poisoning status of the
    /// instance.
    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U;
}

///////////////////////////////////////////////////////////////////////////////
// Helper Types
///////////////////////////////////////////////////////////////////////////////

/// Wrapper to make a type accessable through the `IAccess` trait.
///
/// Note: this makes the type read-only.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Access<T: ?Sized>(T);

impl<T> Access<T> {
    /// Creates a new `Access` wrapper around some value.
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }

    /// Removes the `Access` wrapper and returns the original value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns a reference to the inner value.
    pub const fn inner(&self) -> &T {
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

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(self.inner()))
    }
}

impl<T: ?Sized> IAccess for RefCell<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow() {
            Ok(bor) => Some(f(Poisoning::Healthy(&bor))),
            Err(..) => None,
        }
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&self.borrow()))
    }
}

impl<T: ?Sized + Copy> IAccess for Cell<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        Some(self.access(f))
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&self.get()))
    }
}

impl<T: ?Sized> IAccess for Mutex<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(lock) => Some(f(Poisoning::Healthy(&lock))),
            Err(TryLockError::Poisoned(lock)) => Some(f(Poisoning::Poisoned(&lock.into_inner()))),
            Err(..) => None,
        }
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        match self.lock() {
            Ok(lock) => f(Poisoning::Healthy(&lock)),
            Err(poison) => f(Poisoning::Poisoned(&poison.into_inner())),
        }
    }
}

impl<T: ?Sized> IAccess for RwLock<T> {
    type Target = T;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_read() {
            Ok(read) => Some(f(Poisoning::Healthy(&read))),
            Err(TryLockError::Poisoned(lock)) => Some(f(Poisoning::Poisoned(&lock.into_inner()))),
            Err(..) => None,
        }
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        match self.read() {
            Ok(read) => f(Poisoning::Healthy(&read)),
            Err(poison) => f(Poisoning::Poisoned(&poison.into_inner())),
        }
    }
}

impl<T: ?Sized + IAccess> IAccess for Rc<T> {
    type Target = T::Target;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }
}

impl<T: ?Sized + IAccess> IAccess for Arc<T> {
    type Target = T::Target;

    fn try_access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access(f)
    }

    fn access<U, F: FnOnce(Poisoning<&Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access(f)
    }
}

///////////////////////////////////////////////////////////////////////////////
// IAccessMut Implementations
///////////////////////////////////////////////////////////////////////////////

impl<T: ?Sized> IAccessMut for RefCell<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_borrow_mut() {
            Ok(mut bor) => Some(f(Poisoning::Healthy(&mut bor))),
            Err(..) => None,
        }
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        f(Poisoning::Healthy(&mut self.borrow_mut()))
    }
}

impl<T: ?Sized + Copy> IAccessMut for Cell<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        let mut value = self.get();
        let output = f(Poisoning::Healthy(&mut value));
        self.set(value);
        Some(output)
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        let mut value = self.get();
        let output = f(Poisoning::Healthy(&mut value));
        self.set(value);
        output
    }
}

impl<T: ?Sized> IAccessMut for Mutex<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_lock() {
            Ok(mut lock) => Some(f(Poisoning::Healthy(&mut lock))),
            Err(TryLockError::Poisoned(lock)) => {
                Some(f(Poisoning::Poisoned(&mut lock.into_inner())))
            }
            Err(..) => None,
        }
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        match self.lock() {
            Ok(mut lock) => f(Poisoning::Healthy(&mut lock)),
            Err(poison) => f(Poisoning::Poisoned(&mut poison.into_inner())),
        }
    }
}

impl<T: ?Sized> IAccessMut for RwLock<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        match self.try_write() {
            Ok(mut write) => Some(f(Poisoning::Healthy(&mut write))),
            Err(TryLockError::Poisoned(poison)) => {
                Some(f(Poisoning::Poisoned(&mut poison.into_inner())))
            }
            Err(..) => None,
        }
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        match self.write() {
            Ok(mut write) => f(Poisoning::Healthy(&mut write)),
            Err(poison) => f(Poisoning::Poisoned(&mut poison.into_inner())),
        }
    }
}

impl<T: ?Sized + IAccessMut> IAccessMut for Rc<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }
}

impl<T: ?Sized + IAccessMut> IAccessMut for Arc<T> {
    fn try_access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> Option<U> {
        self.deref().try_access_mut(f)
    }

    fn access_mut<U, F: FnOnce(Poisoning<&mut Self::Target>) -> U>(&self, f: F) -> U {
        self.deref().access_mut(f)
    }
}
