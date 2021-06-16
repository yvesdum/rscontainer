//! Create a container with the builder pattern.

use crate::container::ServiceContainer;
use crate::getters::Shared;
use crate::internal_helpers::{SharedCtor, SharedPtr, LocalCtor, TypeErasedService};
use crate::service_traits::{IShared, ILocal};
use fnv::FnvHashMap;
use std::any::TypeId;

/// Create a container with the builder pattern.
pub struct ContainerBuilder {
    /// The services in the container.
    services: FnvHashMap<TypeId, TypeErasedService>,
}

impl ContainerBuilder {
    /// Creates a new ContainerBuilder.
    pub fn new() -> Self {
        Self {
            services: FnvHashMap::default(),
        }
    }

    /// Creates a new ContainerBuilder with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        ContainerBuilder {
            services: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }

    /// Returns an entry in the service container.
    fn entry(&mut self, key: TypeId) -> &mut TypeErasedService {
        self.services.entry(key).or_default()
    }

    /// Inserts a shared instance.
    pub fn with_shared<S: 'static + ?Sized + IShared>(mut self, shared: Shared<S>) -> Self {
        self.entry(TypeId::of::<S>()).shared_ptr = Some(SharedPtr::new(shared.into_inner()));
        self
    }

    /// Sets a custom constructor for a shared instance.
    pub fn with_shared_constructor<S: 'static + ?Sized + IShared>(
        mut self,
        ctor: SharedCtor<S>,
    ) -> Self {
        self.entry(TypeId::of::<S>()).shared_ctor = Some(unsafe { std::mem::transmute(ctor) });
        self
    }

    /// Sets a custom constructor for a local instance.
    pub fn with_local_constructor<S: 'static + ?Sized + ILocal>(
        mut self,
        ctor: LocalCtor<S>,
    ) -> Self {
        self.entry(TypeId::of::<S>()).local_ctor = Some(unsafe { std::mem::transmute(ctor) });
        self
    }

    /// Sets custom contructors for a local and shared intance.
    pub fn with_constructors<S: 'static + ?Sized + ILocal + IShared>(
        mut self,
        local: LocalCtor<S>,
        shared: SharedCtor<S>,
    ) -> Self {
        let mut entry = self.entry(TypeId::of::<S>());
        entry.shared_ctor = Some(unsafe { std::mem::transmute(shared) });
        entry.local_ctor = Some(unsafe { std::mem::transmute(local) });
        self
    }

    /// Builds the container.
    pub fn build(self) -> ServiceContainer {
        ServiceContainer::new_built(self.services)
    }
}
