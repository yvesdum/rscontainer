//! Object dependency management for Rust.
//!
//! # Features
//!
//! * Automatic construction of objects which depend on other objects
//! * Dependency Injection
//! * Storage and management of Singletons
//! * Compatible with multiple smart pointer types, locking mechanisms and
//!   interiour mutability mechanisms
//! * No setup step required
//! * Works with any existing type without writing complicated wrapper types
//! * Optional registration of custom constructors
//!
//! # Creating a Service Container
//!
//! To create a service container without any configuration, use the 
//! [`ServiceContainer::empty()`] method.
//!
//! ```rust
//! use rscontainer::ServiceContainer;
//!
//! fn main() {
//!     let mut container = ServiceContainer::empty();
//! }
//! ```
//!
//! To register custom constructors, use the [`ContainerBuilder`].
//!
//! ```rust
//! use rscontainer::ContainerBuilder;
//!
//! fn main() {
//!     let mut builder = ContainerBuilder::new();
//!     
//!     builder.constructors::<MyService>(Some(|container| {
//!         MyService::new_instance(container)
//!     }), Some(|container| {
//!         MyService::new_singleton(container)
//!     }));
//!
//!     let mut container = builder.build();
//! }
//! ```
//!
//! # Using the Service Container with static services
//!
//! [**Singletons**] are instances which are shared throughout your 
//! application. Each time you resolve a singleton, you will get the same 
//! instance. A singleton is always behind a shared smart pointer, such as 
//! `Arc` or `Rc` and may have an access mechanism such as `RefCell` or 
//! `Mutex`. Each service decides for themselve which kind of pointer and 
//! mechanism they use. The first time you ask for an instance of a certain 
//! singleton, the container constructs it and recursively constructs and 
//! injects the neccessary dependencies. The instance is than stored in the 
//! container.
//!
//! To read from or mutate a singleton, you use the `read()` and `write()`
//! methods. This might lock the singleton, so immediately use the result
//! of these methods or keep the results in a scope that is as short as
//! possible.
//!
//! ```
//! use rscontainer::{ServiceContainer, Singleton};
//!
//! let mut container = ServiceContainer::new();
//! let singleton: Singleton<MyService> = container.resolve();
//!
//! singleton.write().set_something("something");
//! let something = singleton.read().get_something();
//! ```
//!
//! [**Instances**] are instances which are different each time you resolve 
//! them from the container. They are not behind a pointer and access 
//! mechanism. The container will still take care of injecting the neccessary
//! dependencies.
//!
//! Because instances are not behind a pointer, you don't need `read()` or
//! `write()` to interact with them. Instances implement `Deref` and
//! `DerefMut` instead.
//!
//! ```
//! use rscontainer::{ServiceContainer, Instance};
//!
//! let mut container = ServiceContainer::new();
//! let instance: Instance<MyService> = container.resolve();
//! ```
//!
//! [`Singleton<T>`] and [`Instance<T>`] do not carry a lifetime parameter,
//! therefore they can be stored in structs and enums very easily.
//!
//! # Enabling a type to be used as a Service
//!
//! To enable a type to be resolved through the service container, you need to
//! implement the [`IService`] trait on it. See the documentation of this trait
//! for more information.
//!
//! If your service depends on other services, you need to store these services
//! as fields in your struct as [`Singleton<T>`] or [`Instance<T>`].
//!
//! [`ServiceContainer::empty()`]: struct.ServiceContainer.html#method.empty
//! [`ContainerBuilder`]: struct.ContainerBuilder.html
//! [**Singletons**]: struct.Singleton.html
//! [**Instances**]: struct.Instance.html
//! [`IService`]: trait.IService.html
//! [`Singleton<T>`]: struct.Singleton.html
//! [`Instance<T>`]: struct.Instance.html

mod builder;
mod helpers;
mod static_services;

pub use crate::builder::ContainerBuilder;
pub use crate::static_services::getters::{Instance, Singleton};
pub use crate::static_services::service_traits::IService;

use crate::helpers::{Constructor, Constructors, IResolve, SingletonPtr};
use crate::static_services::pointers::IPointer;
use log::trace;
use std::any::type_name;
use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;

//////////////////////////////////////////////////////////////////////////////
// Main Service Container
//////////////////////////////////////////////////////////////////////////////

/// A container for services.
///
/// Manages dependencies between these services.
pub struct ServiceContainer {
    singletons: HashMap<TypeId, SingletonPtr>,
    constructors: Option<HashMap<TypeId, Constructors>>,
}

impl ServiceContainer {
    //////////////////////////////////////////////////////////////////////////
    // Constructors
    //////////////////////////////////////////////////////////////////////////

    /// Creates a new, empty service container.
    pub fn empty() -> Self {
        Self {
            singletons: HashMap::new(),
            constructors: None,
        }
    }

