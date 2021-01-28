use super::*;
use std::{cell::RefCell, rc::Rc};

///////////////////////////////////////////////////////////////////////////////
// Test Services
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Service(pub u32);

impl IService for Service {
    type Pointer = Rc<RefCell<Service>>;
    type Instance = Service;
    type Params = u32;
    type Error = ();
    // type DefaultInstance = Local<Service>;

    fn new_singleton(_: &mut ServiceContainer) -> Result<Global<Self>, Self::Error> {
        Ok(Global::new(Rc::new(RefCell::new(Service(100)))))
    }

    fn new_local(_: &mut ServiceContainer, number: Self::Params) -> Result<Local<Self>, Self::Error> {
        Ok(Local::new(Service(number)))
    }
}

///////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////

#[test]
fn construction() {
    let ctn = ServiceContainer::new();
    assert_eq!(ctn.inner_hashmap().len(), 0);
}

#[test]
fn insert() {
    let mut ctn = ServiceContainer::new();
    let service = Service::new_singleton(&mut ctn).unwrap();
    ctn.insert::<Service>(service);
    assert_eq!(ctn.inner_hashmap().len(), 1);
}

#[test]
fn resolve_singleton_once() {
    let mut ctn = ServiceContainer::new();
    let singleton = ctn.resolve_global::<Service>();
    assert!(singleton.is_ok())
}

#[test]
fn resolve_singleton_multiple_times() {
    static mut COUNT: usize = 0;

    struct Service2(u32);

    impl IService for Service2 {
        type Pointer = Rc<RefCell<Service2>>;
        type Instance = Service2;
        type Params = u32;
        type Error = ();

        fn new_singleton(_: &mut ServiceContainer) -> Result<Singleton<Self>, Self::Error> {
            unsafe { COUNT+= 1 };
            Ok(Singleton::new(Rc::new(RefCell::new(Service2(100)))))
        }

        fn new_local(_: &mut ServiceContainer, number: Self::Params) -> Result<Local<Self>, Self::Error> {
            Ok(Local::new(Service2(number)))
        }
    }

    let mut ctn = ServiceContainer::new();
    let _singleton1 = ctn.resolve_global::<Service2>();
    let _singleton2 = ctn.resolve_global::<Service2>();
    let _singleton3 = ctn.resolve_global::<Service2>();
    
    assert_eq!(ctn.inner_hashmap().len(), 1);

    unsafe {
        assert_eq!(COUNT, 1);
    }
}