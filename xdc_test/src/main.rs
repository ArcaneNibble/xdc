#![no_std]
extern crate alloc;
extern crate std;
use std::println;

use xdc::*;

trait HasId: ObjBase {
    fn id(&self) -> u32;
}

trait HasLocation: ObjBase + HasId {
    fn location(&self) -> (u32, u32);
}

trait HasColor: ObjBase + HasId {
    fn color(&self) -> u32;
}

trait HasTaste: ObjBase {
    fn taste(&self) -> &'static str;
}

struct Point {
    id: u32,
    x: u32,
    y: u32,
    col: u32,
}
xdc_struct!(Point);

impl HasId for Point {
    fn id(&self) -> u32 {
        println!("getting id {}", self.id);
        self.id
    }
}
xdc_impl!(HasId, Point);

impl HasLocation for Point {
    fn location(&self) -> (u32, u32) {
        println!("getting location {} {}", self.x, self.y);
        (self.x, self.y)
    }
}
xdc_impl!(HasLocation, Point);

impl HasColor for Point {
    fn color(&self) -> u32 {
        println!("getting color {}", self.col);
        self.col
    }
}
xdc_impl!(HasColor, Point);

mod annoying {
    use crate::*;

    pub struct FancyTest<T> {
        pub owo: T,
        pub uwu: &'static str,
    }

    impl HasId for FancyTest<u32> {
        fn id(&self) -> u32 {
            self.owo
        }
    }
    xdc_impl!(HasId, FancyTest<u32>);

    impl<T> HasTaste for FancyTest<T>
    where
        FancyTest<T>: xdc::ObjBase,
    {
        fn taste(&self) -> &'static str {
            self.uwu
        }
    }
    xdc_impl!(HasTaste, FancyTest<u32>);
}
xdc_struct!(annoying::FancyTest<u32>);
xdc_struct!(annoying::FancyTest<u64>);
xdc_impl!(HasTaste, annoying::FancyTest<u64>);

#[cfg(test)]
use alloc::boxed::Box;

#[test]
fn test_const() {
    let test = Point {
        id: 123,
        x: 1,
        y: 2,
        col: 3,
    };
    let test_as_haslocation: &dyn HasLocation = &test;
    assert_eq!(test_as_haslocation.location(), (1, 2));
    assert_eq!(test_as_haslocation.id(), 123);
    let test_cast: &dyn HasColor = xdc::try_cast(test_as_haslocation).unwrap();
    assert_eq!(test_cast.color(), 3);
    assert_eq!(test_cast.id(), 123);
}

#[test]
fn test_mut() {
    let mut test = Point {
        id: 123,
        x: 1,
        y: 2,
        col: 3,
    };
    let test_as_haslocation: &mut dyn HasLocation = &mut test;
    assert_eq!(test_as_haslocation.location(), (1, 2));
    assert_eq!(test_as_haslocation.id(), 123);
    let test_cast: &mut dyn HasColor = xdc::try_cast_mut(test_as_haslocation).unwrap();
    assert_eq!(test_cast.color(), 3);
    assert_eq!(test_cast.id(), 123);
}

#[test]
fn test_boxed() {
    let test = Point {
        id: 123,
        x: 1,
        y: 2,
        col: 3,
    };
    let test_as_haslocation: Box<dyn HasLocation> = Box::new(test);
    assert_eq!(test_as_haslocation.location(), (1, 2));
    assert_eq!(test_as_haslocation.id(), 123);
    let test_cast: Box<dyn HasColor> = xdc::try_cast_boxed(test_as_haslocation).unwrap();
    assert_eq!(test_cast.color(), 3);
    assert_eq!(test_cast.id(), 123);
}

#[test]
fn test_bad_cast() {
    let test = Point {
        id: 123,
        x: 1,
        y: 2,
        col: 3,
    };
    let test_as_haslocation: &dyn HasLocation = &test;
    let test_cast: Option<&dyn HasTaste> = xdc::try_cast(test_as_haslocation);
    assert!(test_cast.is_none());
}

#[test]
fn test_fancy_1() {
    let test = annoying::FancyTest {
        owo: 123u32,
        uwu: "spicy",
    };
    let test_as_objbase: &dyn ObjBase = &test;
    let test_cast_1: &dyn HasId = xdc::try_cast(test_as_objbase).unwrap();
    assert_eq!(test_cast_1.id(), 123);
    let test_cast_2: &dyn HasTaste = xdc::try_cast(test_cast_1).unwrap();
    assert_eq!(test_cast_2.taste(), "spicy");

    let test_cast: Option<&dyn HasColor> = xdc::try_cast(test_cast_2);
    assert!(test_cast.is_none());
    let test_cast: Option<&dyn HasLocation> = xdc::try_cast(test_cast_2);
    assert!(test_cast.is_none());
}

#[test]
fn test_fancy_2() {
    let test = annoying::FancyTest {
        owo: 456u64,
        uwu: "salty",
    };
    let test_as_objbase: &dyn ObjBase = &test;
    let test_cast_1: &dyn HasTaste = xdc::try_cast(test_as_objbase).unwrap();
    assert_eq!(test_cast_1.taste(), "salty");

    let test_cast: Option<&dyn HasId> = xdc::try_cast(test_cast_1);
    assert!(test_cast.is_none());
    let test_cast: Option<&dyn HasColor> = xdc::try_cast(test_cast_1);
    assert!(test_cast.is_none());
    let test_cast: Option<&dyn HasLocation> = xdc::try_cast(test_cast_1);
    assert!(test_cast.is_none());
}

fn main() {
    println!("Hello, world!");
}
