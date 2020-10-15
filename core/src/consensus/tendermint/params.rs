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
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::types::View;
use super::Step;
use std::time::Duration;

/// `Tendermint` params.
pub struct TendermintParams {
    /// Timeout durations for different steps.
    pub timeouts: TimeoutParams,
}

impl From<coordinator::app_desc::TendermintParams> for TendermintParams {
    fn from(p: coordinator::app_desc::TendermintParams) -> Self {
        let dt = TimeoutParams::default();
        TendermintParams {
            timeouts: TimeoutParams {
                propose: p.timeout_propose.map_or(dt.propose, Duration::from_millis),
                propose_delta: p.timeout_propose_delta.map_or(dt.propose_delta, Duration::from_millis),
                prevote: p.timeout_prevote.map_or(dt.prevote, Duration::from_millis),
                prevote_delta: p.timeout_prevote_delta.map_or(dt.prevote_delta, Duration::from_millis),
                precommit: p.timeout_precommit.map_or(dt.precommit, Duration::from_millis),
                precommit_delta: p.timeout_precommit_delta.map_or(dt.precommit_delta, Duration::from_millis),
                commit: p.timeout_commit.map_or(dt.commit, Duration::from_millis),
            },
        }
    }
}

pub struct TimeGapParams {
    pub allowed_past_gap: Duration,
    pub allowed_future_gap: Duration,
}

/// Base timeout of each step in ms.
#[derive(Debug, Copy, Clone)]
pub struct TimeoutParams {
    pub propose: Duration,
    pub propose_delta: Duration,
    pub prevote: Duration,
    pub prevote_delta: Duration,
    pub precommit: Duration,
    pub precommit_delta: Duration,
    pub commit: Duration,
}

impl Default for TimeoutParams {
    fn default() -> Self {
        TimeoutParams {
            propose: Duration::from_millis(1000),
            propose_delta: Duration::from_millis(500),
            prevote: Duration::from_millis(1000),
            prevote_delta: Duration::from_millis(500),
            precommit: Duration::from_millis(1000),
            precommit_delta: Duration::from_millis(500),
            commit: Duration::from_millis(1000),
        }
    }
}

impl TimeoutParams {
    pub fn initial(&self) -> Duration {
        self.propose
    }

    pub fn timeout(&self, step: Step, view: View) -> Duration {
        let base = match step {
            Step::Propose => self.propose,
            Step::Prevote => self.prevote,
            Step::Precommit => self.precommit,
            Step::Commit => self.commit,
        };
        let delta = match step {
            Step::Propose => self.propose_delta,
            Step::Prevote => self.prevote_delta,
            Step::Precommit => self.precommit_delta,
            Step::Commit => Duration::default(),
        };
        base + delta * view as u32
    }
}
