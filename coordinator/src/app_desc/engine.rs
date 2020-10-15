// Copyright 2018-2020 Kodebox, Inc.
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

use super::tendermint::TendermintParams;
use serde::Deserialize;

/// Engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "params")]
pub enum Engine {
    /// Null engine.
    Null,
    Solo,
    Tendermint(Box<TendermintParams>),
}

impl Default for Engine {
    fn default() -> Self {
        Engine::Tendermint(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;

    #[test]
    fn engine_deserialization() {
        let s = r#"
            type = "null"
        "#;

        let deserialized: Engine = toml::from_str(s).unwrap();
        assert_eq!(deserialized, Engine::Null);

        let s = r#"
            type = "solo"
        "#;

        let deserialized: Engine = toml::from_str(s).unwrap();
        match deserialized {
            Engine::Solo => {} // solo is unit tested in its own file.
            _ => panic!(),
        };

        let s = r#"
            type = "tendermint"
            params.timeoutPropose = 6
        "#;
        let deserialized: Engine = toml::from_str(s).unwrap();
        match deserialized {
            Engine::Tendermint(_) => {} // Tendermint is unit tested in its own file.
            _ => panic!(),
        };
    }
}
