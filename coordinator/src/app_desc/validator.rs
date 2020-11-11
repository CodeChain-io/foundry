// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::AppDesc;
use anyhow::bail;

impl AppDesc {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.tx_owners_are_valid()?;

        Ok(())
    }

    fn tx_owners_are_valid(&self) -> anyhow::Result<()> {
        let invalid_owners: Vec<(&str, &str)> = self
            .transactions
            .iter()
            .filter_map(|(tx_type, tx_owner)| {
                if self.modules.contains_key(tx_owner) {
                    None
                } else {
                    Some((tx_type as &str, &**tx_owner as &str))
                }
            })
            .collect();

        if invalid_owners.is_empty() {
            Ok(())
        } else {
            bail!(
                "No such owners for transactions: {}",
                invalid_owners
                    .into_iter()
                    .map(|(tx_type, tx_owner)| format!("{}: {}", tx_type, tx_owner))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
