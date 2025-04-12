extern crate proc_macro;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{Ident, ItemImpl, ItemStruct, PathArguments, Type};

/// Tag a struct with the appropriate data to allow dynamic casting of that struct
///
/// Implementation details: injects a `::xdc::ObjBase` implementation for the struct,
/// including starting to build out the `MetadataEntry` array
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
            fn get_metadata(&self) -> &'static [::xdc::MetadataEntry] {
                &#meta_id
            }
        }
    };

    proc_macro::TokenStream::from(output)
}

/// Tag a trait impl with the appropriate data to allow dynamic casting to that implementation
///
/// Implementation details: adds one entry to the `MetadataEntry` array for this specific trait object
///
/// # Example
///
/// ```
/// use xdc::*;
/// trait Bar : ObjBase {}
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
