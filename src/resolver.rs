//! Resolver for the service container.

use crate::{ILocal, IShared, Instance, ServiceContainer, Shared};

/// Used to resolve services from the service container.
///
/// Acts as a safety measure. When the service container is supplied as a
/// mutable reference, it is possible to replace the entire container with
/// another one, shadowing all the services inside it. Malicious services
/// could abuse this to manipulate the behaviour of the entire program. Through
/// a resolver, only services can be resolved. It is not possible to override
/// existing shared instances.
///
/// It is preferred that the resolver is passed by value. If this is not
/// possible, passing by reference is still secure. It is not possible to
/// shadow the resolver as it cannot be initialized from outside the
/// rscontainer crate.
#[derive(Debug)]
pub struct Resolver<'ctn> {
    ctn: &'ctn mut ServiceContainer,
}

impl<'ctn> Resolver<'ctn> {
    /// Creates a new resolver.
    ///
    /// It's very important that this is `pub(crate)` to prevent users from
    /// creating it.
    pub(crate) fn new(ctn: &'ctn mut ServiceContainer) -> Self {
        Self { ctn }
    }

    /// Resolves a [`Shared`].
    pub fn shared<S: ?Sized + IShared + 'static>(&mut self) -> Result<Shared<S>, S::Error> {
        match self.ctn.resolve_shared::<S>() {
            Ok(s) => Ok(Shared::new(s)),
            Err(e) => Err(e),
        }
    }

    /// Resolves a local instance.
    pub fn local<S: ?Sized + ILocal + 'static>(
        &mut self,
        params: S::Parameters,
    ) -> Result<S::Instance, S::Error> {
        self.ctn.resolve_local::<S>(params)
    }

    /// Resolves an [`Instance::Shared`].
    pub fn shared_instance<S: ?Sized + IShared + ILocal + 'static>(
        &mut self,
    ) -> Result<Instance<S>, <S as IShared>::Error> {
        match self.ctn.resolve_shared::<S>() {
            Ok(s) => Ok(Instance::from_shared(s)),
            Err(e) => Err(e),
        }
    }

    /// Resolves an [`Instance::Local`].
    pub fn local_instance<S: ?Sized + IShared + ILocal + 'static>(
        &mut self,
        params: S::Parameters,
    ) -> Result<Instance<S>, <S as ILocal>::Error> {
        match self.ctn.resolve_local::<S>(params) {
            Ok(l) => Ok(Instance::from_local(l)),
            Err(e) => Err(e)
        }
    }
}
