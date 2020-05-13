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

use helper::*;
use proc_macro2::TokenStream as TokenStream2;

// TODO: Take an optional additional identifier to generate unique key for id registeration.
// This will allow user to have different remote traits with the same name.
// (But of course in different name spaces)
pub fn service(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
    let source_trait = match syn::parse2::<syn::ItemTrait>(input.clone()) {
        Ok(x) => x,
        Err(_) => return syn::Error::new_spanned(input, "You can use #[service] only on a trait").to_compile_error(),
    };

    if !args.is_empty() {
        return syn::Error::new_spanned(input, "#[service] doesn't take any arguments").to_compile_error()
    }

    let id = {
        let result = id::generate_id_registeration(&source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };
    let dispatch = {
        let result = dispatch::generate_dispatch(&source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };
    let import = {
        let result = call::generate_imported_struct(&source_trait);
        match result {
            Ok(x) => x,
            Err(x) => return x,
        }
    };

    quote! {
        #source_trait
        #id
        #dispatch
        #import
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream as TokenStream2;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use std::str::FromStr;

    pub fn service_string(source: &str) -> TokenStream2 {
        service(TokenStream2::new(), TokenStream2::from_str(source).unwrap())
    }

    #[test]
    fn example1() {
        let source = {
            let mut f = File::open(&Path::new("./src/example/ex1.rs")).unwrap();
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();
            service_string(&buffer)
        };
        let expected = {
            let mut f = File::open(&Path::new("./src/example/ex1_ex.rs")).unwrap();
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();
            TokenStream2::from_str(&buffer).unwrap()
        };
        assert_eq!(format!("{}", source), format!("{}", expected))
    }
}
