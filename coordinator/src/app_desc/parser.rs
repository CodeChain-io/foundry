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

#[cfg(test)]
mod test {
    use crate::app_desc::AppDesc;
    use std::error::Error;
    use unindent::unindent;

    #[test]
    fn load_essentials() -> Result<(), Box<dyn Error>> {
        let source = unindent(
            r#"
            modules:
                awesome-module:
                    hash: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
                    init-config:
                        test: 1
                        test:
                            key1: 1
                            key2: sdfsdaf
            host:
                imports:
                    a: a.a
                    =namespace:
                        b.b: asdfsdaf-asdf
            transactions:
                great-tx:
                    owner: awesome-module
                    services:
                        - tx-executor
            params:
                num-threads:
                    int: 10
        "#,
        );
        let desc: AppDesc = serde_yaml::from_str(&source)?;
        eprintln!("{:?}", desc);
        Ok(())
    }
}
