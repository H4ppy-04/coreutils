#![allow(unused_mut)]
extern crate proc_macro;

#[macro_use]
mod quote;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn gen(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut in_fn = false;
    let mut output = TokenStream::new();
    let mut fn_name_changed = false;
    let mut body_brace_encountered = false;

    let is_mut = attr.to_string().contains("mut");
    let is_bin = attr.to_string().contains("bin");

    let cfg_attr = match is_bin {
        false => quote! { #[start] },
        true => quote! { #[cfg_attr(feature = "bin", start)] },
    };

    output.extend(cfg_attr);

    for token in item {
        match token {
            TokenTree::Ident(ref ident) if ident.to_string() == "fn" => {
                in_fn = true;
                output.extend(quote!(pub fn));
            }
            TokenTree::Ident(ref _ident) if in_fn && !fn_name_changed => {
                fn_name_changed = true;
                output.extend(quote!(_start));
            }
            TokenTree::Group(ref group) if in_fn && group.delimiter() == Delimiter::Parenthesis => {
                output.extend(quote!((argc: isize, argv: *const *const u8)));
            }
            TokenTree::Punct(ref punct) if punct.as_char() == '!' => {
                output.extend(quote!(isize));
            }
            TokenTree::Group(ref group) if in_fn && group.delimiter() == Delimiter::Brace && !body_brace_encountered => {
                body_brace_encountered = true;
                let mut body = TokenStream::new();

                body.extend(quote! {
                    let (program, args) = prelude::parse_args(argc, argv);
                    let ?(is_mut: mut) args = args?(is_bin: .into_iter());
                    #(group.stream());
                    return 0;
                });

                export!(output, { body });
            }
            _ => output.extend(std::iter::once(token)),
        }
    }

    return output;
}
