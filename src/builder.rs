//! Create a container with the builder pattern.

use crate::container::ServiceContainer;
use crate::getters::Global;
use crate::internal_helpers::{GlobalCtor, GlobalPtr, LocalCtor, TypeErasedService};
use crate::service_traits::{IGlobal, ILocal};
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

    /// Inserts a global instance.
    pub fn with_global<S: 'static + ?Sized + IGlobal>(&mut self, global: Global<S>) -> &mut Self {
        self.entry(TypeId::of::<S>()).global_ptr = Some(GlobalPtr::new(global.into_inner()));
        self
    }

    /// Sets a custom constructor for a global instance.
    pub fn with_global_constructor<S: 'static + ?Sized + IGlobal>(
        &mut self,
        ctor: GlobalCtor<S>,
    ) -> &mut Self {
        self.entry(TypeId::of::<S>()).global_ctor = Some(unsafe { std::mem::transmute(ctor) });
        self
    }

    /// Sets a custom constructor for a local instance.
    pub fn with_local_constructor<S: 'static + ?Sized + ILocal>(
        &mut self,
        ctor: LocalCtor<S>,
    ) -> &mut Self {
        self.entry(TypeId::of::<S>()).local_ctor = Some(unsafe { std::mem::transmute(ctor) });
        self
    }

    /// Sets custom contructors for a local and global intance.
    pub fn with_constructors<S: 'static + ?Sized + ILocal + IGlobal>(
        &mut self,
        local: LocalCtor<S>,
        global: GlobalCtor<S>,
    ) -> &mut Self {
        let mut entry = self.entry(TypeId::of::<S>());
        entry.global_ctor = Some(unsafe { std::mem::transmute(global) });
        entry.local_ctor = Some(unsafe { std::mem::transmute(local) });
        self
    }

    /// Builds the container.
    pub fn build(self) -> ServiceContainer {
        ServiceContainer::new_built(self.services)
    }
}
