//! eXperimental Dynamic Casting for Rust
//!
//! # Example
//!
//! ```
//! use xdc::*;
//! trait Parent : ObjBase {}
//! trait Foo : Parent {}
//! trait Bar : Parent {}
//!
//! struct Test {}
//! xdc_struct!(Test);
//!
//! impl Parent for Test {}
//! xdc_impl!(Parent, Test);
//!
//! impl Foo for Test {}
//! xdc_impl!(Foo, Test);
//!
//! impl Bar for Test {}
//! xdc_impl!(Bar, Test);
//!
//! let mut example = Test {};
//!
//! let foo_example: &dyn Foo = &example;
//! let bar_example: &dyn Bar = xdc::try_cast(foo_example).unwrap();
//!
//! let foo_example_mut: &mut dyn Foo = &mut example;
//! let bar_example_mut: &mut dyn Bar = xdc::try_cast_mut(foo_example_mut).unwrap();
//!
//! let foo_example_box: Box<dyn Foo> = Box::new(example);
//! let bar_example_box: Box<dyn Bar> = xdc::try_cast_boxed(foo_example_box).unwrap();
//! ```

#![no_std]

use core::any::TypeId;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub use linkme;

/// The Rust fat pointer structure as defined by the compiler
///
/// This was at one point `std::raw::TraitObject` and is now `#![feature(ptr_metadata)]`,
/// but neither of those are available on stable.
///
/// This will break if rustc ever changes this layout.
#[repr(C)]
pub struct FatPointer {
    pub data: *const u8,
    pub vtable: *const u8,
}

/// This function is used as a workaround to get access to [TypeId] in a `const` context
///
/// see https://github.com/rust-lang/rust/issues/77125#issuecomment-2799067806
fn type_id_workaround<T: ?Sized + 'static>() -> TypeId {
    TypeId::of::<T>()
}

/// Get a workaround for the [TypeId] of a given type
pub const fn type_id<T: ?Sized + 'static>() -> fn() -> TypeId {
    type_id_workaround::<T>
}

/// Base trait that will be added to all cast-able structs
pub trait ObjBase {
    fn get_metadata(&self) -> &'static [MetadataEntry];
}

/// The metadata needed to allow for casting
pub struct MetadataEntry {
    /// The id of the type this metadata represents
    pub typeid: fn() -> TypeId,
    /// The vtable the type uses
    pub vtable: *const u8,
}
unsafe impl Sync for MetadataEntry {}

/// Will be used to generate a [MetadataEntry] describing how to cast to
/// "`$type` struct being referred to using a `$trait` trait object pointer"
#[macro_export]
macro_rules! metadata_entry {
    ($type:ty, $trait:path) => {
        ::xdc::MetadataEntry {
            typeid: ::xdc::type_id::<dyn $trait>(),
            vtable: unsafe {
                ::core::mem::transmute::<*const dyn $trait, ::xdc::FatPointer>(
                    ::core::mem::MaybeUninit::<$type>::uninit().as_ptr() as *const dyn $trait,
                )
                .vtable
            },
        }
    };
}

/// Dynamic cast between immutable trait objects
///
/// # Types
///
/// * `T` - The trait type to cast to. Should be a `dyn Trait`
///
/// # Arguments
///
/// * `from` - The object to cast
///
pub fn try_cast<T: ObjBase + ?Sized + 'static>(from: &dyn ObjBase) -> Option<&T> {
    // look for the correct metadata entry
    let typeid = TypeId::of::<T>();
    let meta_ent = from
        .get_metadata()
        .iter()
        .find(|x| (x.typeid)() == typeid)?;

    // vtable found, do transmuting
    let from_data_ptr =
        unsafe { core::mem::transmute::<*const dyn ObjBase, FatPointer>(from) }.data;
    let casted_object = FatPointer {
        data: from_data_ptr,
        vtable: meta_ent.vtable,
    };
    assert_eq!(
        core::mem::size_of::<FatPointer>(),
        core::mem::size_of::<*const T>()
    );
    let new_trait_ptr =
        unsafe { core::mem::transmute_copy::<FatPointer, *const T>(&casted_object) };
    Some(unsafe { &*new_trait_ptr })
}

