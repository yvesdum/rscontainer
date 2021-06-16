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

    /// Returns the inner hashmap for testing purposes.
    #[cfg(test)]
    #[allow(unused)]
    fn inner(&self) -> &FnvHashMap<TypeId, TypeErasedService> {
        &self.services
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

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::Access;

    #[test]
    fn new() {
        let ctn = ContainerBuilder::new();
        assert_eq!(ctn.inner().capacity(), 0);
    }

    #[test]
    fn with_capacity() {
        let ctn = ContainerBuilder::with_capacity(50);
        assert!(ctn.inner().capacity() >= 50);

        let ctn = ContainerBuilder::with_capacity(1350);
        assert!(ctn.inner().capacity() >= 1350);

        let ctn = ContainerBuilder::with_capacity(24);
        assert!(ctn.inner().capacity() >= 24);
    }

    #[test]
    fn entry() {
        let mut ctn = ContainerBuilder::new();
        let entry = ctn.entry(TypeId::of::<()>());

        assert!(entry.shared_ptr.is_none());
        assert!(entry.shared_ctor.is_none());
        assert!(entry.local_ctor.is_none());
    }

    #[test]
    fn with_shared() {
        let mut ctn = ContainerBuilder::new();

        let shared = Shared::<u32>::new(Rc::new(Access::new(100)));
        let shared_clone = shared.clone();
        ctn = ctn.with_shared(shared);

        assert_eq!(ctn.inner().len(), 1);

        let entry = ctn.entry(TypeId::of::<u32>());

        assert_eq!(
            Rc::as_ptr(shared_clone.inner()) as *const (),
            entry.shared_ptr.as_ref().unwrap().ptr.as_ptr() as *const ()
        );
    }

    #[test]
    fn with_shared_constructor() {
        let mut ctn = ContainerBuilder::new();

        fn ctor(_: &mut ServiceContainer) -> Result<Rc<Access<u32>>, ()> {
            Ok(Rc::new(Access::new(456)))
        }

        ctn = ctn.with_shared_constructor::<u32>(ctor);

        assert_eq!(ctn.inner().len(), 1);

        let entry = ctn.entry(TypeId::of::<u32>());

        assert_eq!(
            ctor as *const (),
            *entry.shared_ctor.as_ref().unwrap() as *const ()
        );
    }
    
    #[test]
    fn with_local_constructor() {
        let mut ctn = ContainerBuilder::new();

        fn ctor(_: &mut ServiceContainer, _: ()) -> Result<u32, ()> {
            Ok(456)
        }

        ctn = ctn.with_local_constructor::<u32>(ctor);

        assert_eq!(ctn.inner().len(), 1);

        let entry = ctn.entry(TypeId::of::<u32>());

        assert_eq!(
            ctor as *const (),
            *entry.local_ctor.as_ref().unwrap() as *const ()
        );
    }

    #[test]
    fn with_constructors() {
        let mut ctn = ContainerBuilder::new();

        fn shared_ctor(_: &mut ServiceContainer) -> Result<Rc<Access<u32>>, ()> {
            Ok(Rc::new(Access::new(456)))
        }

        fn local_ctor(_: &mut ServiceContainer, _: ()) -> Result<u32, ()> {
            Ok(456)
        }

        ctn = ctn.with_constructors::<u32>(local_ctor, shared_ctor);

        assert_eq!(ctn.inner().len(), 1);

        let entry = ctn.entry(TypeId::of::<u32>());

        assert_eq!(
            shared_ctor as *const (),
            *entry.shared_ctor.as_ref().unwrap() as *const ()
        );

        assert_eq!(
            local_ctor as *const (),
            *entry.local_ctor.as_ref().unwrap() as *const ()
        );
    }
}