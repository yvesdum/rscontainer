//! Container version 2.0

use super::getters::{Global, IResolveGlobal, IResolveLocal, Local};
use super::pointers::IGlobalPointer;
use super::service_traits::{IGlobal, ILocal};
use fnv::FnvHashMap;
use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::fmt;
use std::ptr::NonNull;

///////////////////////////////////////////////////////////////////////////////
// Internal Storage Helpers
///////////////////////////////////////////////////////////////////////////////

/// A raw pointer to a singleton instance with drop logic.
/// This is a type-erased `Rc` or `Arc` that implements `ISingletonPointer`.
#[derive(Debug)]
struct GlobalPtr {
    ptr: NonNull<()>,
    dtor: unsafe fn(NonNull<()>),
}

impl Drop for GlobalPtr {
    fn drop(&mut self) {
        #[cfg(test)]
        println!("Dropping SingletonPtr {:p}", self);

        unsafe { (self.dtor)(self.ptr) }
    }
}

impl GlobalPtr {
    fn new<P: IGlobalPointer>(instance: P) -> Self {
        GlobalPtr {
            ptr: unsafe { instance.into_ptr() },
            dtor: P::drop,
        }
    }
}

/// A custom constructor for a global instance.
type GlobalCtor<S> = fn(&mut ServiceContainer) -> Result<Global<S>, <S as IGlobal>::Error>;

/// A custom constructor for a local instance.
type LocalCtor<S> =
    fn(&mut ServiceContainer, <S as ILocal>::Parameters) -> Result<Local<S>, <S as ILocal>::Error>;

/// A service in the container that is type erased.
pub(crate) struct TypeErasedService {
    /// A raw pointer to the global instance.
    global_ptr: Option<GlobalPtr>,
    /// Custom constructor for a global instance.
    global_ctor: Option<GlobalCtor<()>>,
    /// Custom constructor for a local instance.
    local_ctor: Option<LocalCtor<()>>,
}

impl TypeErasedService {
    fn new(global_ptr: GlobalPtr) -> Self {
        Self {
            global_ptr: Some(global_ptr),
            global_ctor: None,
            local_ctor: None,
        }
    }
}

impl fmt::Debug for TypeErasedService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeErasedService")
            .field("global_ptr", &self.global_ptr)
            .field("global_ctor", &self.global_ctor.is_some())
            .field("local_ctor", &self.local_ctor.is_some())
            .finish()
    }
}

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

    /// Returns the inner hashmap for testing purposes.
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn inner_hashmap(&self) -> &FnvHashMap<TypeId, TypeErasedService> {
        &self.services
    }

    /// Inserts a global instance.
    ///
    /// Panics if the instance already exists, because it is not allowed to
    /// mutate the container after it's built.
    pub fn insert<S: 'static + ?Sized + IGlobal>(&mut self, singleton: Global<S>) {
        match self.services.entry(TypeId::of::<S>()) {
            Entry::Vacant(entry) => {
                entry.insert(TypeErasedService::new(GlobalPtr::new(
                    singleton.into_inner(),
                )));
            }
            Entry::Occupied(mut entry) => {
                let service = entry.get_mut();
                assert!(service.global_ptr.is_none());
                service.global_ptr = Some(GlobalPtr::new(singleton.into_inner()));
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Generic Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a `Global` or `Instance::Global`.
    #[inline]
    pub fn global<R: IResolveGlobal>(&mut self) -> Result<R, R::Error> {
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

    /// Resolves a global instance.
    pub fn resolve_global<S: 'static + ?Sized + IGlobal>(&mut self) -> Result<Global<S>, S::Error> {
        let instance = match self.services.get(&TypeId::of::<S>()) {
            // There's an instance in the container, so we clone the smart pointer.
            Some(TypeErasedService {
                global_ptr: Some(ptr),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                Global::new(S::Pointer::clone_from_ptr(ptr.ptr))
            },

            // There's no instance, but there is a custom constructor.
            Some(TypeErasedService {
                global_ctor: Some(ctor),
                ..
            }) => unsafe {
                // SAFETY: because the TypeId is the key, we're certain
                // that we're casting to the right type.
                let ctor: GlobalCtor<S> = std::mem::transmute(*ctor);
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
