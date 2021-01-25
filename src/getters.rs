//! Wrapper types to get and store services.

use super::access::{IAccess, IAccessMut};
use super::container::ServiceContainer;
use super::pointers::ISingletonPointer;
use super::service_trait::IService;
use std::fmt;
use std::ops::{Deref, DerefMut};

///////////////////////////////////////////////////////////////////////////////
// Helper Trait
///////////////////////////////////////////////////////////////////////////////

/// An instance that can be resolved from the service container.
pub trait IResolve: Sized {
    type Error;

    /// Resolve the instance from the container.
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error>;
}

/// A singleton instance that can be resolved from the service container.
pub trait IResolveSingleton: IResolve {
    /// Resolve the instance from the container.
    fn resolve_singleton(ctn: &mut ServiceContainer) -> Result<Self, Self::Error>;
}

/// A local instance that can be resolved from the service container.
pub trait IResolveLocal: IResolve {
    /// Resolve the instance from the container.
    fn resolve_local(ctn: &mut ServiceContainer) -> Result<Self, Self::Error>;
}

///////////////////////////////////////////////////////////////////////////////
// Singleton Instance
///////////////////////////////////////////////////////////////////////////////

/// A pointer to a singleton from the service container.
#[repr(transparent)]
pub struct Singleton<S: ?Sized + IService> {
    /// The actual smart pointer to the singleton instance.
    inner: S::Pointer,
}

impl<S: 'static + IService> IResolve for Singleton<S> {
    type Error = S::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_singleton()
    }
}

impl<S: 'static + IService> IResolveSingleton for Singleton<S> {
    #[inline]
    fn resolve_singleton(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_singleton()
    }
}

impl<S: IService> Singleton<S> {
    /// Creates a singleton from the inner smart pointer.
    pub fn new(inner: S::Pointer) -> Self {
        Self { inner }
    }

    /// Returns the inner smart pointer of the singleton.
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

    /// Returns true if two singletons point to the same instance.
    ///
    /// Only compares the pointers, not the contents of the singletons,
    /// and is therefore always cheap.
    pub fn is(&self, other: &Self) -> bool {
        self.inner.ptr_eq(other.inner())
    }

    /// Get access to the singleton.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess,
        F: FnOnce(&<S::Pointer as IAccess>::Target) -> U,
    {
        self.inner.access(accessor)
    }

    /// Get mutable access to the singleton.
    pub fn access_mut<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccessMut,
        F: FnOnce(&mut <S::Pointer as IAccess>::Target) -> U,
    {
        self.inner.access_mut(accessor)
    }
}

impl<S: IService> Deref for Singleton<S>
where
    S::Pointer: Deref,
{
    type Target = <S::Pointer as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<S: IService> Clone for Singleton<S> {
    /// Clones the pointer to the singleton instance.
    ///
    /// Only increases the reference count, so this is very cheap.
    /// See [`Rc::clone`] and [`Arc::clone`].
    ///
    /// [`Rc::clone`]: std::rc::Rc::clone
    /// [`Arc::clone`]: std::sync::Arc::clone
    fn clone(&self) -> Self {
        Singleton {
            inner: self.inner.clone(),
        }
    }
}

impl<S: IService> fmt::Debug for Singleton<S>
where
    S::Pointer: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Singleton")
            .field("inner", &self.inner)
            .finish()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Local Instance
///////////////////////////////////////////////////////////////////////////////

/// A local instance of a service.
#[repr(transparent)]
pub struct Local<S: ?Sized + IService> {
    /// The actual instance of the service.
    inner: S::Instance,
}

impl<S> IResolve for Local<S>
where
    S: 'static + IService,
    S::Params: Default,
{
    type Error = S::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_local_default()
    }
}

impl<S> IResolveLocal for Local<S>
where
    S: 'static + IService,
    S::Params: Default,
 {
    #[inline]
    fn resolve_local(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_local_default()
    }
}

impl<S: IService> Local<S> {
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

    /// Get access to the local instance.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        F: FnOnce(&S::Instance) -> U,
    {
        accessor(&self.inner)
    }

    /// Get mutable access to the local instance.
    pub fn access_mut<U, F>(&mut self, accessor: F) -> U
    where
        F: FnOnce(&mut S::Instance) -> U,
    {
        accessor(&mut self.inner)
    }
}

impl<S: IService> Deref for Local<S> {
    type Target = S::Instance;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<S: IService> DerefMut for Local<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<S: IService> Clone for Local<S>
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

impl<S: IService> fmt::Debug for Local<S>
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

/// Some instance of a service, either a singleton or local instance.
pub enum Instance<S: ?Sized + IService> {
    Singleton(S::Pointer),
    Local(S::Instance),
}

impl<S: 'static + IService> IResolve for Instance<S> {
    type Error = S::Error;

    #[inline]
    fn resolve(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_singleton().map(|s| Self::from_singleton(s))
    }
}

impl<S> IResolveLocal for Instance<S>
where
    S: 'static + IService,
    S::Params: Default,
 {
    #[inline]
    fn resolve_local(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_local_default().map(|l| Self::from_local(l))
    }
}

impl<S: 'static + IService> IResolveSingleton for Instance<S> {
    #[inline]
    fn resolve_singleton(ctn: &mut ServiceContainer) -> Result<Self, Self::Error> {
        ctn.resolve_singleton().map(|s| Self::from_singleton(s))
    }
}

impl<S: IService> Instance<S> {
    /// Creates an instance from a singleton pointer.
    pub fn from_singleton(inner: Singleton<S>) -> Self {
        Self::Singleton(inner.into_inner())
    }

    /// Creates an instance from a local instance.
    pub fn from_local(inner: Local<S>) -> Self {
        Self::Local(inner.into_inner())
    }

    /// Get access to the service.
    pub fn access<U, F>(&self, accessor: F) -> U
    where
        S::Pointer: IAccess<Target = S::Instance>,
        F: FnOnce(&S::Instance) -> U,
    {
        match self {
            Self::Singleton(s) => s.access(accessor),
            Self::Local(l) => accessor(l),
        }
    }

    /// Get mutable access to the service.
    pub fn access_mut<U, F>(&mut self, accessor: F) -> U
    where
        S::Pointer: IAccessMut<Target = S::Instance>,
        F: FnOnce(&mut S::Instance) -> U,
    {
        match self {
            Self::Singleton(s) => s.access_mut(accessor),
            Self::Local(l) => accessor(l),
        }
    }
}

impl<S: IService> From<Singleton<S>> for Instance<S> {
    fn from(s: Singleton<S>) -> Self {
        Self::from_singleton(s)
    }
}

impl<S: IService> From<Local<S>> for Instance<S> {
    fn from(l: Local<S>) -> Self {
        Self::from_local(l)
    }
}
