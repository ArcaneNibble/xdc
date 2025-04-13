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
    #[allow(dead_code)]
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

fn main() {
    println!("Hello, world!");
}
