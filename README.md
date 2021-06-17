# rscontainer

rscontainer is a library for the Rust programming language to manage 
dependencies between objects. The main type is the `ServiceContainer`, which
serves two purposes: it acts as a registry for shared instances (singletons)
and custom constructors, and it provides a mechanism for dependency injection.

For more information see the documentation.

## Resolving instances

There are different kind of instances:

  * **Owned instances**: a fresh instance to be used in a owned scope. This
    instance will not be stored in the service container, you will get a new
    instance each time you resolve a owned instance.
  * **Shared instances**: an instance behind a smart pointer that is stored
    in the service container. You will get the same instance each time you
    resolve a shared service.

## How to use

Resolving a owned instance:

```Rust
use rscontainer::ServiceContainer;

let mut container = ServiceContainer::new();
let mut foo = container.resolver().owned::<SomeService>(()).unwrap();
foo.do_something();
```

Resolving a shared instance (singleton):

```Rust
use rscontainer::ServiceContainer;

let mut container = ServiceContainer::new();
let foo: Shared<SomeService> = container.resolver().shared().unwrap();

foo.access_mut(|foo| {
    let foo = foo.assert_healthy();
    foo.do_something();
});
```
