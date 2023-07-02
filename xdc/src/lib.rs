#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub use linkme;

/// The rust fat pointer structure as defined by the compiler
///
/// May break in non standard rust implementation
#[repr(C)]
pub struct FatPointer {
    pub data: *const u8,
    pub vtable: *const u8,
}
/// Trait assigned to all structs that can be casted
pub trait TypeId {
    const TYPEID: u64;
}

/// Get the type_id of a given type
pub const fn type_id<T: TypeId + ?Sized>() -> u64 {
    T::TYPEID
}

pub trait ObjBase {
    fn to_base(self: &Self) -> &dyn ObjBase;
    fn to_base_mut(self: &mut Self) -> &mut dyn ObjBase;
    #[cfg(feature = "alloc")]
    fn to_base_boxed(self: Box<Self>) -> Box<dyn ObjBase>;
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

/// The underlying implementation for dynamc casting
///
/// It is HIGHLY recommended to use the macro [try_cast!]
/// instead as it is more user friendly, and less likely
/// to produce illegal behaviour.
///
/// # Types
///
/// * `T` - The type to cast to; should be in the form `&dyn Trait`
///
/// # Arguments
///
/// * `from` - The object to cast from
/// * `typeid` - The type id of `T` found using [type_id]
///
pub fn try_cast<T: ObjBase + ?Sized>(from: &dyn ObjBase, typeid: u64) -> Option<&T> {
    // look for the correct metadata entry
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr = core::mem::transmute::<*const dyn ObjBase, FatPointer>(from).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable: meta_ent.vtable,
        };
        assert_eq!(
            core::mem::size_of::<FatPointer>(),
            core::mem::size_of::<*const T>()
        );
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *const T>(&casted_object);
        Some(&*new_trait_ptr)
    }
}

/// The intended userfacing way to cast between immutable trait objects
///
/// # Arguments
///
/// * `type` - The raw trait you want to cast to
/// * `val` - The trait object you wish to cast
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
/// let bar_example: &dyn Bar = xdc::try_cast!(Bar, foo_example).unwrap();
/// ```
///
#[macro_export]
macro_rules! try_cast {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base($val);
        let ret: Option<&dyn $type> = ::xdc::try_cast(objbase, xdc::type_id::<dyn $type>());
        ret
    }};
}

/// The underlying implementation for dynamc casting mutably
///
/// It is HIGHLY recommended to use the macro [try_cast_mut!]
/// instead as it is more user friendly, and less likely
/// to produce illegal behaviour.
///
/// # Types
///
/// * `T` - The type to cast to; should be in the form `&mut dyn Trait`
///
/// # Arguments
///
/// * `from` - The object to cast from
/// * `typeid` - The type id of `T` found using [type_id]
///
pub fn try_cast_mut<T: ObjBase + ?Sized>(from: &mut dyn ObjBase, typeid: u64) -> Option<&mut T> {
    // look for the correct metadata entry
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr = core::mem::transmute::<*mut dyn ObjBase, FatPointer>(from).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable: meta_ent.vtable,
        };
        assert_eq!(
            core::mem::size_of::<FatPointer>(),
            core::mem::size_of::<*mut T>()
        );
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object);
        Some(&mut *new_trait_ptr)
    }
}

/// The intended userfacing way to cast between immutable trait objects
///
/// # Arguments
///
/// * `type` - The raw trait you want to cast to
/// * `val` - The trait object you wish to cast
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
/// let mut bar_example: &mut dyn Bar = xdc::try_cast_mut!(Bar, foo_example).unwrap();
/// ```
///
#[macro_export]
macro_rules! try_cast_mut {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base_mut($val);
        let ret: Option<&mut dyn $type> = ::xdc::try_cast_mut(objbase, xdc::type_id::<dyn $type>());
        ret
    }};
}

#[cfg(feature = "alloc")]
pub fn try_cast_boxed<T: ObjBase + ?Sized>(from: Box<dyn ObjBase>, typeid: u64) -> Option<Box<T>> {
    // look for the correct metadata entry
    let meta_ent = from.get_metadata().iter().find(|x| x.typeid == typeid)?;

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr =
            core::mem::transmute::<*mut dyn ObjBase, FatPointer>(Box::into_raw(from)).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable: meta_ent.vtable,
        };
        assert_eq!(
            core::mem::size_of::<FatPointer>(),
            core::mem::size_of::<*mut T>()
        );
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object);
        Some(Box::from_raw(new_trait_ptr))
    }
}
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! try_cast_boxed {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base_boxed($val);
        let ret: Option<Box<dyn $type>> =
            ::xdc::try_cast_boxed(objbase, xdc::type_id::<dyn $type>());
        ret
    }};
}

pub use xdc_macros::*;
