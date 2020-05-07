// Copyright 2018-2020 Kodebox, Inc.
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

use super::password_entry::PasswordEntry;
use std::io::Read;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PasswordFile(Vec<PasswordEntry>);

impl PasswordFile {
    pub fn load<R>(reader: R) -> Result<Self, serde_json::Error>
    where
        R: Read, {
        serde_json::from_reader(reader)
    }

    pub fn entries(&self) -> &[PasswordEntry] {
        self.0.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::super::password_entry::PasswordEntry;
    use super::PasswordFile;

    #[test]
    fn password_file() {
        let json = r#"
		[
            {
                "address": "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu",
                "password": "mypass1"
            },
            {
                "address": "tccqyaty0ad0jdy7865m06yl7fff5444dpdzhckreqxqjx440m7tkkegtwfee5",
                "password": "mypass2"
            }
		]"#;

        let expected = PasswordFile(vec![
            PasswordEntry {
                address: "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu".into(),
                password: "mypass1".into(),
            },
            PasswordEntry {
                address: "tccqyaty0ad0jdy7865m06yl7fff5444dpdzhckreqxqjx440m7tkkegtwfee5".into(),
                password: "mypass2".into(),
            },
        ]);

        let pf: PasswordFile = serde_json::from_str(json).unwrap();
        assert_eq!(pf, expected);
    }
}
