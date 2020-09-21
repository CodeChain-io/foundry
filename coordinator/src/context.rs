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

mod chain_history_access;
mod mem_pool_access;
mod storage_access;
mod sub_storage_access;

pub use chain_history_access::ChainHistoryAccess;
pub use mem_pool_access::MemPoolAccess;
pub use storage_access::StorageAccess;
pub use sub_storage_access::SubStorageAccess;

/// A `Context` provides the interface against the system services such as moulde substorage access,
/// mempool access
pub trait Context: SubStorageAccess + MemPoolAccess {}
