use proc_macro::TokenStream;
use syn::{parse_macro_input};

#[proc_macro_attribute]
pub fn check_tampering(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

            let __elapsed = __start.elapsed();
            if __elapsed.as_secs() > 5 {
                return Err(tampering_profiler_support::Errors::TamperingDetected(
                    stringify!(#name).to_string(),
                    __elapsed.as_secs()).into()
                );
            }

            __ret
        }
    }.into()
}