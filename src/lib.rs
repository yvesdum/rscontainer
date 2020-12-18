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
//! [`ServiceContainer::new()`] method.
//!
//! ```rust
//! use rscontainer::ServiceContainer;
//!
//! fn main() {
//!     let mut container = ServiceContainer::new();
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
//!     builder.constructors::<MyService>(
//!         |container| {
//!             MyService::new_instance(container)
//!         },
//!         |container| {
//!             MyService::new_singleton(container)
//!         }
//!     );
//!
//!     let mut container = builder.build();
//! }
//! ```
//!
//! # Using the Service Container with static services
//!
//! **Singletons** are instances which are shared throughout your 
//! application. Each time you resolve a singleton, you will get the same 
//! instance. See [`Singleton<T>`] for more information.
//!
//! ```
//! use rscontainer::{ServiceContainer, Singleton};
//!
//! let mut container = ServiceContainer::new();
//!
//! let singleton: Singleton<MyService> = container.resolve();
//! singleton.write().set_something("something");
//! let something = singleton.read().get_something();
//! ```
//!
//! **Instances** are instances which are different each time you resolve 
//! them from the container. See [`Instance<T>`] for more information.
//!
//! ```
//! use rscontainer::{ServiceContainer, Instance};
//!
//! let mut container = ServiceContainer::new();
//!
//! let mut instance: Instance<MyService> = container.resolve();
//! instance.set_something("something");
//! let something = instance.get_something();
//! ```
//!
//! [`Singleton<T>`] and [`Instance<T>`] do not carry a lifetime parameter,
//! therefore they can be stored in structs and enums very easily.
//!
//! # Enabling a type to be used as a Service
//!
//! To enable a type to be resolved through the service container, you need to
//! implement the [`IService`] trait on it. With a simple trick it's possible
//! to use any existing type as a service. See the documentation of this trait
//! for more information.
//!
//! [`ServiceContainer::new()`]: crate::ServiceContainer::new
//! [`ContainerBuilder`]: crate::ContainerBuilder
//! [`Singleton<T>`]: crate::Singleton
//! [`Instance<T>`]: crate::Instance
//! [`IService`]: crate::IService

mod builder;
mod helpers;
mod static_services;

pub use crate::builder::ContainerBuilder;
pub use crate::static_services::getters::{Instance, Singleton};
pub use crate::static_services::service_traits::IService;
pub use crate::static_services::pointers::{IPointer, IWritePointer, IReadPointer};

use crate::helpers::{Constructor, Constructors, IResolve, SingletonPtr};
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
    constructors: HashMap<TypeId, Constructors>,
}

impl ServiceContainer {
    //////////////////////////////////////////////////////////////////////////
    // Constructors
    //////////////////////////////////////////////////////////////////////////

    /// Creates a new, empty service container.
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            constructors: HashMap::new(),
        }
    }

    /// Creates a new service container that is already built.
    pub(crate) fn new_built(
        singletons: HashMap<TypeId, SingletonPtr>,
        constructors: HashMap<TypeId, Constructors>,
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
            constructors: HashMap::new(),
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
        let ctors = match self.constructors.is_empty() {
            true => None,
            false => self.constructors.get(&key)
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
        let ctors = match self.constructors.is_empty() {
            true => None,
            false => self.constructors.get(&key)
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
