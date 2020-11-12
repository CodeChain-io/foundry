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

use ccore::{MinerOptions, TimeGapParams};
use cidr::IpCidr;
use cinformer::InformerConfig;
use ckey::PlatformAddress;
use cnetwork::{FilterEntry, NetworkConfig, SocketAddr};
use primitives::H256;
use std::fmt::Display;
use std::fs;
use std::net::Ipv4Addr;
use std::str::{self, FromStr};
use std::time::Duration;
use structconf::StructConf;

use crate::rpc::{RpcHttpConfig, RpcIpcConfig, RpcWsConfig};

#[derive(Debug, StructConf, Default)]
pub struct Config {
    #[conf(
        no_short,
        no_file,
        long = "config",
        help = "Specify the certain config file path that you want to use to configure CodeChain to your needs."
    )]
    pub config: Option<String>,

    // operating
    #[conf(
        no_short,
        long = "app-desc-path",
        help = "Specify the app descriptor path.",
        default = "\"./app-desc.toml\".to_string()"
    )]
    pub app_desc_path: String,

    #[conf(
        no_short,
        long = "link-desc-path",
        help = "Specify the link descriptor path.",
        default = "\"./link-desc.toml\".to_string()"
    )]
    pub link_desc_path: String,

    #[conf(
        short = "i",
        long = "instance-id",
        help = "Specify instance id for logging. Used when running multiple instances of CodeChain."
    )]
    pub instance_id: Option<usize>,

    #[conf(no_short, long = "base-path", help = "Specify the base directory path on which the "db" and "keys" directory will be created.")]
    pub base_path: Option<String>,

    #[conf(no_short, long = "db-path", help = "Specify the database directory path.")]
    pub db_path: Option<String>,

    #[conf(no_short, long = "keys-path", help = "Specify the path for JSON key files to be found")]
    pub keys_path: Option<String>,

    #[conf(no_short, long = "password-path", help = "Specify the password file path.")]
    pub password_path: Option<String>,

    // mining
    #[conf(
        no_short,
        long = "engine-signer",
        help = "Specify the address which should be used to sign consensus messages and issue blocks."
    )]
    pub engine_signer: Option<PlatformAddress>,

    #[conf(
        no_short,
        long = "mem-pool-size",
        help = "Maximum amount of transactions in the queue (waiting to be included in next block)."
    )]
    pub mem_pool_size: Option<usize>,

    #[conf(
        no_short,
        long = "mem-pool-mem-limit",
        help = "Maximum amount of memory that can be used by the mem pool. Setting this parameter to 0 disables limiting."
    )]
    pub mem_pool_mem_limit: Option<usize>,

    #[conf(
        no_short,
        long = "reseal-on-txs",
        help = "Specify which transactions should force the node to reseal a block."
    )]
    pub reseal_on_txs: Option<String>,

    #[conf(
        no_short,
        long = "reseal-min-period",
        help = "Specify the minimum time between reseals from incoming transactions. MS is time measured in milliseconds."
    )]
    pub reseal_min_period: Option<u64>,

    #[conf(
        no_short,
        long = "allowed-past-gap",
        help = "Specify the allowed gap in the past direction from the system time to the block generation time. MS is time measured in milliseconds."
    )]
    pub allowed_past_gap: Option<u64>,

    #[conf(
        no_short,
        long = "allowed-future-gap",
        help = "Specify the allowed gap in the future direction from the system time to the block generation time. MS is time measured in milliseconds."
    )]
    pub allowed_future_gap: Option<u64>,

    // network
    #[conf(negated_arg, no_short, long = "no-network", help = "Do not open network socket.")]
    pub network_enable: bool,

    #[conf(no_short, long = "interface", help = "Network interface to listen to.")]
    pub interface: Option<Ipv4Addr>,

    #[conf(no_short, long = "port", help = "Listen for connections on PORT.")]
    pub port: Option<u16>,

    #[conf(no_short, long = "bootstrap-addresses", help = "Bootstrap addresses to connect.")]
    pub bootstrap_addresses: Option<CommaSeparated<SocketAddr>>,

    #[conf(no_short, long = "min-peers", help = "Set the minimum number of connections the user would like.")]
    pub min_peers: Option<usize>,

    #[conf(no_short, long = "max-peers", help = "Set the maximum number of connections the user would like.")]
    pub max_peers: Option<usize>,

    #[conf(negated_arg, no_short, long = "no-sync", help = "Do not run block sync extension")]
    pub sync_enable: bool,

    #[conf(no_short, long = "snapshot-hash", help = "The block hash of the snapshot target block.")]
    pub snapshot_hash: Option<H256>,

    #[conf(no_short, long = "snapshot-number", help = "The block number of the snapshot target block.")]
    pub snapshot_number: Option<u64>,

    #[conf(negated_arg, no_short, long = "no-tx-relay", help = "Do not relay transactions.")]
    pub tx_relay_enable: bool,

    #[conf(negated_arg, no_short, long = "no-discovery", help = "Do not use discovery")]
    pub discovery_enable: bool,

    #[conf(
        no_short,
        long = "discovery",
        help = "Decide how to choose the addresses to be sent. Options are kademlia and unstructured. In a testing environment, an unstructured p2p network is desirable because it is more than sufficient when there are a few nodes(< 100)."
    )]
    pub discovery_type: Option<String>,

    #[conf(
        no_short,
        long = "discovery-refresh",
        help = "Refresh timeout of discovery. MS is time measured in milliseconds."
    )]
    pub discovery_refresh: Option<u32>,

    #[conf(no_short, long = "discovery-bucket-size", help = "Bucket size for discovery")]
    pub discovery_bucket_size: Option<u8>,

    #[conf(no_short, long = "blacklist-path", help = "Specify the path for the network blacklist file.")]
    pub blacklist_path: Option<String>,

    #[conf(no_short, long = "whitelist-path", help = "Specify the path for the network whitelist file.")]
    pub whitelist_path: Option<String>,

    // IPC
    #[conf(negated_arg, no_short, long = "no-ipc", help = "Do not run JSON-RPC over IPC service.")]
    pub ipc_enable: bool,

    #[conf(no_short, long = "ipc-path", help = "Specify custom path for JSON-RPC over IPC service.")]
    pub ipc_path: Option<String>,

    // RPC
    #[conf(negated_arg, no_short, long = "no-jsonrpc", help = "Do not run jsonrpc.")]
    pub jsonrpc_enable: bool,

    #[conf(no_short, long = "jsonrpc-interface", help = "Specify the interface address for rpc connections")]
    pub jsonrpc_interface: Option<Ipv4Addr>,

    #[conf(no_short, long = "jsonrpc-hosts", help = "Specify the allowed host addresses for rpc connections")]
    pub jsonrpc_hosts: Option<CommaSeparated<String>>,

    #[conf(no_short, long = "jsonrpc-cors", help = "Specify the cors domains for rpc connections")]
    pub jsonrpc_cors: Option<CommaSeparated<String>>,

    #[conf(no_short, long = "jsonrpc-port", help = "Listen for rpc connections on PORT.")]
    pub jsonrpc_port: Option<u16>,

    #[conf(no_short, long = "enable-devel-api", help = "Enable the RPC's devel APIs")]
    pub enable_devel_api: bool,

    // WS
    #[conf(negated_arg, no_short, long = "no-ws", help = "Do not run the WebSockets JSON-RPC server.")]
    pub ws_enable: bool,

    #[conf(
        no_short,
        long = "ws-interface",
        help = "Specify the interface address for the WebSockets JSON-RPC server."
    )]
    pub ws_interface: Option<Ipv4Addr>,

    #[conf(no_short, long = "ws-port", help = "Specify the port portion of the WebSockets JSON-RPC server.")]
    pub ws_port: Option<u16>,

    #[conf(
        no_short,
        long = "ws-max-connections",
        help = "Maximum number of allowed concurrent WebSockets JSON-RPC connections."
    )]
    pub ws_max_connections: Option<usize>,

    // GraphQL
    #[conf(no_short, long = "graphql-port", help = "Open GraphQL webserver on PORT.")]
    pub graphql_port: Option<u16>,

    #[conf(negated_arg, no_short, long = "no-informer", help = "Do not run the WebSockets JSON-RPC server.")]
    pub informer_enable: bool,

    // Informer
    #[conf(
        no_short,
        long = "informer-interface",
        help = "Specify the interface address for the WebSockets JSON-RPC server."
    )]
    pub informer_interface: Option<Ipv4Addr>,

    #[conf(no_short, long = "informer-port", help = "Specify the port portion of the WebSockets JSON-RPC server.")]
    pub informer_port: Option<u16>,

    #[conf(
        no_short,
        long = "informer-max-connections",
        help = "Maximum number of allowed concurrent WebSockets JSON-RPC connections."
    )]
    pub informer_max_connections: Option<usize>,

    // Snapshot
    #[conf(negated_arg, no_short, long = "no-snapshot", help = "Disable snapshots")]
    pub snapshot_enable: bool,

    #[conf(no_short, long = "snapshot-path", help = "Specify the snapshot directory path.")]
    pub snapshot_path: Option<String>,

    #[conf(no_short, no_long)]
    pub snapshot_expiration: Option<u64>,

    // Email
    #[conf(negated_arg, no_short, long = "no-email-alarm", help = "Do not use email alarm")]
    pub email_alarm_enable: bool,

    #[conf(no_short, long = "email-alarm-to", help = "Specify the email address to receive the alarm.")]
    pub email_alarm_to: Option<String>,

    #[conf(
        no_short,
        long = "email-alarm-sendgrid-key",
        help = "Specify the sendgrid key which is used to send alarms."
    )]
    pub email_alarm_sendgrid_key: Option<String>,
}

