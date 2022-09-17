extern crate proc_macro;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use std::sync::atomic::{AtomicU64, Ordering};
use syn::{ItemTrait, TypeParamBound};

static NEXT_UID: AtomicU64 = AtomicU64::new(1);

#[proc_macro_attribute]
#[proc_macro_error]
pub fn xdc_trait(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let mut input_parsed = match syn::parse2::<ItemTrait>(input) {
        Ok(x) => x,
        Err(e) => return proc_macro::TokenStream::from(e.to_compile_error()),
    };

    if !input_parsed.generics.params.is_empty() {
        abort!(input_parsed.generics, "Cannot have generics here (including const generics)")
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
