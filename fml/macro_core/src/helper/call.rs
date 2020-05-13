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

use super::path_of_single_ident;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;

pub fn generate_imported_struct(the_trait: &syn::ItemTrait) -> Result<TokenStream2, TokenStream2> {
    let trait_ident = the_trait.ident.clone();
    let struct_ident = quote::format_ident!("{}Imported", trait_ident);
    let mut imported_struct = quote! {
        #[derive(Debug)]
        pub struct #struct_ident {
            handle: HandleInstance
        }
    };
    let mut imported_struct_impl = syn::parse2::<syn::ItemImpl>(quote! {
        impl #trait_ident for #struct_ident {
        }
    })
    .unwrap();

    for item in the_trait.items.iter() {
        let method = match item {
            syn::TraitItem::Method(x) => x,
            non_method => {
                return Err(
                    syn::Error::new_spanned(non_method, "Service trait must have only methods").to_compile_error()
                )
            }
        };

        let id_ident = super::id::id_method_ident(the_trait, method);

        let mut the_method = syn::parse_str::<syn::ImplItemMethod>("fn dummy() -> () {}").unwrap();
        the_method.sig = method.sig.clone();
        let mut arguments_in_tuple = syn::ExprTuple {
            attrs: Vec::new(),
            paren_token: syn::token::Paren(Span::call_site()),
            elems: syn::punctuated::Punctuated::new(),
        };
        for arg in &method.sig.inputs {
            match arg {
                syn::FnArg::Receiver(_) => continue, // &self
                syn::FnArg::Typed(pattern) => {
                    if let syn::Pat::Ident(the_arg) = &*pattern.pat {
                        arguments_in_tuple.elems.push(syn::Expr::Path(syn::ExprPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: path_of_single_ident(the_arg.ident.clone()),
                        }));
                    } else {
                        return Err(syn::Error::new_spanned(arg, "You must not use a pattern for the argument")
                            .to_compile_error())
                    }
                }
            }
        }

        let the_call = if let syn::ReturnType::Type(_, x) = &the_method.sig.output {
            // Special case of returning Box<dyn Service>
            if let Some(mut ident) = super::types::is_handle(&*x) {
                let x = ident.segments.last_mut().unwrap();
                x.ident = quote::format_ident!("{}{}", x.ident, "Imported");
                quote! {
                    let handle = #ident{handle: call(&self.handle, #id_ident.load(ID_ORDERING), &#arguments_in_tuple)};
                    Box::new(handle)
                }
            // Or plain data
            } else {
                quote! {
                    call(&self.handle, #id_ident.load(ID_ORDERING), &#arguments_in_tuple)
                }
            }
        } else {
            quote! {
                call(&self.handle, #id_ident.load(ID_ORDERING), &#arguments_in_tuple)
            }
        };
        the_method.block.stmts.push(syn::Stmt::Expr(syn::Expr::Verbatim(the_call)));
        imported_struct_impl.items.push(syn::ImplItem::Method(the_method));
    }
    imported_struct.extend(imported_struct_impl.to_token_stream());
    imported_struct.extend(quote! {
        impl Service for #struct_ident {
            fn get_handle(&self) -> &HandleInstance {
                &self.handle
            }
            fn get_handle_mut(&mut self) -> &mut HandleInstance {
                &mut self.handle
            }
        }
        impl Drop for #struct_ident {
            fn drop(&mut self) {
                delete(&self.handle)
            }
        }
        impl association::Import for TraitHolder<dyn #trait_ident> {
            type T = dyn #trait_ident;
            fn import(handle: HandleInstance) -> Box<dyn #trait_ident>  {
                Box::new(#struct_ident  {
                    handle,
                })
            }
        }
        impl ServiceDispatcher for #struct_ident  {
            fn dispatch(&self, _method: MethodId, _arguments: &[u8], _return_buffer: std::io::Cursor<&mut Vec<u8>>) {panic!()}
        }
    });
    Ok(imported_struct.to_token_stream())
}
