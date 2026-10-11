//! Procedural macros supporting [Neon](https://docs.rs/neon/latest/neon/)

mod export;

#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let syn::ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = syn::parse_macro_input!(item as syn::ItemFn);

    let name = &sig.ident;
    let export_name = quote::format_ident!("__NEON_MAIN__{name}");
    // WEB-FORK: wasm registers the entry point through `inventory`.
    let export_fn = quote::quote!({
        #[cfg_attr(not(target_family = "wasm"), neon::macro_internal::linkme::distributed_slice(neon::macro_internal::MAIN))]
        #[cfg_attr(not(target_family = "wasm"), linkme(crate = neon::macro_internal::linkme))]
        fn #export_name(cx: neon::context::ModuleContext) -> neon::result::NeonResult<()> {
            #name(cx)
        }
        #[cfg(target_family = "wasm")]
        neon::macro_internal::inventory::submit! {
            neon::macro_internal::Main(#export_name)
        }
    });

    quote::quote!(
        #(#attrs) *
        #vis #sig {
            #export_fn
            #block
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn export(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    export::export(attr, item)
}
