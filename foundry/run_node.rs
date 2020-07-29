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

use crate::config::{self, load_config};
use crate::constants::{DEFAULT_DB_PATH, DEFAULT_KEYS_PATH};
use crate::dummy_network_service::DummyNetworkService;
use crate::json::PasswordFile;
use crate::rpc::{rpc_http_start, rpc_ipc_start, rpc_ws_start, setup_rpc_server};
use crate::rpc_apis::ApiDependencies;
use ccore::{snapshot_notify, EngineClient};
use ccore::{
    AccountProvider, AccountProviderError, ChainNotify, ClientConfig, ClientService, EngineInfo, EngineType, Miner,
    MinerService, PeerDb, Scheme, NUM_COLUMNS,
};
use cdiscovery::{Config, Discovery};
use cinformer::{handler::Handler, InformerEventSender, InformerService, MetaIoHandler, PubSubHandler, Session};
use ckey::{Ed25519Public as Public, NetworkId, PlatformAddress};
use ckeystore::accounts_dir::RootDiskDirectory;
use ckeystore::KeyStore;
use clap::ArgMatches;
use clogger::{EmailAlarm, LoggerConfig};
use cnetwork::{Filters, ManagingPeerdb, NetworkConfig, NetworkControl, NetworkService, RoutingTable, SocketAddr};
use crossbeam::unbounded;
use crossbeam_channel as crossbeam;
use csync::snapshot::Service as SnapshotService;
use csync::{BlockSyncExtension, BlockSyncSender, TransactionSyncExtension};
use ctimer::TimerLoop;
use ctrlc::CtrlC;
use fdlimit::raise_fd_limit;
use kvdb::KeyValueDB;
use kvdb_rocksdb::{Database, DatabaseConfig};
use parking_lot::{Condvar, Mutex};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

fn network_start(
    network_id: NetworkId,
    timer_loop: TimerLoop,
    cfg: &NetworkConfig,
    routing_table: Arc<RoutingTable>,
    peer_db: Box<dyn ManagingPeerdb>,
    sender: InformerEventSender,
) -> Result<Arc<NetworkService>, String> {
    let addr = cfg.address.parse().map_err(|_| format!("Invalid NETWORK listen host given: {}", cfg.address))?;
    let sockaddress = SocketAddr::new(addr, cfg.port);
    let filters = Filters::new(cfg.whitelist.clone(), cfg.blacklist.clone());
    let service = NetworkService::start(
        network_id,
        timer_loop,
        sockaddress,
        cfg.bootstrap_addresses.clone(),
        cfg.min_peers,
        cfg.max_peers,
        filters,
        routing_table,
        peer_db,
        sender,
    )
    .map_err(|e| format!("Network service error: {:?}", e))?;

    Ok(service)
}

fn discovery_start(
    service: &NetworkService,
    cfg: &config::Network,
    routing_table: Arc<RoutingTable>,
) -> Result<(), String> {
    let config = Config {
        bucket_size: cfg.discovery_bucket_size.unwrap(),
        t_refresh: cfg.discovery_refresh.unwrap(),
    };
    let use_kademlia = match cfg.discovery_type.as_deref() {
        Some("unstructured") => false,
        Some("kademlia") => true,
        Some(discovery_type) => return Err(format!("Unknown discovery {}", discovery_type)),
        None => return Ok(()),
    };
    service.register_extension(move |api| Discovery::new(routing_table, config, api, use_kademlia));
    Ok(())
}

fn client_start(
    client_config: &ClientConfig,
    timer_loop: &TimerLoop,
    db: Arc<dyn KeyValueDB>,
    scheme: &Scheme,
    miner: Arc<Miner>,
) -> Result<ClientService, String> {
    cinfo!(CLIENT, "Starting client");
    let reseal_timer = timer_loop.new_timer_with_name("Client reseal timer");
    let service = ClientService::start(client_config, &scheme, db, miner, reseal_timer.clone())
        .map_err(|e| format!("Client service error: {}", e))?;
    reseal_timer.set_handler(Arc::downgrade(&service.client()));

    Ok(service)
}

fn new_miner(
    config: &config::Config,
    scheme: &Scheme,
    ap: Arc<AccountProvider>,
    db: Arc<dyn KeyValueDB>,
) -> Result<Arc<Miner>, String> {
    let miner = Miner::new(config.miner_options()?, scheme, ap, db);

    match miner.engine_type() {
        EngineType::PBFT => match &config.mining.engine_signer {
            Some(ref engine_signer) => match miner.set_author((*engine_signer).into_pubkey()) {
                Err(AccountProviderError::NotUnlocked) => {
                    return Err(
                        format!("The account {} is not unlocked. The key file should exist in the keys_path directory, and the account's password should exist in the password_path file.", engine_signer)
                    )
                }
                Err(e) => return Err(format!("{}", e)),
                _ => (),
            },
            None if config.mining.author.is_some() => {
                return Err("PBFT type engine needs not an author but an engine signer for mining. Specify the engine signer using --engine-signer option."
                    .to_string())
            }
            None => (),
        },
        EngineType::Solo => miner
            .set_author(config.mining.author.map_or(Public::default(), PlatformAddress::into_pubkey))
            .expect("set_author never fails when Solo is used"),
    }

    Ok(miner)
}

