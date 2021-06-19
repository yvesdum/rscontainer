//! Container version 2.0

use crate::internal_helpers::{OwnedCtor, SharedCtor, SharedPtr, TypeErasedService};
use crate::pointers::ISharedPointer;
use crate::service_traits::{IOwned, IShared};
use crate::ContainerBuilder;
use crate::Resolver;
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
    pub fn builder_with_capacity(capacity: usize) -> ContainerBuilder {
        ContainerBuilder::with_capacity(capacity)
    }

    /// Returns the inner hashmap for testing purposes.
    #[cfg(test)]
    #[allow(unused)]
    fn inner(&self) -> &FnvHashMap<TypeId, TypeErasedService> {
        &self.services
    }

    /// Inserts a shared instance.
    ///
    /// Panics if the instance already exists, because it is not allowed to
    /// mutate the container in such a way that other services will be
    /// shadowed.
    pub fn insert<S: 'static + ?Sized + IShared>(&mut self, instance: S::Pointer) {
        let entry = self.services.entry(TypeId::of::<S>()).or_default();
        assert!(entry.shared_ptr.is_none());
        entry.shared_ptr = Some(SharedPtr::new(instance));
    }

    /// Creates a resolver that can be used to resolve services.
    #[inline]
    pub fn resolver<'ctn>(&'ctn mut self) -> Resolver<'ctn> {
        Resolver::new(self)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Specialized Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a shared instance.
    pub(crate) fn resolve_shared<S: 'static + ?Sized + IShared>(
        &mut self,
    ) -> Result<S::Pointer, S::Error> {
        let mut instance = match self.services.get(&TypeId::of::<S>()) {
            // There's an instance in the container, so we clone the smart pointer.
            Some(TypeErasedService {
                shared_ptr: Some(ptr),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                S::Pointer::clone_from_ptr(ptr.ptr)
            },

            // There's no instance, but there is a custom constructor.
            Some(TypeErasedService {
                shared_ctor: Some(ctor),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                let ctor: SharedCtor<S> = std::mem::transmute(*ctor);
                let instance = ctor(self.resolver())?;
                self.insert::<S>(instance.clone());
                instance
            },

            // There's no instance and no custom constructor, so use the
            // default constructor.
            _ => {
                let instance = S::construct(self.resolver())?;
                self.insert::<S>(instance.clone());
                instance
            }
        };

        S::resolved(&mut instance, self.resolver());
        Ok(instance)
    }

    /// Resolves an owned instance.
    pub(crate) fn resolve_owned<S: 'static + ?Sized + IOwned>(
        &mut self,
        params: S::Parameters,
    ) -> Result<S::Instance, S::Error> {
        let mut owned = match self.services.get(&TypeId::of::<S>()) {
            // There is a custom constructor registered.
            Some(TypeErasedService {
                owned_ctor: Some(ctor),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                let ctor: OwnedCtor<S> = std::mem::transmute(*ctor);
                ctor(self.resolver(), params)?
            },

            // There is no custom constructor, so use the default one.
            _ => S::construct(self.resolver(), params)?,
        };
        S::resolved(&mut owned, self.resolver());
        Ok(owned)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Access;
    use crate::Shared;
    use std::rc::Rc;

    impl IShared for u32 {
        type Pointer = Rc<Access<u32>>;
        type Target = u32;
        type Error = ();

        fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
            Ok(Rc::new(Access::new(1234)))
        }
    }

    impl IOwned for u32 {
        type Instance = u32;
        type Parameters = ();
        type Error = ();

        fn construct(_: Resolver, _: Self::Parameters) -> Result<Self::Instance, Self::Error> {
            Ok(2468)
        }
    }

    struct Failing;

    impl IShared for Failing {
        type Pointer = Rc<Access<Failing>>;
        type Target = Failing;
        type Error = &'static str;

        fn construct(_: Resolver) -> Result<Self::Pointer, Self::Error> {
            Err("error123")
        }
    }

    impl IOwned for Failing {
        type Instance = Failing;
        type Parameters = ();
        type Error = &'static str;

        fn construct(_: Resolver, _: Self::Parameters) -> Result<Self::Instance, Self::Error> {
            Err("error456")
        }
    }

    #[test]
    fn new() {
        let ctn = ServiceContainer::new();
        assert_eq!(ctn.inner().capacity(), 0);
    }

    #[test]
    fn with_capacity() {
        let ctn = ServiceContainer::with_capacity(50);
        assert!(ctn.inner().capacity() >= 50);

        let ctn = ServiceContainer::with_capacity(1350);
        assert!(ctn.inner().capacity() >= 1350);

        let ctn = ServiceContainer::with_capacity(24);
        assert!(ctn.inner().capacity() >= 24);
    }

    #[test]
    fn insert() {
        let mut ctn = ServiceContainer::new();
        let instance = Rc::new(Access::new(()));
        ctn.insert::<()>(instance);

        assert_eq!(ctn.inner().len(), 1);
    }

    #[test]
    fn resolve_inserted() {
        let mut ctn = ServiceContainer::new();
        let instance = Rc::new(Access::new(()));
        let instance_clone = Rc::clone(&instance);
        ctn.insert::<()>(instance);
        let instance_resolved: Shared<()> = ctn.resolver().shared().unwrap();
        assert!(Rc::ptr_eq(&instance_clone, instance_resolved.inner()));
    }

    #[test]
    fn resolve_shared_returns_same_instance() {
        let mut ctn = ServiceContainer::new();
        let instance = Rc::new(Access::new(()));
        ctn.insert::<()>(instance);
        let instance_resolved: Shared<()> = ctn.resolver().shared().unwrap();
        let instance_resolved_2: Shared<()> = ctn.resolver().shared().unwrap();
        assert!(Rc::ptr_eq(
            instance_resolved.inner(),
            instance_resolved_2.inner()
        ));
    }

    #[test]
    fn resolve_shared_increases_ref_count() {
        let mut ctn = ServiceContainer::new();
        let instance = Rc::new(Access::new(()));
        ctn.insert::<()>(instance);

        let instance_resolved: Shared<()> = ctn.resolver().shared().unwrap();
        assert_eq!(Rc::strong_count(instance_resolved.inner()), 2);

        let instance_resolved_2: Shared<()> = ctn.resolver().shared().unwrap();
        assert_eq!(Rc::strong_count(instance_resolved.inner()), 3);

        drop(instance_resolved);
        drop(instance_resolved_2);
    }

    #[test]
    fn container_drop_decreases_ref_count() {
        let mut ctn = ServiceContainer::new();
        let instance = Rc::new(Access::new(()));
        let instance_clone = Rc::clone(&instance);
        ctn.insert::<()>(instance);

        assert_eq!(Rc::strong_count(&instance_clone), 2);

        drop(ctn);

        assert_eq!(Rc::strong_count(&instance_clone), 1);
    }

    #[test]
    fn resolve_shared_default_constructor() {
        let mut ctn = ServiceContainer::new();
        let instance: Shared<u32> = ctn.resolver().shared().unwrap();
        assert_eq!(***instance.inner(), 1234);
    }

    #[test]
    fn resolve_shared_custom_constructor() {
        let mut ctn = ServiceContainer::builder()
            .with_shared_constructor::<u32>(|_| Ok(Rc::new(Access::new(5678))))
            .build();

        let instance: Shared<u32> = ctn.resolver().shared().unwrap();
        assert_eq!(***instance.inner(), 5678);
    }

    #[test]
    fn resolve_shared_failing() {
        let mut ctn = ServiceContainer::new();
        let result: Result<Shared<Failing>, _> = ctn.resolver().shared();
        assert!(matches!(result, Err("error123")));
    }

    #[test]
    fn resolve_shared_custom_failing() {
        let mut ctn = ServiceContainer::builder()
            .with_shared_constructor::<u32>(|_| Err(()))
            .build();

        let result: Result<Shared<u32>, _> = ctn.resolver().shared();
        assert!(matches!(result, Err(())));
    }

    #[test]
    fn failing_should_not_insert() {
        let mut ctn = ServiceContainer::new();
        let _: Result<Shared<Failing>, _> = ctn.resolver().shared();
        assert_eq!(ctn.inner().len(), 0);
    }

    #[test]
    fn resolve_owned() {
        let mut ctn = ServiceContainer::new();
        let instance = ctn.resolver().owned::<u32>(()).unwrap();
        assert_eq!(instance, 2468);
    }

    #[test]
    fn resolve_owned_custom_constructor() {
        let mut ctn = ServiceContainer::builder()
            .with_owned_constructor::<u32>(|_, _| Ok(1357))
            .build();

        let instance = ctn.resolver().owned::<u32>(()).unwrap();
        assert_eq!(instance, 1357);
    }

    #[test]
    fn resolve_owned_custom_constructor_twice() {
        let mut ctn = ServiceContainer::builder()
            .with_owned_constructor::<u32>(|_, _| Ok(1357))
            .build();

        let instance = ctn.resolver().owned::<u32>(()).unwrap();
        let instance_2 = ctn.resolver().owned::<u32>(()).unwrap();
        assert_eq!(instance, instance_2);
    }

    #[test]
    fn resolve_owned_failing() {
        let mut ctn = ServiceContainer::new();
        let result = ctn.resolver().owned::<Failing>(());
        assert!(matches!(result, Err("error456")));
    }

    #[test]
    fn resolve_owned_custom_failing() {
        let mut ctn = ServiceContainer::builder()
            .with_owned_constructor::<u32>(|_, _| Err(()))
            .build();

        let result = ctn.resolver().owned::<u32>(());
        assert!(matches!(result, Err(())));
    }
}
