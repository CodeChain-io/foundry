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

// Common dependencis for services
extern crate codechain_fml as fml;
use fml::service_prelude::*;
pub struct TraitHolder<T: ?Sized>(std::marker::PhantomData<T>);

#[fml_macro::service]
pub trait HelloFactory: Service {
    fn create(&self, name: &String) -> Box<dyn HelloRobot>;
}

#[fml_macro::service]
pub trait HelloRobot: Service {
    fn hello(&self, flag: i32) -> String;
}