fn wait_for_exit() {
    let exit = Arc::new((Mutex::new(()), Condvar::new()));

    // Handle possible exits
    let e = exit.clone();
    CtrlC::set_handler(move || {
        e.1.notify_all();
    });

    // Wait for signal
    let mut l = exit.0.lock();
    exit.1.wait(&mut l);
}

fn prepare_account_provider(keys_path: &str) -> Result<Arc<AccountProvider>, String> {
    let keystore_dir = RootDiskDirectory::create(keys_path).map_err(|_| "Cannot read key path directory")?;
    let keystore = KeyStore::open(Box::new(keystore_dir)).map_err(|_| "Cannot open key store")?;
    Ok(AccountProvider::new(keystore))
}

fn load_password_file(path: &Option<String>) -> Result<PasswordFile, String> {
    let pf = match path.as_ref() {
        Some(path) => {
            let file = fs::File::open(path).map_err(|e| format!("Could not read password file at {}: {}", path, e))?;
            PasswordFile::load(file).map_err(|e| format!("Invalid password file {}: {}", path, e))?
        }
        None => PasswordFile::default(),
    };
    Ok(pf)
}

fn unlock_accounts(ap: &AccountProvider, pf: &PasswordFile) -> Result<(), String> {
    for entry in pf.entries() {
        let pubkey = entry.address.into_pubkey();
        let has_account = ap
            .has_account(&pubkey)
            .map_err(|e| format!("Unexpected error while querying account {:?}: {}", pubkey, e))?;
        if has_account {
            ap.unlock_account_permanently(pubkey, entry.password.clone())
                .map_err(|e| format!("Failed to unlock account {:?}: {}", pubkey, e))?;
        }
    }
    Ok(())
}

pub fn open_db(cfg: &config::Operating, client_config: &ClientConfig) -> Result<Arc<dyn KeyValueDB>, String> {
    let base_path = cfg.base_path.as_ref().unwrap().clone();
    let db_path = cfg.db_path.as_ref().map(String::clone).unwrap_or_else(|| base_path + "/" + DEFAULT_DB_PATH);
    let client_path = Path::new(&db_path);
    let mut db_config = DatabaseConfig::with_columns(NUM_COLUMNS);

    db_config.memory_budget = client_config.db_cache_size;
    db_config.compaction = client_config.db_compaction.compaction_profile(client_path);

    let db = Arc::new(
        Database::open(&db_config, &client_path.to_str().expect("DB path could not be converted to string."))
            .map_err(|_e| "Low level database error. Some issue with disk?".to_string())?,
    );

    Ok(db)
}

