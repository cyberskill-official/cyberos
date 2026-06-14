//! Proc macros for `cyberos-obs-sdk`.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Error, Ident, ItemFn, LitStr, Result, Token};

struct RedArgs {
    service: LitStr,
    route: LitStr,
}

impl Parse for RedArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut service = None;
        let mut route = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            match key.to_string().as_str() {
                "service" => service = Some(value),
                "route" => route = Some(value),
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!("unsupported red_instrument arg `{other}`"),
                    ));
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self {
            service: service.ok_or_else(|| input.error("missing `service = \"...\"`"))?,
            route: route.ok_or_else(|| input.error("missing `route = \"...\"`"))?,
        })
    }
}

/// Instrument an async handler with RED metrics.
#[proc_macro_attribute]
pub fn red_instrument(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as RedArgs);
    let input = parse_macro_input!(item as ItemFn);
    if input.sig.asyncness.is_none() {
        return Error::new_spanned(input.sig.fn_token, "red_instrument requires async fn")
            .to_compile_error()
            .into();
    }

    let attrs = input.attrs;
    let vis = input.vis;
    let sig = input.sig;
    let block = input.block;
    let service = args.service;
    let route = args.route;

    quote! {
        #(#attrs)*
        #vis #sig {
            let __cyberos_red_started = ::std::time::Instant::now();
            let __cyberos_red_result = (async move #block).await;
            let __cyberos_red_duration_ms =
                __cyberos_red_started.elapsed().as_millis().min(u32::MAX as u128) as u32;
            ::cyberos_obs_sdk::red::record_request(
                #service,
                #route,
                "unknown",
                ::cyberos_obs_sdk::red::status_from_response_like(&__cyberos_red_result),
                __cyberos_red_duration_ms,
                &[],
            );
            __cyberos_red_result
        }
    }
    .into()
}