impl Config {
    pub fn miner_options(&self) -> Result<MinerOptions, String> {
        let (reseal_on_own_transaction, reseal_on_external_transaction) = match self.reseal_on_txs.as_deref() {
            Some("all") => (true, true),
            Some("own") => (true, false),
            Some("ext") => (false, true),
            Some("none") => (false, false),
            Some(x) => {
                return Err(format!(
                    "{} isn't a valid value for reseal-on-txs. Possible values are all, own, ext, none",
                    x
                ))
            }
            None => unreachable!(),
        };

        Ok(MinerOptions {
            mem_pool_size: self.mem_pool_size.unwrap(),
            mem_pool_memory_limit: match self.mem_pool_mem_limit.unwrap() {
                0 => None,
                mem_size => Some(mem_size * 1024 * 1024),
            },
            reseal_on_own_transaction,
            reseal_on_external_transaction,
            reseal_min_period: Duration::from_millis(self.reseal_min_period.unwrap()),
        })
    }

    pub fn rpc_http_config(&self) -> RpcHttpConfig {
        // FIXME: Add interface, cors and hosts options.
        RpcHttpConfig {
            interface: self.jsonrpc_interface.clone().unwrap(),
            port: self.jsonrpc_port.unwrap(),
            cors: self.jsonrpc_cors.as_ref().map(|cors| cors.inner.clone()),
            hosts: self.jsonrpc_hosts.as_ref().map(|cors| cors.inner.clone()),
        }
    }

