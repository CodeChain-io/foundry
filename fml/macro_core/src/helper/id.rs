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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::Ident;

pub fn id_method_ident(the_trait: &syn::ItemTrait, method: &syn::TraitItemMethod) -> Ident {
    quote::format_ident!("ID_METHOD_{}_{}", the_trait.ident, method.sig.ident)
}

pub fn id_trait_ident(the_trait: &syn::ItemTrait) -> Ident {
    quote::format_ident!("ID_TRAIT_{}", the_trait.ident)
}

pub fn id_trait_ident_from_ident(the_trait_ident: &syn::Ident) -> Ident {
    quote::format_ident!("ID_TRAIT_{}", the_trait_ident)
}

fn lit_index(index: usize) -> syn::Lit {
    // We put a distinctive offset for easy debug.
    syn::Lit::Int(syn::LitInt::new(&format!("{}", index + 7), Span::call_site()))
}

fn id_method_entry_ident(the_trait: &syn::ItemTrait, method: &syn::TraitItemMethod) -> Ident {
    quote::format_ident!("ID_METHOD_ENTRY_{}_{}", the_trait.ident, method.sig.ident)
}

fn id_method_setter_ident(the_trait: &syn::ItemTrait, method: &syn::TraitItemMethod) -> Ident {
    quote::format_ident!("id_method_setter_{}_{}", the_trait.ident, method.sig.ident)
}

fn id_trait_entry_ident(the_trait: &syn::ItemTrait) -> Ident {
    quote::format_ident!("ID_TRAIT_ENTRY_{}", the_trait.ident)
}

fn id_trait_setter_ident(the_trait: &syn::ItemTrait) -> Ident {
    quote::format_ident!("id_trait_setter_{}", the_trait.ident)
}

pub fn generate_id_registeration(the_trait: &syn::ItemTrait) -> Result<TokenStream2, TokenStream2> {
    let mut result = TokenStream2::new();

    let lit_trait_name = syn::LitStr::new(&format!("{}", the_trait.ident), Span::call_site());
    // registeration for trait itslef
    result.extend({
        let id_ident = id_trait_ident(&the_trait);
        let id_entry_ident = id_trait_entry_ident(&the_trait);
        let id_setter_ident = id_trait_setter_ident(&the_trait);
        let id_entry = quote! {
            #[allow(non_upper_case_globals)]
            static #id_ident: TraitIdAtomic = TraitIdAtomic::new(0);
            #[allow(non_upper_case_globals)]
            #[distributed_slice(TID_REG)]
            static #id_entry_ident: (&'static str, fn(id: TraitId)) =
            (#lit_trait_name, #id_setter_ident);
            #[allow(non_snake_case)]
            fn #id_setter_ident(id: TraitId) {
                #id_ident.store(id, ID_ORDERING);
            }
        };
        id_entry
    });

    // registeration for methods in the trait
    let mut method_id_table = TokenStream2::new();
    for (i, item) in the_trait.items.iter().enumerate() {
        let method = match item {
            syn::TraitItem::Method(x) => x,
            non_method => {
                return Err(
                    syn::Error::new_spanned(non_method, "Service trait must have only methods").to_compile_error()
                )
            }
        };
        let lit_index = lit_index(i);
        let lit_method_name = syn::LitStr::new(&format!("{}", method.sig.ident), Span::call_site());

        let id_ident = id_method_ident(&the_trait, method);
        let id_entry_ident = id_method_entry_ident(&the_trait, method);
        let id_setter_ident = id_method_setter_ident(&the_trait, method);
        let id_entry = quote! {
            #[allow(non_upper_case_globals)]
            static #id_ident: MethodIdAtomic = MethodIdAtomic::new(#lit_index);
            #[distributed_slice(MID_REG)]
            #[allow(non_upper_case_globals)]
            static #id_entry_ident: (&'static str, &'static str, fn(id: MethodId)) =
            (#lit_trait_name, #lit_method_name, #id_setter_ident);
            #[allow(non_snake_case)]
            fn #id_setter_ident(id: MethodId) {
                #id_ident.store(id, ID_ORDERING);
            }
        };
        method_id_table.extend(id_entry);
    }
    result.extend(method_id_table);
    Ok(result)
}