    /// Creates a new service container.
    pub(crate) fn new(
        singletons: HashMap<TypeId, SingletonPtr>,
        constructors: Option<HashMap<TypeId, Constructors>>,
    ) -> Self {
        Self {
            singletons,
            constructors,
        }
    }

    /// Creates a new service container with the specified reserved capacity.
    pub fn with_capacity(singletons: usize) -> Self {
        Self {
            singletons: HashMap::with_capacity(singletons),
            constructors: None,
        }
    }

    //////////////////////////////////////////////////////////////////////////
    // Resolve Methods
    //////////////////////////////////////////////////////////////////////////

    /// Resolve an object from the service container.
    ///
    /// This method can be used to resolve a service in any form, be it
    /// a singleton, instance, etc..
    #[inline]
    pub fn resolve<T>(&mut self) -> T
    where
        T: IResolve,
    {
        T::resolve(self)
    }

    /// Resolves or constructs a singleton.
    pub fn resolve_singleton<T>(&mut self) -> Singleton<T>
    where
        T: IService + 'static,
    {
        trace!("Resolving singleton {}", type_name::<T>());
        let key = TypeId::of::<T>();

        // Check if we have a saved instance and return it.
        if let Some(singleton) = self.singletons.get(&key) {
            // We are sure that the raw pointer is `T`, because we saved the
            // type id in the key. We clone the smart pointer and forget the 
            // original one to increase the reference count.
            let raw_ptr = singleton.ptr.as_ptr();
            let smart_ptr = unsafe { T::Pointer::from_type_erased_raw(raw_ptr) };
            let pointer = smart_ptr.clone();
            std::mem::forget(smart_ptr);
            return Singleton { pointer };
        }

        // If there's no saved instance, check if there is
        // a custom constructor registered.
        let ctors = match &self.constructors {
            Some(ctors) => ctors.get(&key),
            None => None
        };

        // If there is a custom constructor, call it. If there is none,
        // call the default constructor.
        let pointer = if let Some(ctors) = ctors {
            let ctor: Constructor<T::Pointer> = unsafe { std::mem::transmute(ctors.singleton) };
            ctor(self)
        } else {
            T::construct_singleton(self)
        };

        // Store a clone of the singleton in the container.
        let raw_ptr = unsafe { T::Pointer::into_type_erased_raw(pointer.clone()) };
        let nonnull_ptr = unsafe { NonNull::new_unchecked(raw_ptr as *mut ()) };
        let singleton_ptr = SingletonPtr {
            ptr: nonnull_ptr,
            dtor: T::Pointer::drop_type_erased,
        };
        self.singletons.insert(key, singleton_ptr);

        Singleton { pointer }
    }

    /// Constructs an instance through the service container.
    pub fn resolve_instance<T>(&mut self) -> Instance<T>
    where
        T: IService + 'static,
    {
        trace!("Resolving instance {}", type_name::<T>());
        let key = TypeId::of::<T>();

        // Check if there's a custom constructor registered.
        let ctors = match &self.constructors {
            Some(ctors) => ctors.get(&key),
            None => None
        };

        // If there is a custom constructor, call it. If there is none,
        // call the default constructor.
        let instance = if let Some(ctors) = ctors {
            let ctor: Constructor<T::Instance> = unsafe { std::mem::transmute(ctors.instance) };
            ctor(self)
        } else {
            T::construct(self)
        };

        Instance { instance }
    }

    //////////////////////////////////////////////////////////////////////////
    // Meta Data Getters
    //////////////////////////////////////////////////////////////////////////

    /// Returns the number of static singletons that are currently residing in
    /// the service container.
    pub fn num_singletons(&self) -> usize {
        self.singletons.len()
    }

    /// Returns true if an instance of the supplied TypeId is currently
    /// residing in the container as a singleton.
    pub fn constains_type(&self, key: TypeId) -> bool {
        self.singletons.contains_key(&key)
    }

    /// Returns true if an instance of the supplied singleton is currently
    /// residing in the container.
    pub fn contains<T>(&self) -> bool
    where
        T: IService + 'static,
    {
        let key = TypeId::of::<T>();
        self.constains_type(key)
    }

    //////////////////////////////////////////////////////////////////////////
    // Modifiers
    //////////////////////////////////////////////////////////////////////////

    /// Removes a singleton from the container.
    ///
    /// This does not invalidate any previously resolved instances of the
    /// singleton.
    ///
    /// If the singleton existed, it is returned. Otherwise `None` is returned.
    pub fn remove<T>(&mut self) -> Option<Singleton<T>>
    where
        T: IService + 'static,
    {
        let key = TypeId::of::<T>();
        if let Some(singleton) = self.singletons.remove(&key) {
            // We are sure that the raw pointer is `T`, because we saved the
            // type id in the key. Here we don't clone the smart pointer but
            // return the original one.
            let raw_ptr = singleton.ptr.as_ptr();
            let pointer = unsafe { T::Pointer::from_type_erased_raw(raw_ptr) };
            Some(Singleton { pointer })
        } else {
            None
        }
    }
}
