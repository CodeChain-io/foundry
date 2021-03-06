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

use crate::config;
use crate::constants::{DEFAULT_DB_PATH, DEFAULT_KEYS_PATH};
use crate::dummy_network_service::DummyNetworkService;
use crate::json::PasswordFile;
use ccore::{
    genesis::Genesis, AccountProvider, AccountProviderError, ChainNotify, Client, ClientConfig, ClientService,
    EngineInfo, EngineType, Miner, MinerService, NullEngine, PeerDb, Solo, Tendermint, NUM_COLUMNS,
};
use ccore::{snapshot_notify, ConsensusEngine, EngineClient};
use cdiscovery::{Config, Discovery};
use cinformer::{handler::Handler, InformerEventSender, InformerService, MetaIoHandler, PubSubHandler, Session};
use ckey::{Ed25519Public as Public, NetworkId, PlatformAddress};
use ckeystore::accounts_dir::RootDiskDirectory;
use ckeystore::KeyStore;
use clogger::{EmailAlarm, LoggerConfig};
use cnetwork::{Filters, ManagingPeerdb, NetworkConfig, NetworkControl, NetworkService, RoutingTable, SocketAddr};
use coordinator::{AppDesc, Coordinator, LinkDesc};
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
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};
use std::{collections::HashMap, path::Path};
use std::{fs, net::IpAddr};

struct ClientWrapper(Arc<Client>);

impl foundry_graphql::ManageSession for ClientWrapper {
    fn new_session(&self, block: ctypes::BlockId) -> coordinator::module::SessionId {
        self.0.new_session(block)
    }

    fn end_session(&self, session: coordinator::module::SessionId) {
        self.0.end_session(session)
    }
}

fn network_start(
    network_id: NetworkId,
    timer_loop: TimerLoop,
    cfg: &NetworkConfig,
    routing_table: Arc<RoutingTable>,
    peer_db: Box<dyn ManagingPeerdb>,
    sender: InformerEventSender,
) -> Result<Arc<NetworkService>, String> {
    let sockaddress = SocketAddr::new(IpAddr::V4(cfg.address), cfg.port);
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
    cfg: &config::Config,
    routing_table: Arc<RoutingTable>,
) -> Result<(), String> {
    let config = Config {
        bucket_size: cfg.discovery_bucket_size,
        t_refresh: cfg.discovery_refresh,
    };
    let use_kademlia = match cfg.discovery_type.as_str() {
        "unstructured" => false,
        "kademlia" => true,
        discovery_type => return Err(format!("Unknown discovery {}", discovery_type)),
    };
    service.register_extension(move |api| Discovery::new(routing_table, config, api, use_kademlia));
    Ok(())
}

fn client_start(
    client_config: &ClientConfig,
    timer_loop: &TimerLoop,
    db: Arc<dyn KeyValueDB>,
    engine: Arc<dyn ConsensusEngine>,
    genesis: &Genesis,
    miner: Arc<Miner>,
    coordinator: Arc<Coordinator>,
) -> Result<ClientService, String> {
    cinfo!(CLIENT, "Starting client");
    let reseal_timer = timer_loop.new_timer_with_name("Client reseal timer");
    let service = ClientService::start(client_config, engine, &genesis, db, miner, coordinator, reseal_timer.clone())
        .map_err(|e| format!("Client service error: {}", e))?;
    reseal_timer.set_handler(Arc::downgrade(&service.client()));

    Ok(service)
}

