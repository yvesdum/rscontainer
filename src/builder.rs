//! Support for custom constructors.

use crate::helpers::{Constructor, Constructors};
use crate::static_services::service_traits::IService;
use crate::ServiceContainer;
use std::any::TypeId;
use std::collections::HashMap;

/// Constructs a Service Container through the builder pattern.
pub struct ContainerBuilder {
    ctors: Option<HashMap<TypeId, Constructors>>,
}

impl ContainerBuilder {
    /// Creates a new service container builder.
    pub fn new() -> Self {
        Self { ctors: None }
    }

    /// Creates a new service container builder with the specified capacity.
    pub fn with_constructors_capacity(capacity: usize) -> Self {
        Self {
            ctors: Some(HashMap::with_capacity(capacity)),
        }
    }

    /// Register a pair of custom constructors for a service.
    pub fn constructors<T: IService + 'static>(
        &mut self,
        instance_ctor: Option<Constructor<T::Instance>>,
        singleton_ctor: Option<Constructor<T::Pointer>>,
    ) -> &mut Self {
        let instance: Option<Constructor<()>> = unsafe { std::mem::transmute(instance_ctor) };
        let singleton: Option<Constructor<()>> = unsafe { std::mem::transmute(singleton_ctor) };

        let constructors = Constructors {
            instance,
            singleton,
        };

        let type_id = TypeId::of::<T>();
        self.ctors
            .get_or_insert_with(|| HashMap::new())
            .insert(type_id, constructors);
        
        self
    }

    /// Builds the service container.
    pub fn build(self) -> ServiceContainer {
        ServiceContainer::new(HashMap::new(), self.ctors)
    }
}
