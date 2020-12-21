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
use chrono::Local;
use ckey::{Ed25519Private as Private, Ed25519Public as Public, Signature};
use coordinator::Transaction;
use serde_json::Value;
use std::{collections::HashMap, sync::atomic::Ordering};
use std::{
    fs::File,
    process::{Child, Command},
};
use std::{path::Path, sync::atomic::AtomicUsize};

static ID_COUNT: AtomicUsize = AtomicUsize::new(1);

pub struct FoundryNode {
    child: Child,
}

impl Drop for FoundryNode {
    fn drop(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }
}

pub struct RunNodeArgs {
    pub foundry_path: String,
    pub rust_log: String,
    pub app_desc_path: String,
    pub link_desc_path: String,
    pub config_path: String,
}

pub struct FoundryArgs {
    pub graphql_port: u16,
    pub mem_pool_size: u64,
    pub engine_signer: String,
    pub port: u16,
    pub bootstrap_addresses: Vec<String>,
    pub password_path: String,
}

pub fn run_node(
    RunNodeArgs {
        foundry_path,
        rust_log,
        app_desc_path,
        link_desc_path,
        config_path,
    }: RunNodeArgs,
    foundry_args: FoundryArgs,
) -> FoundryNode {
    let path = std::fs::canonicalize(foundry_path).expect("Can't find foundry binary");
    let mut command = Command::new(path);
    let id = ID_COUNT.fetch_add(1, Ordering::SeqCst);

    if let Ok(x) = std::env::var("USE_LOG") {
        if x == "1" {
            let LogFiles {
                stdout,
                stderr,
            } = create_log_files(id);
            command.stdout(stdout).stderr(stderr);
        }
    }

    command
        .env("RUST_LOG", rust_log)
        .arg("--app-desc-path")
        .arg(app_desc_path)
        .arg("--link-desc-path")
        .arg(link_desc_path)
        .arg("--config")
        .arg(config_path)
        .arg("-i")
        .arg(format!("{}", id))
        .arg("--db-path")
        .arg(format!("/tmp/foundry_db{}", id));

    command
        .arg("--graphql-port")
        .arg(format!("{}", foundry_args.graphql_port))
        .arg("--mem-pool-size")
        .arg(format!("{}", foundry_args.mem_pool_size))
        .arg("--engine-signer")
        .arg(foundry_args.engine_signer)
        .arg("--port")
        .arg(format!("{}", foundry_args.port))
        .arg("--bootstrap-addresses")
        .arg(foundry_args.bootstrap_addresses.join(","))
        .arg("--password-path")
        .arg(foundry_args.password_path);

    FoundryNode {
        child: command.spawn().unwrap(),
    }
}

struct LogFiles {
    stdout: File,
    stderr: File,
}

fn create_log_files(id: usize) -> LogFiles {
    if !Path::new("logs").is_dir() {
        std::fs::create_dir("logs").expect("create log directory");
    }

    let now = Local::now().format("%y%m%d_%H%M%S");
    let stderr_log_file = {
        let name = format!("logs/{now}.{id}.stderr.log", now = now, id = id);
        File::create(name).expect("Create log file")
    };
    let stdout_log_file = {
        let name = format!("logs/{now}.{id}.stdout.log", now = now, id = id);
        File::create(name).expect("Create log file")
    };

    LogFiles {
        stdout: stdout_log_file,
        stderr: stderr_log_file,
    }
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

pub async fn get_latest_block(port: u16) -> u64 {
    let query_result = request_query(port, "engine", "{block{header{number}}}", "{}").await;
    let value: Value = serde_json::from_str(&query_result).unwrap();
    value["data"]["block"]["header"]["number"].as_u64().unwrap()
}

pub async fn get_event(port: u16, tx_hash: primitives::H256) -> Vec<Vec<u8>> {
    let query = "query Test($txHash: String!) {
        event(txHash: $txHash)
    }";
    let mut variables = Value::Object(Default::default());
    variables["txHash"] = Value::String(hex::encode(tx_hash.as_ref()));

    let query_result = request_query(port, "engine", query, &variables.to_string()).await;
    let value: Value = serde_json::from_str(&query_result).unwrap();
    let list = value["data"]["event"].as_array().unwrap();
    list.iter().map(|event| hex::decode(event.as_str().unwrap()).unwrap()).collect()
}

/// Returns the number of block including it, if there is.
pub async fn get_tx(port: u16, tx_hash: primitives::H256) -> Option<u64> {
    let query = "query Test($txHash: String!) {
        transaction(txHash: $txHash)
    }";
    let mut variables = Value::Object(Default::default());
    variables["txHash"] = Value::String(hex::encode(tx_hash.as_ref()));

    let query_result = request_query(port, "engine", query, &variables.to_string()).await;
    let value: Value = serde_json::from_str(&query_result).unwrap();
    serde_json::from_value(value["data"]["transaction"].clone()).unwrap()
}

/// This is a copy from `foundry-timestamp`.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub action: Vec<u8>,
}

pub fn sign_tx(public: &Public, private: &Private, tx_type: String, action: Vec<u8>) -> Transaction {
    let tx = SignedTransaction {
        signature: ckey::sign(&action, private),
        signer_public: *public,
        action,
    };
    Transaction::new(tx_type, serde_cbor::to_vec(&tx).unwrap())
}

pub async fn send_tx(port: u16, tx_type: &str, body: &[u8]) -> Result<(), ()> {
    let query = "mutation Test($txType: String!, $body: String!) {
        sendTransaction(txType: $txType, body: $body)
    }";
    let mut variables = Value::Object(Default::default());
    variables["txType"] = Value::String(tx_type.to_owned());
    variables["body"] = Value::String(hex::encode(body));

    let query_result = request_query(port, "engine", query, &variables.to_string()).await;
    let value: Value = serde_json::from_str(&query_result).unwrap();

    if value["data"]["sendTransaction"].as_str().unwrap() == "Done" {
        Ok(())
    } else {
        Err(())
    }
}

pub async fn create_tx_hello(port: u16, public: &Public, private: &Private, sequence: u64) -> Transaction {
    let query = "query Test($seq: Int!) {
        txHello(seq: $seq)
    }";
    let mut variables = Value::Object(Default::default());
    variables["seq"] = Value::Number(sequence.into());

    let query_result = request_query(port, "module-account", query, &variables.to_string()).await;
    let value: Value = serde_json::from_str(&query_result).unwrap();
    let tx = hex::decode(value["data"]["txHello"].as_str().unwrap()).unwrap();

    sign_tx(public, private, "hello".to_owned(), tx)
}
