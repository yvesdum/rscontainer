//! Container version 2.0

use crate::{ContainerBuilder, getters::{Shared, IResolveShared, IResolveLocal, Local}};
use crate::internal_helpers::{SharedCtor, SharedPtr, LocalCtor, TypeErasedService};
use crate::pointers::ISharedPointer;
use crate::service_traits::{IShared, ILocal};
use fnv::FnvHashMap;
use std::any::TypeId;

///////////////////////////////////////////////////////////////////////////////
// Container
///////////////////////////////////////////////////////////////////////////////

/// Container for all the services of an application.
#[derive(Debug, Default)]
pub struct ServiceContainer {
    /// The services in the container.
    services: FnvHashMap<TypeId, TypeErasedService>,
}

impl ServiceContainer {
    /// Creates a new service container.
    pub fn new() -> Self {
        ServiceContainer {
            services: FnvHashMap::default(),
        }
    }

    /// Creates a new service container with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ServiceContainer {
            services: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Creates a container that is already built by the ContainerBuilder.
    pub(crate) fn new_built(services: FnvHashMap<TypeId, TypeErasedService>) -> Self {
        Self { services }
    }

    /// Creates a ContainerBuilder.
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    /// Creates a ContainerBuilder with the specified capacity.
    pub fn builder_with_capcity(capacity: usize) -> ContainerBuilder {
        ContainerBuilder::with_capacity(capacity)
    }

    /// Returns the inner hashmap for testing purposes.
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn inner_hashmap(&self) -> &FnvHashMap<TypeId, TypeErasedService> {
        &self.services
    }

    /// Inserts a shared instance.
    ///
    /// Panics if the instance already exists, because it is not allowed to
    /// mutate the container after it is built.
    pub fn insert<S: 'static + ?Sized + IShared>(&mut self, instance: Shared<S>) {
        let entry = self.services.entry(TypeId::of::<S>()).or_default();
        assert!(entry.shared_ptr.is_none());
        entry.shared_ptr = Some(SharedPtr::new(instance.into_inner()));
    }

    ///////////////////////////////////////////////////////////////////////////
    // Generic Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a `Shared` or `Instance::Shared`.
    #[inline]
    pub fn shared<R: IResolveShared>(&mut self) -> Result<R, R::Error> {
        R::resolve(self)
    }

    /// Resolves a `Local` or `Instance::Local`.
    #[inline]
    pub fn local<R: IResolveLocal>(&mut self, params: R::Parameters) -> Result<R, R::Error> {
        R::resolve(self, params)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Specialized Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a shared instance.
    pub fn resolve_shared<S: 'static + ?Sized + IShared>(&mut self) -> Result<Shared<S>, S::Error> {
        let instance = match self.services.get(&TypeId::of::<S>()) {
            // There's an instance in the container, so we clone the smart pointer.
            Some(TypeErasedService {
                shared_ptr: Some(ptr),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                Shared::new(S::Pointer::clone_from_ptr(ptr.ptr))
            },

            // There's no instance, but there is a custom constructor.
            Some(TypeErasedService {
                shared_ctor: Some(ctor),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                let ctor: SharedCtor<S> = std::mem::transmute(*ctor);
                let instance = ctor(self)?;
                self.insert(instance.clone());
                instance
            },

            // There's no instance and no custom constructor, so use the
            // default constructor.
            _ => {
                let instance = S::construct(self)?;
                self.insert(instance.clone());
                instance
            }
        };

        S::resolved(&instance, self);
        Ok(instance)
    }

    /// Resolves a local instance.
    pub fn resolve_local<S: 'static + ?Sized + ILocal>(
        &mut self,
        params: S::Parameters,
    ) -> Result<Local<S>, S::Error> {
        let mut local = match self.services.get(&TypeId::of::<S>()) {
            // There is a custom constructor registered.
            Some(TypeErasedService {
                local_ctor: Some(ctor),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                let ctor: LocalCtor<S> = std::mem::transmute(*ctor);
                ctor(self, params)?
            },

            // There is no custom constructor, so use the default one.
            _ => S::construct(self, params)?,
        };
        S::resolved(&mut local, self);
        Ok(local)
    }
}
