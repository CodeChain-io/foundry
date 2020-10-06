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

use awc::Client;
use std::collections::HashMap;
use std::process::{Child, Command};

pub fn run_node(port: u16) -> Child {
    let path = std::fs::canonicalize("../target/debug/foundry").unwrap();
    let mut command = Command::new(path);
    command.arg("--graphql-port").arg(format!("{}", port)).current_dir("../").spawn().unwrap()
}

pub async fn request_query(port: u16, module: &str, query: &str, variables: &str) -> String {
    let query: HashMap<String, &str> =
        vec![("query".to_owned(), query), ("variables".to_owned(), variables)].into_iter().collect();

    let client = Client::new();
    let request = client.get(&format!("http://localhost:{}/{}/graphql", port, module)).query(&query).unwrap();
    let response_bytes = request.send().await.unwrap().body().await.unwrap();
    let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
    response.to_owned()
}
