extern crate proc_macro;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use std::sync::atomic::{AtomicU64, Ordering};
use syn::{Ident, ItemImpl, ItemStruct, ItemTrait, PathArguments, Type, TypeParamBound};

static NEXT_UID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "std")]
fn std_is_enabled() -> bool {
    true
}

#[cfg(not(feature = "std"))]
fn std_is_enabled() -> bool {
    false
}

#[cfg(feature = "alloc")]
fn alloc_is_enabled() -> bool {
    true
}

#[cfg(not(feature = "alloc"))]
fn alloc_is_enabled() -> bool {
    false
}

/// Tag a trait with the appropriate data to allow dynamic casting to that trait
/// 
/// # Example
/// 
/// ```
/// use xdc::*;
/// #[xdc_trait]
/// trait Foo {}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn xdc_trait(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let mut input_parsed = match syn::parse2::<ItemTrait>(input) {
        Ok(x) => x,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    if !input_parsed.generics.params.is_empty() {
        // if (input_parsed.generics.p)
        abort!(
            input_parsed.generics,
            "Cannot have generics here (including const generics)"
        )
    }

    let trait_id = input_parsed.ident.clone();
    let trait_uid: u64 = NEXT_UID.fetch_add(1, Ordering::SeqCst);

    let objbase_bound = syn::parse2::<TypeParamBound>(quote! { ::xdc::ObjBase }).unwrap();
    input_parsed.supertraits.push(objbase_bound);

    let output = quote! {
        #input_parsed

        impl ::xdc::TypeId for dyn #trait_id {
            const TYPEID: u64 = #trait_uid;
        }
    };

    proc_macro::TokenStream::from(output)
}

/// Tag a struct with the appropriate data to allow dynamic casting of that struct
/// 
/// # Example
/// 
/// ```
/// use xdc::*;
/// #[xdc_struct]
/// struct Foo {}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn xdc_struct(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let input_parsed = match syn::parse2::<ItemStruct>(input) {
        Ok(x) => x,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let struct_id = input_parsed.ident.clone();
    let meta_id = Ident::new(&format!("__{}_XDC_METADATA", struct_id), struct_id.span());
    let objbase_id = Ident::new(
        &format!("__{}_XDC_METADATA_OBJBASE", struct_id),
        struct_id.span(),
    );

    let alloc_impl = if std_is_enabled() {
        quote! {
            fn to_base_boxed(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::xdc::ObjBase> {
                self
            }
        }
    } else if alloc_is_enabled() {
        quote! {
            fn to_base_boxed(self: ::alloc::boxed::Box<Self>) -> ::alloc::boxed::Box<dyn ::xdc::ObjBase> {
                self
            }
        }
    } else {
        quote! {}
    };

    let output = quote! {
        #input_parsed

        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice]
        #[linkme(crate = ::xdc::linkme)]
        static #meta_id: [::xdc::MetadataEntry] = [..];

        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice(#meta_id)]
        #[linkme(crate = ::xdc::linkme)]
        static #objbase_id: ::xdc::MetadataEntry = ::xdc::metadata_entry!(#struct_id, ::xdc::ObjBase);

        impl ::xdc::ObjBase for #struct_id {
            fn to_base(self: &Self) -> &dyn ::xdc::ObjBase {
                self
            }
            fn to_base_mut(self: &mut Self) -> &mut dyn ::xdc::ObjBase {
                self
            }
            #alloc_impl

            fn get_metadata(&self) -> &'static [::xdc::MetadataEntry] {
                &#meta_id
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

/// Tag a trait with the appropriate data to allow dynamic casting to that implementation
/// 
/// # Example
/// 
/// ```
/// use xdc::*;
/// #[xdc_trait]
/// trait Bar {}
/// #[xdc_struct]
/// struct Foo {}
/// #[xdc_impl]
/// impl Bar for Foo {}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn xdc_impl(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let input_parsed = match syn::parse2::<ItemImpl>(input) {
        Ok(x) => x,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    let (bang, trait_, _for) = match &input_parsed.trait_ {
        Some(x) => x,
        None => abort!(input_parsed, "Must impl some trait"),
    };
    if bang.is_some() {
        abort!(bang, "Cannot have a ! here");
    }
    let on_type = match input_parsed.self_ty.as_ref() {
        Type::Path(x) => x,
        _ => abort!(input_parsed.self_ty, "Must impl on a path"),
    };

    // make our necessary idents
    if let Some(_) = &on_type.qself {
        abort!(
            on_type,
            "Cannot have qualified syntax here (must be bare ident)"
        )
    }
    let on_type = match on_type.path.get_ident() {
        Some(x) => x,
        None => abort!(on_type, "Cannot have a path here (must be bare ident)"),
    };
    let meta_id = Ident::new(&format!("__{}_XDC_METADATA", on_type), on_type.span());

    let entry_path_concat = trait_
        .segments
        .iter()
        .map(|ps| {
            if ps.arguments != PathArguments::None {
                abort!(ps, "Cannot have arguments here");
            }
            ps.ident.to_string()
        })
        .fold(String::new(), |mut s, id| {
            s.push_str("_");
            s.push_str(&id);
            s
        });
    let entry_id = Ident::new(
        &format!("__{}_XDC_METADATA{}", on_type, entry_path_concat),
        on_type.span(),
    );

    // output
    let output = quote! {
        #input_parsed

        #[allow(non_upper_case_globals)]
        #[::xdc::linkme::distributed_slice(#meta_id)]
        #[linkme(crate = ::xdc::linkme)]
        static #entry_id: ::xdc::MetadataEntry = ::xdc::metadata_entry!(#on_type, #trait_);
    };

    proc_macro::TokenStream::from(output)
}
