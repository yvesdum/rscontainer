//! Wrapper types to get and store services.

use super::access::{IAccess, IAccessMut, Poisoning};
use super::container::ServiceContainer;
use super::pointers::ISharedPointer;
use super::service_traits::{IShared, IInstance, ILocal};
use std::fmt;
use std::ops::{Deref, DerefMut};

///////////////////////////////////////////////////////////////////////////////
// Helper Traits
///////////////////////////////////////////////////////////////////////////////

/// A shared instance that can be resolved from the service container.
pub trait IResolveShared: Sized {
    type Error;

    /// Resolve the instance from the container.
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error>;
}

/// A local instance that can be resolved from the service container.
pub trait IResolveLocal: Sized {
    type Error;
    type Parameters;

    /// Resolve the instance from the container.
    fn resolve(ctn: &mut ServiceContainer, params: Self::Parameters) -> Result<Self, Self::Error>;
}

///////////////////////////////////////////////////////////////////////////////
// Shared Instance
///////////////////////////////////////////////////////////////////////////////

/// A pointer to a shared instance from the service container.
#[repr(transparent)]
pub struct Shared<S: ?Sized + IShared> {
    /// The actual smart pointer to the shared instance.
    inner: S::Pointer,
}

impl<S: 'static + ?Sized + IShared> IResolveShared for Shared<S> {
    type Error = S::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_shared()
    }
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
// Local Instance
///////////////////////////////////////////////////////////////////////////////

/// A local instance of a service.
#[repr(transparent)]
pub struct Local<S: ?Sized + ILocal> {
    /// The actual instance of the service.
    inner: S::Instance,
}

impl<S: 'static + ?Sized + ILocal> IResolveLocal for Local<S>
where
    S::Parameters: Default,
{
    type Error = S::Error;
    type Parameters = S::Parameters;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer, params: Self::Parameters) -> Result<Self, Self::Error> {
        ctn.resolve_local(params)
    }
}

impl<S: ?Sized + ILocal> Local<S> {
    /// Creates a local service from the inner instance.
    pub fn new(inner: S::Instance) -> Self {
        Self { inner }
    }

    /// Returns the inner instance of the local service.
    pub fn into_inner(self) -> S::Instance {
        self.inner
    }

    /// Returns a reference to the inner instance.
    pub fn inner(&self) -> &S::Instance {
        &self.inner
    }

    /// Returns a mutable reference to the inner instance.
    pub fn inner_mut(&mut self) -> &mut S::Instance {
        &mut self.inner
    }
}

impl<S: ?Sized + ILocal> Deref for Local<S> {
    type Target = S::Instance;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<S: ?Sized + ILocal> DerefMut for Local<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<S: ?Sized + ILocal> Clone for Local<S>
where
    S::Instance: Clone,
{
    /// Clones the instance of the service.
    ///
    /// This might be expensive, depending on the service.
    fn clone(&self) -> Self {
        Local {
            inner: self.inner.clone(),
        }
    }
}

impl<S: ?Sized + ILocal> fmt::Debug for Local<S>
where
    S::Instance: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Local").field("inner", &self.inner).finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Any Kind Instance
///////////////////////////////////////////////////////////////////////////////

/// Some instance of a service, either a shared instance or local instance.
///
/// Use this as a field, when you want the user to decide whether they want to
/// supply a shared or local instance.
pub enum Instance<S: ?Sized + IInstance> {
    Shared(Shared<S>),
    Local(Local<S>),
}

impl<S: 'static + ?Sized + IInstance> IResolveLocal for Instance<S>
where
    S::Parameters: Default,
{
    type Error = <S as ILocal>::Error;
    type Parameters = <S as ILocal>::Parameters;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer, params: Self::Parameters) -> Result<Self, Self::Error> {
        ctn.resolve_local(params).map(|s| Self::from_local(s))
    }
}

impl<S: 'static + ?Sized + IInstance> IResolveShared for Instance<S> {
    type Error = <S as IShared>::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_shared().map(|s| Self::from_shared(s))
    }
}

impl<S: ?Sized + IInstance> Instance<S> {
    /// Creates an instance from a shared instance pointer.
    pub fn from_shared(inner: Shared<S>) -> Self {
        Self::Shared(inner)
    }

    /// Creates an instance from a local instance.
    pub fn from_local(inner: Local<S>) -> Self {
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

impl<S: ?Sized + IInstance> From<Shared<S>> for Instance<S> {
    fn from(s: Shared<S>) -> Self {
        Self::from_shared(s)
    }
}

impl<S: ?Sized + IInstance> From<Local<S>> for Instance<S> {
    fn from(l: Local<S>) -> Self {
        Self::from_local(l)
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