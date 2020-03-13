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

use crate::cmodule::link::{Linkable, Port};
use crate::cmodule::sandbox::{Result, Sandbox, Sandboxer};
use crate::sandbox::BaseSandbox;
use std::path::Path;
use std::sync::Arc;

pub struct BaseSandboxer {}

impl Sandboxer for BaseSandboxer {
    fn id(&self) -> &'static str {
        "Base"
    }

    fn supported_module_types(&self) -> &'static [&'static str] {
        static L: [&'static str; 1] = ["Base"];
        &L
    }

    fn load(&self, hash: &dyn AsRef<Path>) -> Result<Arc<dyn Sandbox>> {
        let path = "Hi.exe".to_owned();
        let id = "John".to_owned();

        Ok(Arc::new(BaseSandbox::new(path, id, self.id().to_string())))
    }
}