fn new_miner(
    config: &config::Config,
    engine: Arc<dyn ConsensusEngine>,
    ap: Arc<AccountProvider>,
    db: Arc<dyn KeyValueDB>,
    coordinator: Arc<Coordinator>,
) -> Result<Arc<Miner>, String> {
    let miner = Miner::new(config.miner_options()?, engine, db, coordinator);

    match miner.engine_type() {
        EngineType::PBFT => match &config.engine_signer {
            Some(ref engine_signer) => match miner.set_author(ap, (*engine_signer).into_pubkey()) {
                Err(AccountProviderError::NotUnlocked) => {
                    return Err(
                        format!("The account {} is not unlocked. The key file should exist in the keys_path directory, and the account's password should exist in the password_path file.", engine_signer)
                    )
                }
                Err(e) => return Err(format!("{}", e)),
                _ => (),
            },
            None => (),
        },
        EngineType::Solo => miner
            .set_author(ap, config.engine_signer.map_or(Public::default(), PlatformAddress::into_pubkey))
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

pub fn open_db(cfg: &config::Config, client_config: &ClientConfig) -> Result<Arc<dyn KeyValueDB>, String> {
    let base_path = cfg.base_path.clone();
    let db_path = cfg.db_path.as_ref().map(String::clone).unwrap_or_else(|| base_path + "/" + DEFAULT_DB_PATH);
    // this is for debug
    std::process::Command::new("rm").arg("-rf").arg(&db_path).output().unwrap();

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

pub fn run_node(config: config::Config, module_arguments: BTreeMap<String, String>) -> Result<(), String> {
    // increase max number of open files
    raise_fd_limit();

    let timer_loop = TimerLoop::new(2);

    let time_gap_params = config.create_time_gaps();

    let mut app_desc = {
        let app_desc_path = &config.app_desc_path;
        let app_desc_string = fs::read_to_string(app_desc_path)
            .map_err(|err| format!("Foundry failed to read an app desc at {}: {}", app_desc_path, err))?;
        AppDesc::from_str(&app_desc_string).map_err(|err| format!("Foundry failed to parse app descriptor: {}", err))?
    };
    app_desc
        .merge_params(&module_arguments)
        .map_err(|err| format!("Foundry failed to merge params you supplied into the app descriptor. {}", err))?;
    let mut link_desc = {
        let link_desc_path = &config.link_desc_path;
        let link_desc_string = fs::read_to_string(link_desc_path)
            .map_err(|err| format!("Foundry failed to read a link desc at {}: {}", link_desc_path, err))?;
        LinkDesc::from_str(&link_desc_string)
            .map_err(|err| format!("Foundry failed to parse link descriptor: {}", err))?
    };
    link_desc
        .merge_params(&module_arguments)
        .map_err(|err| format!("Foundry failed to merge params you supplied into the link descriptor. {}", err))?;
    let coordinator = Arc::new(
        Coordinator::from_descs(&app_desc, &link_desc)
            .map_err(|err| format!("Failed to parse app descriptor and link descriptor: {}", err))?,
    );

    let genesis = Genesis::new(app_desc.host.genesis, coordinator.as_ref());

    let engine: Arc<dyn ConsensusEngine> = {
        let engine_config = app_desc.host.engine;
        match engine_config {
            coordinator::app_desc::Engine::Null => Arc::new(NullEngine::default()),
            coordinator::app_desc::Engine::Solo => Arc::new(Solo::new()),
            coordinator::app_desc::Engine::Tendermint(tendermint) => Tendermint::new((*tendermint).into()),
        }
    };
    engine.register_time_gap_config_to_worker(time_gap_params);

    let pf = load_password_file(&config.password_path)?;
    let base_path = config.base_path.clone();
    let keys_path = config.keys_path.as_ref().map(String::clone).unwrap_or_else(|| base_path + "/" + DEFAULT_KEYS_PATH);
    let ap = prepare_account_provider(&keys_path)?;
    unlock_accounts(&*ap, &pf)?;

    let client_config: ClientConfig = Default::default();
    let db = open_db(&config, &client_config)?;

    let miner = new_miner(&config, Arc::clone(&engine), ap, Arc::clone(&db), coordinator.clone())?;
    let client =
        client_start(&client_config, &timer_loop, db, Arc::clone(&engine), &genesis, miner.clone(), coordinator)?;
    miner.recover_from_db();

    let engine_graphql_handler = foundry_graphql_engine::EngineLevelGraphQlHandler::new(client.client());

    let _graphql_webserver = {
        use foundry_graphql::{GraphQlRequestHandler, ServerData};
        use std::net::{Ipv4Addr, SocketAddr};

        let mut handlers: HashMap<String, GraphQlRequestHandler> = client
            .client()
            .graphql_handlers()
            .iter()
            .map(|(k, v)| {
                (k.to_string(), GraphQlRequestHandler {
                    handler: Arc::clone(v),
                    session_needed: true,
                })
            })
            .collect();

        handlers.insert("engine".to_owned(), GraphQlRequestHandler {
            handler: Arc::new(engine_graphql_handler),
            session_needed: false,
        });

        let server_data = ServerData::new(Arc::new(ClientWrapper(client.client())), handlers);
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.graphql_port);
        foundry_graphql::run_server(server_data, socket)
    };

    let instance_id = config.instance_id.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time should be later than unix epoch")
            .subsec_nanos() as usize,
    );
    let email_alarm = if config.email_alarm_enable {
        let to = config.email_alarm_to.clone().ok_or_else(|| "email-alarm-to is not specified".to_string())?;
        let sendgrid_key = config
            .email_alarm_sendgrid_key
            .clone()
            .ok_or_else(|| "email-alarm-sendgrid-key is not specified".to_string())?;
        Some(EmailAlarm::new(to, sendgrid_key, client.client().network_id().to_string()))
    } else {
        None
    };
    clogger::init(&LoggerConfig::new(instance_id), email_alarm.clone())
        .expect("Logger must be successfully initialized");
    if let Some(email_alarm) = email_alarm {
        panic_hook::set_with_email_alarm(email_alarm);
    }

    // FIXME: unbound would cause memory leak.
    // FIXME: The full queue should be handled.
    // This will be fixed soon.
    let (informer_sub_sender, informer_sub_receiver) = unbounded();
    let informer_event_sender = {
        if config.informer_enable {
            let (service, event_sender) = InformerService::new(informer_sub_receiver, client.client());
            service.run_service();
            event_sender
        } else {
            InformerEventSender::null_notifier()
        }
    };

    let mut _maybe_sync = None;

    engine.register_chain_notify(client.client().as_ref());

    let _network_service: Arc<dyn NetworkControl> = {
        if config.network_enable {
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

            if config.discovery_enable {
                discovery_start(&service, &config, routing_table)?;
            } else {
                cwarn!(DISCOVERY, "Node runs without discovery extension");
            }

            if config.sync_enable {
                let sync_sender = {
                    let client = client.client();
                    let snapshot_target = match (config.snapshot_hash, config.snapshot_number) {
                        (Some(hash), Some(num)) => Some((hash, num)),
                        _ => None,
                    };
                    let snapshot_dir = config.snapshot_path.clone();
                    service.register_extension(move |api| {
                        BlockSyncExtension::new(client, api, snapshot_target, snapshot_dir)
                    })
                };
                let sync = Arc::new(BlockSyncSender::from(sync_sender));
                client.client().add_notify(Arc::downgrade(&sync) as Weak<dyn ChainNotify>);
                _maybe_sync = Some(sync); // Hold sync to ensure it not to be destroyed.
            }
            if config.tx_relay_enable {
                let client = client.client();
                service.register_extension(move |api| TransactionSyncExtension::new(client, api));
            }

            engine.register_network_extension_to_service(&service);

            service
        } else {
            Arc::new(DummyNetworkService::new())
        }
    };

    let informer_server = {
        if config.informer_enable {
            let io: PubSubHandler<Arc<Session>> = PubSubHandler::new(MetaIoHandler::default());
            let mut informer_handler = Handler::new(io);
            informer_handler.event_subscription(informer_sub_sender);

            Some(informer_handler.start_ws(config.informer_config())?)
        } else {
            None
        }
    };

    let _snapshot_service = {
        let client = client.client();
        let (tx, rx) = snapshot_notify::create();
        client.engine().register_snapshot_notify_sender(tx);
        if config.snapshot_enable {
            let service = Arc::new(SnapshotService::new(client, rx, config.snapshot_path, config.snapshot_expiration));
            Some(service)
        } else {
            None
        }
    };

    client.client().engine().complete_register();

    cinfo!(TEST_SCRIPT, "Initialization complete");

    wait_for_exit();

    if let Some(server) = informer_server {
        server.close_handle().close();
    }

    Ok(())
}
