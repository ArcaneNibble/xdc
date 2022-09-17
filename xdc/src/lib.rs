#![no_std]
extern crate alloc;
use alloc::boxed::Box;

#[repr(C)]
pub struct FatPointer {
    pub data: *const u8,
    pub vtable: *const u8,
}

pub trait TypeId {
    const TYPEID: u64;
}
pub const fn type_id<T: TypeId + ?Sized>() -> u64 { T::TYPEID }

pub trait ObjBase {
    fn to_base(self: &Self) -> &dyn ObjBase;
    fn to_base_mut(self: &mut Self) -> &mut dyn ObjBase;
    fn to_base_boxed(self: Box<Self>) -> Box<dyn ObjBase>;
    fn get_metadata(&self) -> &'static [MetadataEntry];
}
impl TypeId for dyn ObjBase {
    const TYPEID: u64 = 0;
}


pub struct MetadataEntry {
    pub typeid: u64,
    pub vtable: *const u8,
}
unsafe impl Sync for MetadataEntry {}

#[macro_export]
macro_rules! metadata_entry {
    ($type:ty, $trait:path) => {
        MetadataEntry {
            typeid: type_id::<dyn $trait>(),
            vtable: unsafe {
                ::core::mem::transmute::<*const dyn $trait, ::xdc::FatPointer>(
                    ::core::mem::MaybeUninit::<$type>::uninit().as_ptr() as *const dyn $trait).vtable
            }
        }
    }
}


pub fn try_cast<T: ObjBase + ?Sized>(from: &dyn ObjBase, typeid: u64) -> Option<&T>{
    // look for the correct vtable
    let mut vtable = core::ptr::null();

    let meta = from.get_metadata();
    for meta_ent in meta {
        if meta_ent.typeid == typeid {
            vtable = meta_ent.vtable;
            break;
        }
    }

    if vtable == core::ptr::null() {
        return None;
    }

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr = core::mem::transmute::<*const dyn ObjBase, FatPointer>(from).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable,
        };
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *const T>(&casted_object);
        Some(&*new_trait_ptr)
    }
}
#[macro_export]
macro_rules! try_cast {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base($val);
        let ret: Option<&dyn $type> = ::xdc::try_cast(objbase, xdc::type_id::<dyn $type>());
        ret
    }}
}

pub fn try_cast_mut<T: ObjBase + ?Sized>(from: &mut dyn ObjBase, typeid: u64) -> Option<&mut T>{
    // look for the correct vtable
    let mut vtable = core::ptr::null();

    let meta = from.get_metadata();
    for meta_ent in meta {
        if meta_ent.typeid == typeid {
            vtable = meta_ent.vtable;
            break;
        }
    }

    if vtable == core::ptr::null() {
        return None;
    }

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr = core::mem::transmute::<*mut dyn ObjBase, FatPointer>(from).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable,
        };
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object);
        Some(&mut *new_trait_ptr)
    }
}
#[macro_export]
macro_rules! try_cast_mut {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base_mut($val);
        let ret: Option<&mut dyn $type> = ::xdc::try_cast_mut(objbase, xdc::type_id::<dyn $type>());
        ret
    }}
}

pub fn try_cast_boxed<T: ObjBase + ?Sized>(from: Box<dyn ObjBase>, typeid: u64) -> Option<Box<T>>{
    // look for the correct vtable
    let mut vtable = core::ptr::null();

    let meta = from.get_metadata();
    for meta_ent in meta {
        if meta_ent.typeid == typeid {
            vtable = meta_ent.vtable;
            break;
        }
    }

    if vtable == core::ptr::null() {
        return None;
    }

    // vtable found, do transmuting
    unsafe {
        let from_data_ptr = core::mem::transmute::<*mut dyn ObjBase, FatPointer>(Box::into_raw(from)).data;
        let casted_object = FatPointer {
            data: from_data_ptr,
            vtable,
        };
        let new_trait_ptr = core::mem::transmute_copy::<FatPointer, *mut T>(&casted_object);
        Some(Box::from_raw(new_trait_ptr))
    }
}
#[macro_export]
macro_rules! try_cast_boxed {
    ($type:path, $val:expr) => {{
        let objbase = ::xdc::ObjBase::to_base_boxed($val);
        let ret: Option<Box<dyn $type>> = ::xdc::try_cast_boxed(objbase, xdc::type_id::<dyn $type>());
        ret
    }}
}
