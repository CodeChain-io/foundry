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

use super::commitment_23::CommitmentRootCounter;
use super::*;
use crate::consensus::light_client::verify_header;
use types::{ClientState, ConsensusState, Header};

pub fn check_validity_and_update_state(
    client_state: &ClientState,
    header: &Header,
) -> Result<(ClientState, ConsensusState), String> {
    if !verify_header(&client_state.raw, &header.update_header) {
        return Err("Invalid header has been given".to_string())
    }

    let new_client_state = ClientState {
        raw: header.update_header.make_client_state(),
    };
    let consensus_state = ConsensusState {
        validator_set_hash: client_state.raw.next_validator_set_hash,
        state_root: CommitmentRootCounter {
            raw: *header.update_header.header_raw.state_root(),
        },
    };

    Ok((new_client_state, consensus_state))
}
