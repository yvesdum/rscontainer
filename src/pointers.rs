//! Traits for type-erasing of shared pointers.

use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////
// Trait
///////////////////////////////////////////////////////////////////////////////

/// A smart pointer that can be used to store a shared instance.
///
/// # Safety
///
/// This trait may only be implemented on reference counted pointers, such as
/// `Rc` and `Arc`. It may not be implemented on `Box`, because it could lead
/// to multiple boxes pointing to the same location.
pub unsafe trait ISharedPointer: Sized + Clone {
    /// Transforms the smart pointer into a raw pointer.
    ///
    /// # Safety
    ///
    /// After calling this method, dropping of the smart pointer should be
    /// manually handled.
    unsafe fn into_ptr(self) -> NonNull<()>;

    /// Re-inits the smart pointer from a type erased raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` should be created by the `into_ptr()` method of the
    /// same impl block. This ensures that `ptr` has the same type as `Self`.
    ///
    /// Apart from dropping, the returned smart pointer should always be cloned
    /// before it's used, because this method does not increase the ref count.
    ///
    /// It is preferred to use the `clone_from_ptr` method instead.
    unsafe fn from_ptr(ptr: NonNull<()>) -> Self;

    /// Creates a clone of the smart pointer from a raw pointer.
    ///
    /// This increases the reference count of the smart pointer.
    ///
    /// # Safety
    ///
    /// `ptr` should be created by the `into_ptr()` method of the
    /// same impl block. This ensures that `ptr` has the same type as `Self`.
    unsafe fn clone_from_ptr(ptr: NonNull<()>) -> Self {
        // SAFETY: we need to prevent the destructor of the original smart
        // pointer from running, so we wrap it in ManuallyDrop.
        let original = ManuallyDrop::new(Self::from_ptr(ptr));
        // We clone the ManuallyDrop and take the pointer out of the clone.
        // `original` is dropped without running the destructor.
        ManuallyDrop::into_inner(original.clone())
    }

    /// Decreases the reference count when the service container is dropped.
    ///
    /// # Safety
    ///
    /// `ptr` should be created by the `into_ptr()` method of the
    /// same impl block. This ensures that `ptr` has the same type as `Self`.
    ///
    /// After this method `ptr` points to possibly freed memory, so it should
    /// not be used anymore.
    unsafe fn drop_from_ptr(ptr: NonNull<()>) {
        drop(Self::from_ptr(ptr))
    }

    /// Returns true if `self` points to the same location as `other`.
    fn ptr_eq(&self, other: &Self) -> bool;
}

///////////////////////////////////////////////////////////////////////////////
// Implementations
///////////////////////////////////////////////////////////////////////////////

unsafe impl<T> ISharedPointer for Rc<T> {
    unsafe fn from_ptr(ptr: NonNull<()>) -> Self {
        Rc::from_raw(ptr.as_ptr() as *const T)
    }

    unsafe fn into_ptr(self) -> NonNull<()> {
        let raw = Rc::into_raw(self) as *mut ();
        NonNull::new_unchecked(raw)
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

unsafe impl<T> ISharedPointer for Arc<T> {
    unsafe fn from_ptr(ptr: NonNull<()>) -> Self {
        Arc::from_raw(ptr.as_ptr() as *const T)
    }

    unsafe fn into_ptr(self) -> NonNull<()> {
        let raw = Arc::into_raw(self) as *mut ();
        NonNull::new_unchecked(raw)
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc_from_ptr() {
        let rc = Rc::new(100u32);
        let rc_ptr = Rc::into_raw(Rc::clone(&rc));

        let rc_from_ptr: Rc<u32> =
            unsafe { ISharedPointer::from_ptr(NonNull::new(rc_ptr as *mut _).unwrap()) };

        assert!(Rc::ptr_eq(&rc_from_ptr, &rc));
        assert_eq!(*rc_from_ptr, *rc);
    }

    #[test]
    fn rc_into_ptr() {
        let rc = Rc::new(100u32);
        let rc_clone = Rc::clone(&rc);

        let rc_ptr = Rc::as_ptr(&rc);
        let rc_clone_ptr = unsafe { ISharedPointer::into_ptr(rc_clone) };

        assert_eq!(rc_ptr, rc_clone_ptr.as_ptr() as *const _);

        unsafe {
            drop(Rc::from_raw(rc_clone_ptr.as_ptr() as *const u32));
        }
    }

    #[test]
    fn rc_clone_from_ptr() {
        let rc = Rc::new(100u32);

        let ptr = unsafe { ISharedPointer::into_ptr(rc) };

        let rc_clone: Rc<u32> = unsafe { ISharedPointer::clone_from_ptr(ptr) };

        assert_eq!(Rc::strong_count(&rc_clone), 2);

        unsafe {
            drop(Rc::from_raw(ptr.as_ptr() as *const u32));
        }
    }

    #[test]
    fn rc_drop_from_ptr() {
        let rc = Rc::new(100u32);

        let ptr = unsafe { ISharedPointer::into_ptr(rc) };

        let rc_clone: Rc<u32> = unsafe { ISharedPointer::clone_from_ptr(ptr) };

        assert_eq!(Rc::strong_count(&rc_clone), 2);

        unsafe {
            <Rc<u32> as ISharedPointer>::drop_from_ptr(ptr);
        }

        assert_eq!(Rc::strong_count(&rc_clone), 1);
    }

    #[test]
    fn arc_from_ptr() {
        let rc = Arc::new(100u32);
        let rc_ptr = Arc::into_raw(Arc::clone(&rc));

        let rc_from_ptr: Arc<u32> =
            unsafe { ISharedPointer::from_ptr(NonNull::new(rc_ptr as *mut _).unwrap()) };

        assert!(Arc::ptr_eq(&rc_from_ptr, &rc));
        assert_eq!(*rc_from_ptr, *rc);
    }

    #[test]
    fn arc_into_ptr() {
        let rc = Arc::new(100u32);
        let rc_clone = Arc::clone(&rc);

        let rc_ptr = Arc::as_ptr(&rc);
        let rc_clone_ptr = unsafe { ISharedPointer::into_ptr(rc_clone) };

        assert_eq!(rc_ptr, rc_clone_ptr.as_ptr() as *const _);

        unsafe {
            drop(Arc::from_raw(rc_clone_ptr.as_ptr() as *const u32));
        }
    }

    #[test]
    fn arc_clone_from_ptr() {
        let rc = Arc::new(100u32);

        let ptr = unsafe { ISharedPointer::into_ptr(rc) };

        let rc_clone: Arc<u32> = unsafe { ISharedPointer::clone_from_ptr(ptr) };

        assert_eq!(Arc::strong_count(&rc_clone), 2);

        unsafe {
            drop(Arc::from_raw(ptr.as_ptr() as *const u32));
        }
    }

    #[test]
    fn arc_drop_from_ptr() {
        let rc = Arc::new(100u32);

        let ptr = unsafe { ISharedPointer::into_ptr(rc) };

        let rc_clone: Arc<u32> = unsafe { ISharedPointer::clone_from_ptr(ptr) };

        assert_eq!(Arc::strong_count(&rc_clone), 2);

        unsafe {
            <Arc<u32> as ISharedPointer>::drop_from_ptr(ptr);
        }

        assert_eq!(Arc::strong_count(&rc_clone), 1);
    }
}