pub fn run_node(matches: &ArgMatches<'_>) -> Result<(), String> {
    // increase max number of open files
    raise_fd_limit();

    let timer_loop = TimerLoop::new(2);

    let config = load_config(matches)?;

    let time_gap_params = config.mining.create_time_gaps();
    let scheme = match &config.operating.chain {
        Some(chain) => chain.scheme()?,
        None => return Err("chain is not specified".to_string()),
    };
    scheme.engine.register_time_gap_config_to_worker(time_gap_params);

    let instance_id = config.operating.instance_id.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time should be later than unix epoch")
            .subsec_nanos() as usize,
    );
    let email_alarm = if !config.email_alarm.disable.unwrap() {
        let to = config.email_alarm.to.clone().ok_or_else(|| "email-alarm-to is not specified".to_string())?;
        let sendgrid_key = config
            .email_alarm
            .sendgrid_key
            .clone()
            .ok_or_else(|| "email-alarm-sendgrid-key is not specified".to_string())?;
        Some(EmailAlarm::new(to, sendgrid_key, scheme.genesis_params().network_id().to_string()))
    } else {
        None
    };
    clogger::init(&LoggerConfig::new(instance_id), email_alarm.clone())
        .expect("Logger must be successfully initialized");
    if let Some(email_alarm) = email_alarm {
        panic_hook::set_with_email_alarm(email_alarm);
    }

    let pf = load_password_file(&config.operating.password_path)?;
    let base_path = config.operating.base_path.as_ref().unwrap().clone();
    let keys_path =
        config.operating.keys_path.as_ref().map(String::clone).unwrap_or_else(|| base_path + "/" + DEFAULT_KEYS_PATH);
    let ap = prepare_account_provider(&keys_path)?;
    unlock_accounts(&*ap, &pf)?;

    let client_config: ClientConfig = Default::default();
    let db = open_db(&config.operating, &client_config)?;

    let miner = new_miner(&config, &scheme, ap.clone(), Arc::clone(&db))?;
    let client = client_start(&client_config, &timer_loop, db, &scheme, miner.clone())?;
    miner.recover_from_db(client.client().as_ref());

    // FIXME: unbound would cause memory leak.
    // FIXME: The full queue should be handled.
    // This will be fixed soon.
    let (informer_sub_sender, informer_sub_receiver) = unbounded();
    let informer_event_sender = {
        if !config.informer.disable.unwrap() {
            let (service, event_sender) = InformerService::new(informer_sub_receiver, client.client());
            service.run_service();
            event_sender
        } else {
            InformerEventSender::null_notifier()
        }
    };

    let mut _maybe_sync = None;
    let mut maybe_sync_sender = None;

    scheme.engine.register_chain_notify(client.client().as_ref());

    let network_service: Arc<dyn NetworkControl> = {
        if !config.network.disable.unwrap() {
            let network_config = config.network_config()?;
            // XXX: What should we do if the network id has been changed.
            let c = client.client();
            let network_id = c.network_id();
            let routing_table = RoutingTable::new();
            let peer_db = PeerDb::new(c.get_kvdb());
            let service = network_start(
                network_id,
                timer_loop,
                &network_config,
                Arc::clone(&routing_table),
                peer_db,
                informer_event_sender,
            )?;

            if config.network.discovery.unwrap() {
                discovery_start(&service, &config.network, routing_table)?;
            } else {
                cwarn!(DISCOVERY, "Node runs without discovery extension");
            }

            if config.network.sync.unwrap() {
                let sync_sender = {
                    let client = client.client();
                    let snapshot_target = match (config.network.snapshot_hash, config.network.snapshot_number) {
                        (Some(hash), Some(num)) => Some((hash, num)),
                        _ => None,
                    };
                    let snapshot_dir = config.snapshot.path.clone();
                    service.register_extension(move |api| {
                        BlockSyncExtension::new(client, api, snapshot_target, snapshot_dir)
                    })
                };
                let sync = Arc::new(BlockSyncSender::from(sync_sender.clone()));
                client.client().add_notify(Arc::downgrade(&sync) as Weak<dyn ChainNotify>);
                _maybe_sync = Some(sync); // Hold sync to ensure it not to be destroyed.
                maybe_sync_sender = Some(sync_sender);
            }
            if config.network.transaction_relay.unwrap() {
                let client = client.client();
                service.register_extension(move |api| TransactionSyncExtension::new(client, api));
            }

            scheme.engine.register_network_extension_to_service(&service);

            service
        } else {
            Arc::new(DummyNetworkService::new())
        }
    };

    let informer_server = {
        if !config.informer.disable.unwrap() {
            let io: PubSubHandler<Arc<Session>> = PubSubHandler::new(MetaIoHandler::default());
            let mut informer_handler = Handler::new(io);
            informer_handler.event_subscription(informer_sub_sender);

            Some(informer_handler.start_ws(config.informer_config())?)
        } else {
            None
        }
    };
    let (rpc_server, ipc_server, ws_server) = {
        let rpc_apis_deps = ApiDependencies {
            client: client.client(),
            miner: Arc::clone(&miner),
            network_control: Arc::clone(&network_service),
            account_provider: ap,
            block_sync: maybe_sync_sender,
        };

        let rpc_server = {
            if !config.rpc.disable.unwrap() {
                let server = setup_rpc_server(&config, &rpc_apis_deps);
                Some(rpc_http_start(server, config.rpc_http_config())?)
            } else {
                None
            }
        };

        let ipc_server = {
            if !config.ipc.disable.unwrap() {
                let server = setup_rpc_server(&config, &rpc_apis_deps);
                Some(rpc_ipc_start(server, config.rpc_ipc_config())?)
            } else {
                None
            }
        };

        let ws_server = {
            if !config.ws.disable.unwrap() {
                let server = setup_rpc_server(&config, &rpc_apis_deps);
                Some(rpc_ws_start(server, config.rpc_ws_config())?)
            } else {
                None
            }
        };

        (rpc_server, ipc_server, ws_server)
    };

    let _snapshot_service = {
        let client = client.client();
        let (tx, rx) = snapshot_notify::create();
        client.engine().register_snapshot_notify_sender(tx);
        if !config.snapshot.disable.unwrap() {
            let service =
                Arc::new(SnapshotService::new(client, rx, config.snapshot.path.unwrap(), config.snapshot.expiration));
            Some(service)
        } else {
            None
        }
    };

    // drop the scheme to free up genesis state.
    drop(scheme);
    client.client().engine().complete_register();

    cinfo!(TEST_SCRIPT, "Initialization complete");

    wait_for_exit();

    if let Some(server) = informer_server {
        server.close_handle().close();
    }
    if let Some(server) = rpc_server {
        server.close_handle().close();
        server.wait();
    }
    if let Some(server) = ipc_server {
        server.close_handle().close();
        server.wait();
    }
    if let Some(server) = ws_server {
        server.close_handle().close();
        server.wait().map_err(|err| format!("Error while closing jsonrpc ws server: {}", err))?;
    }

    Ok(())
}
