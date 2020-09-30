//! Traits to enable smart pointers to work with the container.

use crate::static_services::service_traits::IService;
use std::cell::{RefCell, Ref, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::Deref;

///////////////////////////////////////////////////////////////////////////////
// Main Traits
///////////////////////////////////////////////////////////////////////////////

/// A smart pointer that can be used as a pointer to a singleton.
///
/// # Safety
///
/// This trait may only be implemented on reference counted pointers, such as
/// `Rc` and `Arc`. It may not be implemented on `Box`, because it could lead
/// to multiple boxes pointing to the same object, which violates Rusts memory
/// model.
pub unsafe trait IPointer {
    /// Transforms the smart pointer into a type-erased raw pointer.
    ///
    /// # Safety
    ///
    /// After calling this method, dropping of the smart pointer should be
    /// manually handled.
    unsafe fn into_type_erased_raw(self) -> *const ();

    /// Re-inits the smart pointer from a type erased pointer.
    ///
    /// # Safety
    ///
    /// `ptr` should be created by the `into_type_erased_raw()` method of the 
    /// same impl block. This ensures that `ptr` has the same type as `Self`.
    ///
    /// Apart from dropping, the returned smart pointer should always be cloned
    /// before it's used, because this method does not increase the ref count.
    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized;

    /// Called when the service is removed from the service container,
    /// or when the service container is dropped.
    ///
    /// # Safety
    ///
    /// `ptr` should be created by the `into_type_erased_raw()` method of the 
    /// same impl block. This ensures that `ptr` has the same type as `Self`.
    unsafe fn drop_type_erased(ptr: *const ())
    where
        Self: Sized,
    {
        // We want to drop the original instance, so we don't clone.
        let pointer = Self::from_type_erased_raw(ptr);
        drop(pointer)
    }
}

/// A smart pointer to read from a service.
pub trait IReadPointer<'a> {
    type ReadGuard;

    fn read(&'a self) -> Self::ReadGuard;
}

/// A smart pointer to mutate a service.
pub trait IWritePointer<'a> {
    type WriteGuard;

    fn write(&'a self) -> Self::WriteGuard;
}

///////////////////////////////////////////////////////////////////////////////
// Blanket Implementation for Rc<T>
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T: IService> IPointer for Rc<T> {
    unsafe fn into_type_erased_raw(self) -> *const () {
        Rc::into_raw(self) as *const ()
    }

    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized,
    {
        Rc::from_raw(ptr as *const T)
    }
}

impl<'a, T: IService + 'a> IReadPointer<'a> for Rc<T> {
    type ReadGuard = &'a T;

    fn read(&'a self) -> Self::ReadGuard {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////
// Blanket Implementation for Rc<RefCell<T>>
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T: IService> IPointer for Rc<RefCell<T>> {
    unsafe fn into_type_erased_raw(self) -> *const () {
        Rc::into_raw(self) as *const ()
    }

    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized,
    {
        Rc::from_raw(ptr as *const RefCell<T>)
    }
}

impl<'a, T: IService + 'a> IReadPointer<'a> for Rc<RefCell<T>> {
    type ReadGuard = Ref<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.borrow()
    }
}

impl<'a, T: IService + 'a> IWritePointer<'a> for Rc<RefCell<T>> {
    type WriteGuard = RefMut<'a, T>;

    fn write(&'a self) -> Self::WriteGuard {
        self.borrow_mut()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Blanket Implementation for Arc<T>
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T: IService> IPointer for Arc<T> {
    unsafe fn into_type_erased_raw(self) -> *const () {
        Arc::into_raw(self) as *const ()
    }

    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized,
    {
        Arc::from_raw(ptr as *const T)
    }
}

impl<'a, T: IService + 'a> IReadPointer<'a> for Arc<T> {
    type ReadGuard = &'a T;

    fn read(&'a self) -> Self::ReadGuard {
        self
    }
}


///////////////////////////////////////////////////////////////////////////////
// Blanket Implementation for Arc<Mutex<T>>
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T: IService> IPointer for Arc<Mutex<T>> {
    unsafe fn into_type_erased_raw(self) -> *const () {
        Arc::into_raw(self) as *const ()
    }

    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized,
    {
        Arc::from_raw(ptr as *const Mutex<T>)
    }
}

impl<'a, T: IService + 'a> IReadPointer<'a> for Arc<Mutex<T>> {
    type ReadGuard = MutexGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        // `lock()` returns an `Err` if the lock is poisoned. If this is the
        // case than something went seriously wrong so `unwrap()` is allowed.
        self.lock().unwrap()
    }
}

impl<'a, T: IService + 'a> IWritePointer<'a> for Arc<Mutex<T>> {
    type WriteGuard = MutexGuard<'a, T>;

    fn write(&'a self) -> Self::WriteGuard {
        // `lock()` returns an `Err` if the lock is poisoned. If this is the
        // case than something went seriously wrong so `unwrap()` is allowed.
        self.lock().unwrap()
    }
}

///////////////////////////////////////////////////////////////////////////////
// Blanket Implementation for Arc<RwLock<T>>
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T: IService> IPointer for Arc<RwLock<T>> {
    unsafe fn into_type_erased_raw(self) -> *const () {
        Arc::into_raw(self) as *const ()
    }

    unsafe fn from_type_erased_raw(ptr: *const ()) -> Self
    where
        Self: Sized,
    {
        Arc::from_raw(ptr as *const RwLock<T>)
    }
}

impl<'a, T: IService + 'a> IReadPointer<'a> for Arc<RwLock<T>> {
    type ReadGuard = RwLockReadGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        // `read()` returns an `Err` if the lock is poisoned. If this is the
        // case than something went seriously wrong so `unwrap()` is allowed.
        self.deref().read().unwrap()
    }
}

impl<'a, T: IService + 'a> IWritePointer<'a> for Arc<RwLock<T>> {
    type WriteGuard = RwLockWriteGuard<'a, T>;

    fn write(&'a self) -> Self::WriteGuard {
        // `write()` returns an `Err` if the lock is poisoned. If this is the
        // case than something went seriously wrong so `unwrap()` is allowed.
        self.deref().write().unwrap()
    }
}