    pub fn rpc_ipc_config(&self) -> RpcIpcConfig {
        RpcIpcConfig {
            socket_addr: self.ipc_path.clone().unwrap(),
        }
    }

    pub fn rpc_ws_config(&self) -> RpcWsConfig {
        // FIXME: Add hosts and origins options.
        RpcWsConfig {
            interface: self.ws_interface.clone().unwrap(),
            port: self.ws_port.unwrap(),
            max_connections: self.ws_max_connections.unwrap(),
        }
    }

    pub fn informer_config(&self) -> InformerConfig {
        InformerConfig {
            interface: self.informer_interface.clone().unwrap(),
            port: self.informer_port.unwrap(),
            max_connections: self.informer_max_connections.unwrap(),
        }
    }

    pub fn network_config(&self) -> Result<NetworkConfig, String> {
        fn make_ipaddr_list(list_path: Option<&String>, list_name: &str) -> Result<Vec<FilterEntry>, String> {
            if let Some(path) = list_path {
                fs::read_to_string(path)
                    .map_err(|e| format!("Cannot open the {}list file {:?}: {:?}", list_name, path, e))
                    .map(|rstr| {
                        rstr.lines()
                            .map(|s| {
                                const COMMENT_CHAR: char = '#';
                                if let Some(index) = s.find(COMMENT_CHAR) {
                                    let (ip_str, tag_str_with_sign) = s.split_at(index);
                                    (ip_str.trim(), (&tag_str_with_sign[1..]).trim().to_string())
                                } else {
                                    (s.trim(), String::new())
                                }
                            })
                            .filter(|(s, _)| !s.is_empty())
                            .map(|(addr, tag)| {
                                Ok(FilterEntry {
                                    cidr: IpCidr::from_str(addr)
                                        .map_err(|e| format!("Cannot parse IP address {}: {:?}", addr, e))?,
                                    tag,
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })?
            } else {
                Ok(Vec::new())
            }
        }

        let bootstrap_addresses =
            self.bootstrap_addresses.as_ref().map(|addresses| addresses.inner.clone()).unwrap_or_default();

        let whitelist = make_ipaddr_list(self.whitelist_path.as_ref(), "white")?;
        let blacklist = make_ipaddr_list(self.blacklist_path.as_ref(), "black")?;

        Ok(NetworkConfig {
            address: self.interface.clone().unwrap(),
            port: self.port.unwrap(),
            bootstrap_addresses,
            min_peers: self.min_peers.unwrap(),
            max_peers: self.max_peers.unwrap(),
            whitelist,
            blacklist,
        })
    }

    pub fn create_time_gaps(&self) -> TimeGapParams {
        let allowed_past_gap = Duration::from_millis(self.allowed_past_gap.unwrap_or(30000));
        let allowed_future_gap = Duration::from_millis(self.allowed_future_gap.unwrap_or(5000));

        TimeGapParams {
            allowed_past_gap,
            allowed_future_gap,
        }
    }
}

#[derive(Debug)]
pub struct CommaSeparated<T> {
    pub inner: Vec<T>,
}

impl<T: Display> Display for CommaSeparated<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string_vec = Vec::new();
        for t in &self.inner {
            string_vec.push(format!("{}", t));
        }
        write!(f, "{}", string_vec.join(","))
    }
}

impl<T: FromStr> FromStr for CommaSeparated<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "" {
            return Ok(CommaSeparated {
                inner: vec![],
            })
        }

        let tokens = s.split(',');
        let mut ret: Vec<T> = Vec::new();

        for token in tokens {
            let t: T = token.parse()?;
            ret.push(t);
        }

        Ok(CommaSeparated {
            inner: ret,
        })
    }
}

#[cfg(test)]
mod test {
    use super::CommaSeparated;

    #[test]
    fn comma_separated_empty() {
        let empty: CommaSeparated<String> = "".parse().unwrap();
        let expected: Vec<String> = vec![];
        assert_eq!(empty.inner, expected);
    }

    #[test]
    fn comma_separated_one() {
        let one: CommaSeparated<String> = "one".parse().unwrap();
        assert_eq!(one.inner, vec!["one".to_string()]);
    }

    #[test]
    fn comma_separated_two() {
        let onetwo: CommaSeparated<String> = "one,two".parse().unwrap();
        assert_eq!(onetwo.inner, vec!["one".to_string(), "two".to_string()]);
    }
}
