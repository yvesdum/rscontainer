# rscontainer

rscontainer is a library that manages intialization and lifetimes of objects 
that depend on other objects (services). This crate provides:
  * a registry for shared instances (singletons), the `ServiceContainer`,
  * a mechanism for [dependency injection], which separates code that
    initializes objects and code that uses objects,
  * a mechanism for [inversion of control] through overridable constructors and
  * lazy initialization of shared objects.

rscontainer finds its use cases in the following scenarios:
  * Multiple crates need access to the same shared or global object;
  * You want such an object to be lazy initialized;
  * You want the user to control the configuration and initialization of
    certain objects;
  * You need to pass many objects around in a deeply nested code
    structure, such as the event loop of GUI applications;
  * You find that the complex initialization code for an object with many
    dependencies clutters your code.

## Instances

An object that is able to be initialized through the service container is 
called a service. When initializing a service, the service container 
recursively initializes its dependencies. This process is called *resolving*.
A service can be resolved as different kind of instances:
  * **Owned instances**: a regular owned instance which will be destroyed if
    you no longer need it;
  * **Shared instances**: a shared instance is always behind a smart pointer
    (such as [`Rc`] or [`Arc`]) and optionally a locking or borrowing mechanism.
    When it is resolved the first time, the pointer is stored in the service
    container. Each next time that same pointer will be cloned and returned.
    The kind of smart pointer is chosen by the services themselves.

For more information see the [documentation].

## Example

```rust
use rscontainer::{ServiceContainer, Resolver, IShared, IOwned, Shared};
use std::sync::{Arc, Mutex};
use std::time::Instant;

enum LogService {}

impl IOwned for LogService {
    type Instance = Vec<Instant>;
    type Parameters = ();
    type Error = ();

    fn construct(
        _: Resolver, 
        _: Self::Parameters
    ) -> Result<Self::Instance, Self::Error> {
        Ok(Vec::new())
    }
}

struct Counter {
    value: u32,
    log: Vec<Instant>,
}

impl Counter {
    fn increase(&mut self) {
        self.value += 1;
        self.log.push(Instant::now());
    }
}

impl IShared for Counter {
    type Pointer = Arc<Mutex<Counter>>;
    type Target = Counter;
    type Error = ();

    fn construct(mut r: Resolver) -> Result<Self::Pointer, Self::Error> {
        Ok(Arc::new(Mutex::new(Counter {
            value: 0,
            log: r.owned::<LogService>(())?,
        })))
    }
}

fn main() -> Result<(), ()> {
    let mut container = ServiceContainer::new();

    // Initialize the counter service and recursively intialize an owned
    // instance of the log service and inject it in the counter service.
    let counter: Shared<Counter> = container.resolver().shared()?;

    counter.access_mut(|instance| {
        instance.assert_healthy().increase();
    });

    let timestamps = counter.access(|instance| {
        let counter = instance.assert_healthy();
        assert_eq!(counter.value, 1);
        counter.log.clone()
    });

    println!("Timestamps: {:?}", timestamps);

    Ok(())
}
```

[dependency injection]: https://en.wikipedia.org/wiki/Dependency_injection
[inversion of control]: https://en.wikipedia.org/wiki/Inversion_of_control
[`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[documentation]: https://docs.rs/rscontainer