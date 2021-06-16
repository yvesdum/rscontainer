//! Wrapper types to get and store services.

use super::access::{IAccess, IAccessMut, Poisoning};
use super::container::ServiceContainer;
use super::pointers::IGlobalPointer;
use super::service_traits::{IGlobal, IInstance, ILocal};
use std::fmt;
use std::ops::{Deref, DerefMut};

///////////////////////////////////////////////////////////////////////////////
// Helper Traits
///////////////////////////////////////////////////////////////////////////////

/// A global instance that can be resolved from the service container.
pub trait IResolveGlobal: Sized {
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
// global instance Instance
///////////////////////////////////////////////////////////////////////////////

/// A pointer to a global instance from the service container.
#[repr(transparent)]
pub struct Global<S: ?Sized + IGlobal> {
    /// The actual smart pointer to the global instance instance.
    inner: S::Pointer,
}

impl<S: 'static + ?Sized + IGlobal> IResolveGlobal for Global<S> {
    type Error = S::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_global()
    }
}

impl<S: ?Sized + IGlobal> Global<S> {
    /// Creates a global instance from the inner smart pointer.
    pub fn new(inner: S::Pointer) -> Self {
        Self { inner }
    }

    /// Returns the inner smart pointer of the global instance.
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

    /// Returns true if two global instances point to the same instance.
    ///
    /// Only compares the pointers, not the contents of the global instances,
    /// and is therefore always cheap.
    pub fn is(&self, other: &Self) -> bool {
        self.inner.ptr_eq(other.inner())
    }

    /// Get access to the global instance through a closure.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess,
        F: FnOnce(&<S::Pointer as IAccess>::Target) -> U,
    {
        self.inner.access(accessor)
    }

    /// Get access to the global instance through a closure.
    pub fn access_poisoned<U, F>(&self, f: F) -> U
    where
        S::Pointer: IAccess,
        F: FnOnce(Poisoning<&<S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.access_poisoned(f)
    }

    /// Get access to the global instance through a closure.
    pub fn try_access<U, F>(&self, f: F) -> Option<U>
    where
        S::Pointer: IAccess,
        F: FnOnce(Poisoning<&<S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.try_access(f)
    }

    /// Get mutable access to the global instance.
    pub fn access_mut<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccessMut,
        F: FnOnce(&mut <S::Pointer as IAccess>::Target) -> U,
    {
        self.inner.access_mut(accessor)
    }

    /// Get access to the global instance through a closure.
    pub fn access_poisoned_mut<U, F>(&self, f: F) -> U
    where
        S::Pointer: IAccessMut,
        F: FnOnce(Poisoning<&mut <S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.access_poisoned_mut(f)
    }

    /// Get access to the global instance through a closure.
    pub fn try_access_mut<U, F>(&self, f: F) -> Option<U>
    where
        S::Pointer: IAccessMut,
        F: FnOnce(Poisoning<&mut <S::Pointer as IAccess>::Target>) -> U,
    {
        self.inner.try_access_mut(f)
    }
}

impl<S: ?Sized + IGlobal> Deref for Global<S>
where
    S::Pointer: Deref,
{
    type Target = <S::Pointer as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<S: ?Sized + IGlobal> Clone for Global<S> {
    /// Clones the pointer to the global instance instance.
    ///
    /// Only increases the reference count, so this is very cheap.
    /// See [`Rc::clone`] and [`Arc::clone`].
    ///
    /// [`Rc::clone`]: std::rc::Rc::clone
    /// [`Arc::clone`]: std::sync::Arc::clone
    fn clone(&self) -> Self {
        Global {
            inner: self.inner.clone(),
        }
    }
}

impl<S: ?Sized + IGlobal> fmt::Debug for Global<S>
where
    S::Pointer: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("global instance")
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

/// Some instance of a service, either a global instance or local instance.
///
/// Use this as a field, when you want the user to decide whether they want to
/// supply a global or local instance.
pub enum Instance<S: ?Sized + IInstance> {
    Global(Global<S>),
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

impl<S: 'static + ?Sized + IInstance> IResolveGlobal for Instance<S> {
    type Error = <S as IGlobal>::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_global().map(|s| Self::from_global(s))
    }
}

impl<S: ?Sized + IInstance> Instance<S> {
    /// Creates an instance from a global instance pointer.
    pub fn from_global(inner: Global<S>) -> Self {
        Self::Global(inner)
    }

    /// Creates an instance from a local instance.
    pub fn from_local(inner: Local<S>) -> Self {
        Self::Local(inner)
    }

    /// Get access to the service.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(&S::Instance) -> U,
    {
        match self {
            Self::Global(s) => s.access(accessor),
            Self::Local(l) => accessor(l),
        }
    }

    /// Get access to the global instance through a closure.
    pub fn access_poisoned<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(Poisoning<&S::Instance>) -> U,
    {
        match self {
            Self::Global(s) => s.access_poisoned(accessor),
            Self::Local(l) => accessor(Poisoning::Healthy(l)),
        }
    }

    /// Get access to the global instance through a closure.
    pub fn try_access<U, F>(&self, accessor: F) -> Option<U>
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(Poisoning<&S::Instance>) -> U,
    {
        match self {
            Self::Global(s) => s.try_access(accessor),
            Self::Local(l) => Some(accessor(Poisoning::Healthy(l))),
        }
    }

    /// Get mutable access to the service.
    pub fn access_mut<U, F>(&mut self, accessor: F) -> U
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(&mut S::Instance) -> U,
    {
        match self {
            Self::Global(s) => s.access_mut(accessor),
            Self::Local(l) => accessor(l),
        }
    }
    
    /// Get access to the global instance through a closure.
    pub fn access_poisoned_mut<U, F>(&mut self, accessor: F) -> U
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(Poisoning<&mut S::Instance>) -> U,
    {
        match self {
            Self::Global(s) => s.access_poisoned_mut(accessor),
            Self::Local(l) => accessor(Poisoning::Healthy(l)),
        }
    }

    /// Get access to the global instance through a closure.
    pub fn try_access_mut<U, F>(&mut self, accessor: F) -> Option<U>
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(Poisoning<&mut S::Instance>) -> U,
    {
        match self {
            Self::Global(s) => s.try_access_mut(accessor),
            Self::Local(l) => Some(accessor(Poisoning::Healthy(l))),
        }
    }
}

impl<S: ?Sized + IInstance> From<Global<S>> for Instance<S> {
    fn from(s: Global<S>) -> Self {
        Self::from_global(s)
    }
}

impl<S: ?Sized + IInstance> From<Local<S>> for Instance<S> {
    fn from(l: Local<S>) -> Self {
        Self::from_local(l)
    }
}
