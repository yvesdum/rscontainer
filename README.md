# rscontainer

rscontainer is a library for the Rust language to manage dependencies between
objects. By implementing a trait, it is possible to recursively construct
all necessary types that a specific type needs. rscontainer provides the
following features:

  * Automatically construct objects and their dependencies recursively
  * Multiple crates can resolve the same global objects
  * Override default constructors to customize behaviour
  * Get access to many objects while only copying one reference
  * Inversion of Control without generic type parameters
  * Lazy initialization of singletons
  * Setup is optional, not required

rscontainer provides one main type: the ServiceContainer. This can be seen
as a registry for singleton instances and custom constructors. While resolving
an instance through the container, it will take care of injecting the required
dependencies. The container differentiates between local and shared instances.
Shared instances are always behind a smart pointer and a locking mechanism.
Which exact kinds kan be different for each type.

rscontainer provides a common interface for working with different smart
pointers, interior mutability types, locking mechanisms and poisoning.

## How does it work

Resolving a local instance:

```Rust
use rscontainer::ServiceContainer;

let mut container = ServiceContainer::new();
let mut foo = container.local::<SomeService>(()).unwrap();
foo.do_something();
```

Resolving a shared instance (singleton):

```Rust
use rscontainer::ServiceContainer;

let mut container = ServiceContainer::new();
let foo: Shared<SomeService> = container.shared().unwrap();

foo.access_mut(|foo| {
    let foo = foo.assert_healthy();
    foo.do_something();
});
```
