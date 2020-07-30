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

use std::time::{Duration, Instant};
use tokio::time;

pub struct RateLimiter {
    maximum_ticket: u64,
    current_ticket: u64,
    last_reset: Instant,
}

impl RateLimiter {
    pub fn new(ticket: u64) -> Self {
        Self {
            maximum_ticket: ticket,
            current_ticket: 0,
            last_reset: Instant::now(),
        }
    }

    pub async fn acquire_ticket(&mut self) {
        self.current_ticket += 1;
        let elapsed = self.last_reset.elapsed();
        if elapsed < Duration::from_secs(1) && self.current_ticket == self.maximum_ticket {
            time::delay_for(Duration::from_secs(1) - elapsed).await;
            self.current_ticket = 0;
            self.last_reset = Instant::now();
        } else if elapsed > Duration::from_secs(1) {
            self.last_reset = Instant::now();
            self.current_ticket = 0;
        }
    }
}
