//! Container version 2.0

use super::getters::{IResolve, IResolveLocal, IResolveSingleton, Instance, Local, Singleton};
use super::pointers::ISingletonPointer;
use super::service_trait::IService;
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
struct SingletonPtr {
    ptr: NonNull<()>,
    dtor: unsafe fn(NonNull<()>),
}

/// Custom constructors for a service.
#[repr(C)]
struct Ctors<S: IService> {
    /// A custom constructor that creates a singleton instance.
    singleton_ctor: Option<fn(&mut ServiceContainer) -> Result<Singleton<S>, S::Error>>,
    /// A custom constructor that creates a local instance.
    local_ctor: Option<fn(&mut ServiceContainer, S::Params) -> Result<Local<S>, S::Error>>,
}

/// A service in the container that is type erased.
#[derive(Debug)]
pub(crate) struct TypeErasedService {
    /// A raw pointer to the singleton instance.
    singleton_ptr: Option<SingletonPtr>,
    /// Custom constructors for the service.
    ctors: Ctors<()>,
}

impl Drop for SingletonPtr {
    fn drop(&mut self) {
        #[cfg(test)]
        println!("Dropping SingletonPtr {:p}", self);

        unsafe { (self.dtor)(self.ptr) }
    }
}

impl SingletonPtr {
    fn new<P: ISingletonPointer>(instance: P) -> Self {
        SingletonPtr {
            ptr: unsafe { instance.into_ptr() },
            dtor: P::drop,
        }
    }
}

impl Clone for Ctors<()> {
    fn clone(&self) -> Self {
        Ctors {
            singleton_ctor: self.singleton_ctor.clone(),
            local_ctor: self.local_ctor.clone(),
        }
    }
}

impl Copy for Ctors<()> {}

impl fmt::Debug for Ctors<()> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Service")
            .field("singleton_ctor", &self.singleton_ctor.is_some())
            .field("local_ctor", &self.local_ctor.is_some())
            .finish()
    }
}

impl TypeErasedService {
    fn new<S: IService>(singleton: Singleton<S>) -> Self {
        Self {
            singleton_ptr: Some(SingletonPtr::new(singleton.into_inner())),
            ctors: Ctors {
                singleton_ctor: None,
                local_ctor: None,
            },
        }
    }

    unsafe fn ctors<S: IService>(&self) -> Ctors<S> {
        std::mem::transmute::<Ctors<()>, Ctors<S>>(self.ctors)
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
    pub(crate) fn inner_hashmap(&self) -> &FnvHashMap<TypeId, TypeErasedService> {
        &self.services
    }

    /// Inserts a singleton instance.
    ///
    /// Panics if the singleton already exists.
    pub fn insert<S: IService + 'static>(&mut self, singleton: Singleton<S>) {
        match self.services.entry(TypeId::of::<S>()) {
            Entry::Vacant(entry) => {
                entry.insert(TypeErasedService::new(singleton));
            }
            Entry::Occupied(mut entry) => {
                let service = entry.get_mut();
                assert!(service.singleton_ptr.is_none());
                service
                    .singleton_ptr
                    .replace(SingletonPtr::new(singleton.into_inner()));
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Generic Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a `Singleton`, `Local` or `Instance::Singleton`.
    #[inline]
    pub fn resolve<R: IResolve>(&mut self) -> Result<R, R::Error> {
        R::resolve(self)
    }

    /// Resolves a `Singleton` or `Instance::Singleton`.
    #[inline]
    pub fn singleton<R: IResolveSingleton>(&mut self) -> Result<R, R::Error> {
        R::resolve_singleton(self)
    }

    /// Resolves a `Local` or `Instance::Local`.
    #[inline]
    pub fn local<R: IResolveLocal>(&mut self) -> Result<R, R::Error> {
        R::resolve_local(self)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Specialized Resolve Methods
    ///////////////////////////////////////////////////////////////////////////

    /// Resolves a `Singleton` instance.
    pub fn resolve_singleton<S: IService + 'static>(&mut self) -> Result<Singleton<S>, S::Error> {
        let singleton = match self.services.get(&TypeId::of::<S>()) {
            Some(service) => match &service.singleton_ptr {
                Some(ptr) => unsafe {
                    // If there is an instance in the hashmap, we clone the
                    // pointer.
                    // SAFETY: because the TypeId is the key, we're certain
                    // that we're casting to the right type.
                    Singleton::new(S::Pointer::clone_from_ptr(ptr.ptr))
                },
                None => {
                    let ctors = unsafe {
                        // SAFETY: because the TypeId is the key, we're certain
                        // that we're casting to the right type.
                        service.ctors::<S>()
                    };
                    let singleton = match ctors.singleton_ctor {
                        Some(ctor) => ctor(self)?,
                        None => S::new_singleton(self)?,
                    };
                    self.insert(singleton.clone());
                    singleton
                }
            },
            None => {
                let singleton = S::new_singleton(self)?;
                self.insert(singleton.clone());
                singleton
            }
        };
        S::resolved_singleton(&singleton, self);
        Ok(singleton)
    }

    /// Resolves a `Local` instance.
    pub fn resolve_local<S: IService + 'static>(
        &mut self,
        params: S::Params,
    ) -> Result<Local<S>, S::Error> {
        let mut local = match self.services.get(&TypeId::of::<S>()) {
            Some(service) => {
                let ctors = unsafe {
                    // SAFETY: because the TypeId is the key, we're certain
                    // that we're casting to the right type.
                    service.ctors::<S>()
                };
                match ctors.local_ctor {
                    Some(ctor) => ctor(self, params)?,
                    None => S::new_local(self, params)?,
                }
            }
            None => S::new_local(self, params)?,
        };
        S::resolved_local(&mut local, self);
        Ok(local)
    }

    /// Resolves a `Local` instance with default parameters.
    pub fn resolve_local_default<S: IService + 'static>(&mut self) -> Result<Local<S>, S::Error>
    where
        S::Params: Default,
    {
        self.resolve_local::<S>(Default::default())
    }

    /// Resolves an `Instance::Singleton` instance.
    pub fn resolve_singleton_instance<S: IService + 'static>(
        &mut self,
    ) -> Result<Instance<S>, S::Error> {
        self.resolve_singleton()
            .map(|s| Instance::from_singleton(s))
    }

    /// Resolves an `Instance::Local` instance.
    pub fn resolve_local_instance<S: IService + 'static>(
        &mut self,
        params: S::Params,
    ) -> Result<Instance<S>, S::Error> {
        self.resolve_local(params).map(|l| Instance::from_local(l))
    }

    /// Resolves an `Instance::Local` instance with default parameters.
    pub fn resolve_local_instance_default<S: IService + 'static>(
        &mut self,
    ) -> Result<Instance<S>, S::Error>
    where
        S::Params: Default,
    {
        self.resolve_local_default()
            .map(|l| Instance::from_local(l))
    }
}