/// Dynamic cast between mutable trait objects
///
/// # Types
///
/// * `T` - The trait type to cast to. Should be a `dyn Trait`
///
/// # Arguments
///
/// * `from` - The object to cast
///
pub fn try_cast_mut<T: ObjBase + ?Sized + 'static>(from: &mut dyn ObjBase) -> Option<&mut T> {
    // look for the correct metadata entry
    let typeid = TypeId::of::<T>();
    let meta_ent = from
        .get_metadata()
        .iter()
        .find(|x| (x.typeid)() == typeid)?;

    // vtable found, do transmuting
    let from_data_ptr = unsafe { core::mem::transmute::<*mut dyn ObjBase, FatPointer>(from) }.data;
    let casted_object = FatPointer {
        data: from_data_ptr,
        vtable: meta_ent.vtable,
    };
    assert_eq!(
        core::mem::size_of::<FatPointer>(),
        core::mem::size_of::<*mut T>()
    );
    let new_trait_ptr = unsafe { core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object) };
    Some(unsafe { &mut *new_trait_ptr })
}

/// Dynamic cast between [Box]ed trait objects
///
/// # Types
///
/// * `T` - The trait type to cast to. Should be a `dyn Trait`
///
/// # Arguments
///
/// * `from` - The object to cast
///
#[cfg(feature = "alloc")]
pub fn try_cast_boxed<T: ObjBase + ?Sized + 'static>(from: Box<dyn ObjBase>) -> Option<Box<T>> {
    // look for the correct metadata entry
    let typeid = TypeId::of::<T>();
    let meta_ent = from
        .get_metadata()
        .iter()
        .find(|x| (x.typeid)() == typeid)?;

    // vtable found, do transmuting
    let from_data_ptr =
        unsafe { core::mem::transmute::<*mut dyn ObjBase, FatPointer>(Box::into_raw(from)) }.data;
    let casted_object = FatPointer {
        data: from_data_ptr,
        vtable: meta_ent.vtable,
    };
    assert_eq!(
        core::mem::size_of::<FatPointer>(),
        core::mem::size_of::<*mut T>()
    );
    let new_trait_ptr = unsafe { core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object) };
    Some(unsafe { Box::from_raw(new_trait_ptr) })
}

pub use mident::mident;

#[macro_export]
macro_rules! xdc_struct_ {
    ($ty:path, $meta:ident, $dummy:ident) => {
        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice]
        #[linkme(crate = ::xdc::linkme)]
        static $meta: [::xdc::MetadataEntry] = [..];

        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice($meta)]
        #[linkme(crate = ::xdc::linkme)]
        static $dummy: ::xdc::MetadataEntry = ::xdc::metadata_entry!($ty, ::xdc::ObjBase);

        impl ::xdc::ObjBase for $ty {
            fn get_metadata(&self) -> &'static [::xdc::MetadataEntry] {
                &$meta
            }
        }
    };
}

#[macro_export]
macro_rules! xdc_struct {
    ($ty:path) => {
        ::xdc::mident! {
            ::xdc::xdc_struct_! {
            $ty,
            #concat(__ #flatten_basename($ty) _XDC_METADATA),
            #rand
            }
        }
    };
}

#[macro_export]
macro_rules! xdc_impl_ {
    ($trait:path, $obj:path, $meta:path, $dummy:ident) => {
        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice($meta)]
        #[linkme(crate = ::xdc::linkme)]
        static $dummy: ::xdc::MetadataEntry = ::xdc::metadata_entry!($obj, $trait);
    };
}

#[macro_export]
macro_rules! xdc_impl {
    ($trait:path, $obj:path) => {
        ::xdc::mident! {
            ::xdc::xdc_impl_! {
                $trait,
                $obj,
                #concat(__ #flatten_basename($obj) _XDC_METADATA),
                #rand
            }
        }
    };
}
