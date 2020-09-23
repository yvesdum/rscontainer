//! Read/Write guards.

use crate::service::IService;
use crate::pointer::{IReadPointer, IWritePointer};

//////////////////////////////////////////////////////////////////////////////
// Read Guard
//////////////////////////////////////////////////////////////////////////////

/// Read-only access to a singleton without cloning the pointer.
pub struct ReadService<'a, T>
where
    T: IService + 'a,
    T::Pointer: IReadPointer<'a>
{
    pub(crate) _guard: <T::Pointer as IReadPointer<'a>>::ReadGuard
}

//////////////////////////////////////////////////////////////////////////////
// Write Guard
//////////////////////////////////////////////////////////////////////////////

/// Read/write access to a singleton without cloning the pointer.
pub struct WriteService<'a, T>
where
    T: IService + 'a,
    T::Pointer: IWritePointer<'a>
{
    pub(crate) _guard: <T::Pointer as IWritePointer<'a>>::WriteGuard
}