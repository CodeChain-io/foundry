// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License

extern crate proc_macro;
extern crate syn;
extern crate proc_macro_crate;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use syn::Ident;
use proc_macro2::Span;
use proc_macro::TokenStream;
use quote::ToTokens;

use syn::parse_macro_input;
use syn::{DeriveInput, ItemImpl};

#[proc_macro_attribute]
pub fn handle_callable(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_copy = input.clone(); // parse_macro_input! take only single identifier
    let ast = parse_macro_input!(input_copy as syn::Item);
    let source_trait = match ast {
		syn::Item::Trait(item_trait) => item_trait,
		item => {
			syn::Error::new_spanned(
				item,
				"#[fml_callable] must be with a trait",
            ).to_compile_error();
            panic!();
		}
    };

    let name = source_trait.ident.clone();
    let name_con = Ident::new(&format!("{}_Con", name),
    source_trait.ident.span());

    let con_base = TokenStream::from(quote! {
        struct #name_con {
        }
    });
    let con_item = parse_macro_input!(con_base as syn::ItemStruct);

    let impl_base = TokenStream::from(quote! {
        impl #name for #name_con {
        }
    });
    let mut impl_item = parse_macro_input!(impl_base as syn::ItemImpl);

    let block_base = TokenStream::from(quote! {
        {
            panic!()
        }
    });
    let mut block_item = parse_macro_input!(block_base as syn::Block);

    let methods: Vec<&syn::TraitItemMethod> = source_trait
		.items
		.iter()
		.filter_map(|trait_item| {
			if let syn::TraitItem::Method(method) = trait_item {
				Some(method)
			} else {
				None // type, const, ... will be ignored
			}
		})
        .collect();

    for method in methods.iter() {
        impl_item.items.push(syn::ImplItem::Method(
            syn::ImplItemMethod {
                attrs: Vec::new(),
                vis: syn::Visibility::Inherited,
                defaultness: None,
                sig: method.sig.clone(),
                block: block_item.clone(),
            }
        ));
    }
    let mut result = input.clone();
    result.extend(TokenStream::from(con_item.to_token_stream()));
    result.extend(TokenStream::from(impl_item.to_token_stream()));

    result
}
