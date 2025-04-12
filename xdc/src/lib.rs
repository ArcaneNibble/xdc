#![no_std]

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
/// Trait assigned to all trait objects that can be casted by xdc
///
/// Ideally, we would be able to just use [core::any::TypeId], but
/// `std::any::TypeId::of<T>` is not `const fn`
/// ([tracking issue](https://github.com/rust-lang/rust/issues/77125)).
/// We require `const fn` in order to make metadata tables be fully generated
/// at compile time and embedded into the read-only section of the resulting binary.
///
/// Instead, we generate our own type IDs by incrementing an integer in our proc macro.
/// This is the primary cause of potential unsafety when using xdc across multiple crates
/// but is the simplest way of getting something working on stable.
pub trait TypeId {
    const TYPEID: u64;
}

/// Get the [TypeId] of a given type
pub const fn type_id<T: TypeId + ?Sized>() -> u64 {
    T::TYPEID
}

/// Base trait that will be added to all cast-able structs
pub trait ObjBase {
    fn get_metadata(&self) -> &'static [MetadataEntry];
}
impl TypeId for dyn ObjBase {
    const TYPEID: u64 = 0;
}

/// The metadata needed to allow for casting
pub struct MetadataEntry {
    /// The id of the type this metadata represents
    pub typeid: u64,
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
/// # Example
///
/// ```
/// use xdc::*;
/// #[xdc_trait]
/// trait Parent {}
/// #[xdc_trait]
/// trait Foo : Parent {}
/// #[xdc_trait]
/// trait Bar : Parent {}
/// #[xdc_struct]
/// struct Test {}
/// #[xdc_impl]
/// impl Parent for Test {}
/// #[xdc_impl]
/// impl Foo for Test {}
/// #[xdc_impl]
/// impl Bar for Test {}
///
/// let example = Test {};
/// let foo_example: &dyn Foo = &example;
/// let bar_example: &dyn Bar = xdc::try_cast(foo_example).unwrap();
/// ```
///
pub fn try_cast<T: ObjBase + TypeId + ?Sized>(from: &dyn ObjBase) -> Option<&T> {
    // look for the correct metadata entry
    let typeid = type_id::<T>();
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

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
/// # Example
///
/// ```
/// use xdc::*;
/// #[xdc_trait]
/// trait Parent {}
/// #[xdc_trait]
/// trait Foo : Parent {}
/// #[xdc_trait]
/// trait Bar : Parent {}
/// #[xdc_struct]
/// struct Test {}
/// #[xdc_impl]
/// impl Parent for Test {}
/// #[xdc_impl]
/// impl Foo for Test {}
/// #[xdc_impl]
/// impl Bar for Test {}
///
/// let mut example: Test = Test {};
/// let mut foo_example: &mut dyn Foo = &mut example;
/// let mut bar_example: &mut dyn Bar = xdc::try_cast_mut(foo_example).unwrap();
/// ```
///
pub fn try_cast_mut<T: ObjBase + TypeId + ?Sized>(from: &mut dyn ObjBase) -> Option<&mut T> {
    // look for the correct metadata entry
    let typeid = type_id::<T>();
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

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
/// # Example
///
/// ```
/// use xdc::*;
/// #[xdc_trait]
/// trait Parent {}
/// #[xdc_trait]
/// trait Foo : Parent {}
/// #[xdc_trait]
/// trait Bar : Parent {}
/// #[xdc_struct]
/// struct Test {}
/// #[xdc_impl]
/// impl Parent for Test {}
/// #[xdc_impl]
/// impl Foo for Test {}
/// #[xdc_impl]
/// impl Bar for Test {}
///
/// let example: Test = Test {};
/// let foo_example: Box<dyn Foo> = Box::new(example);
/// let bar_example: Box<dyn Bar> = xdc::try_cast_boxed(foo_example).unwrap();
/// ```
///
#[cfg(feature = "alloc")]
pub fn try_cast_boxed<T: ObjBase + TypeId + ?Sized>(from: Box<dyn ObjBase>) -> Option<Box<T>> {
    // look for the correct metadata entry
    let typeid = type_id::<T>();
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

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

pub use xdc_macros::*;
