use proc_macro::TokenStream;
use syn::{parse_macro_input, };

#[proc_macro_attribute]
pub fn profile_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let vis = input.vis;
    let sig = input.sig;
    let block = input.block;
    let name = &sig.ident;

    if sig.asyncness.is_some() {
        return syn::Error::new_spanned(
            &sig.asyncness,
            "async functions are not supported"
        ).to_compile_error().into()
    }

    quote::quote! {
        #vis #sig {
            let __start = std::time::Instant::now();

            let __ret = (||#block)();

            let __sec_elapsed = __start.elapsed().as_secs();
            if __sec_elapsed > 0 {
                return Err(::common::Error::TamperingDetected(
                    stringify!(#name).to_string(),
                    __sec_elapsed)
                );
            }

            __ret
        }
    }.into()
}