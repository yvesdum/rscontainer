//! Wrapper types to get and store services.

use super::access::{IAccess, IAccessMut, Poisoning};
use super::pointers::ISharedPointer;
use super::service_traits::{ILocal, IShared};
use std::fmt;
use std::ops::Deref;

///////////////////////////////////////////////////////////////////////////////
// Shared Instance
///////////////////////////////////////////////////////////////////////////////

/// A pointer to a shared instance from the service container.
#[repr(transparent)]
pub struct Shared<S: ?Sized + IShared> {
    /// The actual smart pointer to the shared instance.
    inner: S::Pointer,
}

impl<S: ?Sized + IShared> Shared<S> {
    /// Creates a shared instance from the inner smart pointer.
    pub fn new(inner: S::Pointer) -> Self {
        Self { inner }
    }

    /// Returns the inner smart pointer of the shared instance.
    pub fn into_inner(self) -> S::Pointer {
        self.inner
    }

    /// Returns a reference to the inner smart pointer.
    pub fn inner(&self) -> &S::Pointer {
        &self.inner
    }

    /// Returns a reference to the inner smart pointer.
    pub fn inner_mut(&mut self) -> &mut S::Pointer {
        &mut self.inner
    }

    /// Returns true if two shared instances point to the same instance.
    ///
    /// Only compares the pointers, not the contents of the shared instances,
    /// and is therefore always cheap.
    pub fn is(&self, other: &Self) -> bool {
        self.inner.ptr_eq(other.inner())
    }

    /// Get access to the shared instance through a closure.
    pub fn access<U, F>(&self, f: F) -> U
    where
        S::Pointer: IAccess,
        F: FnOnce(Poisoning<&<S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.access(f)
    }

    /// Get access to the shared instance through a closure.
    pub fn try_access<U, F>(&self, f: F) -> Option<U>
    where
        S::Pointer: IAccess,
        F: FnOnce(Poisoning<&<S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.try_access(f)
    }

    /// Get access to the shared instance through a closure.
    pub fn access_mut<U, F>(&self, f: F) -> U
    where
        S::Pointer: IAccessMut,
        F: FnOnce(Poisoning<&mut <S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.access_mut(f)
    }

    /// Get access to the shared instance through a closure.
    pub fn try_access_mut<U, F>(&self, f: F) -> Option<U>
    where
        S::Pointer: IAccessMut,
        F: FnOnce(Poisoning<&mut <S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.try_access_mut(f)
    }
}

impl<S: ?Sized + IShared> Deref for Shared<S>
where
    S::Pointer: Deref,
{
    type Target = <S::Pointer as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<S: ?Sized + IShared> Clone for Shared<S> {
    /// Clones the pointer to the shared instance.
    ///
    /// Only increases the reference count, so this is very cheap.
    /// See [`Rc::clone`] and [`Arc::clone`].
    ///
    /// [`Rc::clone`]: std::rc::Rc::clone
    /// [`Arc::clone`]: std::sync::Arc::clone
    fn clone(&self) -> Self {
        Shared {
            inner: self.inner.clone(),
        }
    }
}

impl<S: ?Sized + IShared> fmt::Debug for Shared<S>
where
    S::Pointer: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("inner", &self.inner)
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Any Kind Instance
///////////////////////////////////////////////////////////////////////////////

/// Some instance of a service, either a shared instance or local instance.
///
/// Use this as a field, when you want the user to decide whether they want to
/// supply a shared or local instance.
pub enum Instance<S: ?Sized + IShared + ILocal> {
    Shared(S::Pointer),
    Local(S::Instance),
}

impl<S: ?Sized + IShared + ILocal> Instance<S> {
    /// Creates an instance from a shared instance pointer.
    pub fn from_shared(inner: S::Pointer) -> Self {
        Self::Shared(inner)
    }

    /// Creates an instance from a local instance.
    pub fn from_local(inner: S::Instance) -> Self {
        Self::Local(inner)
    }

    /// Get access to the shared instance through a closure.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(Poisoning<&S::Instance>) -> U,
    {
        match self {
            Self::Shared(s) => s.access(accessor),
            Self::Local(l) => accessor(Poisoning::Healthy(l)),
        }
    }

    /// Get access to the shared instance through a closure.
    pub fn try_access<U, F>(&self, accessor: F) -> Option<U>
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(Poisoning<&S::Instance>) -> U,
    {
        match self {
            Self::Shared(s) => s.try_access(accessor),
            Self::Local(l) => Some(accessor(Poisoning::Healthy(l))),
        }
    }

    /// Get access to the shared instance through a closure.
    pub fn access_mut<U, F>(&mut self, accessor: F) -> U
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(Poisoning<&mut S::Instance>) -> U,
    {
        match self {
            Self::Shared(s) => s.access_mut(accessor),
            Self::Local(l) => accessor(Poisoning::Healthy(l)),
        }
    }

    /// Get access to the shared instance through a closure.
    pub fn try_access_mut<U, F>(&mut self, accessor: F) -> Option<U>
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(Poisoning<&mut S::Instance>) -> U,
    {
        match self {
            Self::Shared(s) => s.try_access_mut(accessor),
            Self::Local(l) => Some(accessor(Poisoning::Healthy(l))),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use crate::Access;
    use std::rc::Rc;

    #[test]
    fn shared_is() {
        let s1 = Shared::<u32>::new(Rc::new(Access::new(100)));
        let s2 = s1.clone();

        assert!(s1.is(&s2));
    }
}
