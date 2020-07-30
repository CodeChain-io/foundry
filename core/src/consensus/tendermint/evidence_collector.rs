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
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::vote_collector::DoubleVote;
use std::mem::take;

pub type Evidence = DoubleVote; // This may be generalized in the future

#[derive(Default)]
pub struct EvidenceCollector {
    evidences: Vec<Evidence>,
}

impl EvidenceCollector {
    pub fn insert_double_vote(&mut self, double_vote: DoubleVote) {
        self.evidences.push(double_vote);
    }

    pub fn fetch_evidences(&mut self) -> Vec<Evidence> {
        take(&mut self.evidences)
    }

    pub fn remove_published_evidences(&mut self, published: Vec<Evidence>) {
        self.evidences.retain(|e| !published.contains(e));
    }
}
