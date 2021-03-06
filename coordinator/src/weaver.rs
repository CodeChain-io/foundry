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
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::RangeBounds;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context};

use cmodule::link::{best_linker, Port};
use cmodule::sandbox::{sandboxer, Sandbox};

use crate::{
    app_desc::{AppDesc, Constructor, GlobalName, ModuleSetup, Namespaced, SimpleName},
    link_desc::{self, LinkDesc},
};
use crate::{Occurrences, Services};
use crate::{HOST_ID, SERVICES_FOR_HOST, TX_SERVICES_FOR_HOST};

#[cfg(test)]
mod test;

type ExportIdMap = BTreeMap<String, usize>;
type ServiceSpec<'a> = (&'a str, &'a dyn erased_serde::Serialize);

#[derive(Default)]
pub(super) struct Weaver {
    modules: HashMap<String, LinkInfo>,
    tx_owners: HashMap<String, String>,
    services: Arc<RwLock<Option<Services>>>,
}

struct LinkInfo {
    linkable: RefCell<Box<dyn Sandbox>>,
    exports: ExportIdMap,
    imports: RefCell<HashMap<String, Vec<Import>>>,
}

#[derive(Debug)]
struct Import {
    from: String,
    to: String,
}

impl Weaver {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn weave(
        mut self,
        app_desc: &AppDesc,
        link_desc: &LinkDesc,
    ) -> anyhow::Result<(Vec<Box<dyn Sandbox>>, Services)> {
        self.modules.reserve(app_desc.modules.len());

        let host_module = link_desc.get("host").ok_or_else(|| anyhow!("can't find host module in app descriptor"))?;
        self.process_host(host_module);
        self.process_modules(app_desc, link_desc)?;
        self.tx_owners =
            app_desc.transactions.iter().map(|(tx_type, module)| (tx_type.clone(), (**module).clone())).collect();
        self.import_tx_services_for_modules(&app_desc.modules);
        self.import_tx_services(HOST_ID, TX_SERVICES_FOR_HOST);
        self.import_services(HOST_ID, SERVICES_FOR_HOST)?;
        self.link_all()?;

        let linkables = self.modules.into_iter().map(|(_, link_info)| link_info.linkable.into_inner()).collect();

        Ok((linkables, self.services.write().take().unwrap()))
    }

    fn process_host(&mut self, link: &link_desc::ModuleSetup) {
        let (exports, init_exports) = Self::process_exports(&link.exports);
        let imports = Self::process_imports(&link.imports);
        let imports = RefCell::new(imports);

        let init_exports: Vec<(String, Vec<u8>)> = init_exports
            .iter()
            .map(|(name, data)| {
                let mut buffer = Vec::<u8>::new();
                let cbor =
                    &mut serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(std::io::Cursor::new(&mut buffer)));
                data.erased_serialize(&mut erased_serde::Serializer::erase(cbor)).unwrap();
                (name.to_string(), buffer)
            })
            .collect();

        self.services.write().replace(Default::default());
        let linkable = foundry_module_rt::create_foundry_module(
            super::linkable::HostModule {
                services: Arc::clone(&self.services),
            },
            &init_exports,
        );

        let linkable = RefCell::new(Box::new(linkable) as Box<dyn Sandbox>);

        self.modules.insert(HOST_ID.to_owned(), LinkInfo {
            linkable,
            exports,
            imports,
        });
    }

    fn process_modules(&mut self, app_desc: &AppDesc, link_desc: &LinkDesc) -> anyhow::Result<()> {
        for (name, setup) in app_desc.modules.iter() {
            let link =
                link_desc.get(name).ok_or_else(|| anyhow!("Failed to find module {} in the link descriptor", name))?;

            let sandboxer_id = if link.sandboxer.is_empty() {
                &link_desc.default_sandboxer
            } else {
                &link.sandboxer
            };
            let sandboxer = sandboxer(sandboxer_id).ok_or_else(|| anyhow!("Sandboxer unknown: {}", sandboxer_id))?;
            // FIXME: assumes that path is not used to locate a module here. Fix this later when we
            //        introduce a proper module registry.
            let path = if sandboxer_id == "multi-process" {
                format!("../target/debug/{:x}", &setup.hash.value)
            } else {
                format!("{:x}", &setup.hash.value)
            };
            let (exports, init_exports) = Self::process_exports(&link.exports);
            let imports = RefCell::new(Self::process_imports(&link.imports));
            let linkable = RefCell::new(sandboxer.load(&path, &link.init_config, &*init_exports)?);

            self.modules.insert((*name).clone(), LinkInfo {
                linkable,
                exports,
                imports,
            });
        }

        Ok(())
    }

    fn import_tx_services_for_modules(&mut self, modules: &HashMap<SimpleName, ModuleSetup>) {
        for (module, services) in modules.iter().filter_map(|(module, setup)| {
            if setup.transactions.is_empty() {
                None
            } else {
                Some((module, &setup.transactions))
            }
        }) {
            let exports: Box<_> = services.iter().map(|name| &**name as &str).collect();
            self.import_tx_services(&**module, &exports);
        }
    }

    fn link_all(&mut self) -> anyhow::Result<()> {
        let mut linked_pairs = HashSet::new();

        fn set_imports(
            export_ids: &ExportIdMap,
            import_port: &mut Box<dyn Port>,
            export_port: &mut Box<dyn Port>,
            imports: &[Import],
        ) {
            let slots: Vec<&str> = imports.iter().map(|import| &*import.to).collect();
            import_port.import(&slots);

            let mut exports = Vec::with_capacity(imports.len());
            for Import {
                from,
                ..
            } in imports.iter()
            {
                exports.push(*export_ids.get(from).expect("We checked whether exporter exist in validate"));
            }
            export_port.export(&exports);
        }

        static NO_IMPORT: Vec<Import> = Vec::new();

        for (a, link_info_a) in self.modules.iter() {
            let imports_into_a = link_info_a.imports.borrow();
            for (b, imports_from_b) in imports_into_a.iter() {
                let pair = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };

                if !linked_pairs.insert(pair) {
                    continue
                }

                let link_info_b = &self.modules[b];
                let exports_from_b = &link_info_b.exports;

                let mut linkable_a = link_info_a.linkable.borrow_mut();
                let mut linkable_b = link_info_b.linkable.borrow_mut();

                let linker = best_linker(&**linkable_a, &**linkable_b)
                    .with_context(|| format!("no linker for a pair: {} - {}", a, b))?;

                let mut port_a = linkable_a.new_port();
                let mut port_b = linkable_b.new_port();

                set_imports(exports_from_b, &mut port_a, &mut port_b, imports_from_b);

                let exports_from_a = &link_info_a.exports;
                let imports_into_b = link_info_b.imports.borrow();
                let imports_from_a = imports_into_b.get(a).unwrap_or(&NO_IMPORT);

                set_imports(exports_from_a, &mut port_b, &mut port_a, imports_from_a);

                linker.link(&mut *port_a, &mut *port_b)?;
            }
        }

        self.modules.iter().for_each(|(_, link_info)| link_info.linkable.borrow_mut().seal());

        Ok(())
    }

    fn process_exports(exports: &Namespaced<Constructor>) -> (ExportIdMap, Vec<ServiceSpec>) {
        let mut export_ids = BTreeMap::new();
        let mut init_exports: Vec<ServiceSpec> = Vec::with_capacity(exports.len());

        for (
            i,
            (
                export_name,
                Constructor {
                    name,
                    args,
                },
            ),
        ) in exports.iter().enumerate()
        {
            export_ids.insert(export_name.clone(), i);
            init_exports.push((name, args));
        }

        (export_ids, init_exports)
    }

    fn process_imports(imports: &Namespaced<GlobalName>) -> HashMap<String, Vec<Import>> {
        let mut imports_per_module = HashMap::with_capacity(imports.len());
        for (import, export) in imports.iter() {
            let export_from = export.module().to_owned();
            let export_name = export.name().to_owned();
            let import_list = imports_per_module.entry(export_from).or_insert_with(Vec::new);
            import_list.push(Import {
                from: export_name,
                to: import.clone(),
            });
        }

        imports_per_module
    }

    fn import_tx_services(&mut self, module: &str, services: &[&str]) {
        let imports = &mut self.modules[module].imports.borrow_mut();

        for (tx_type, tx_owner) in self.tx_owners.iter().filter(|(_, owner)| *owner != module) {
            let exports = &self.modules[tx_owner].exports;

            for &service in services {
                let type_specific_export = format!("{}.{}", service, tx_type);
                let export = if exports.contains_key(&type_specific_export) {
                    type_specific_export
                } else if exports.contains_key(service) {
                    service.to_owned()
                } else {
                    continue
                };

                // import only if such an export exists
                imports.entry(tx_owner.to_owned()).or_default().push(Import {
                    from: export,
                    to: format!("@tx/{}/{}", tx_type, service),
                });
            }
        }
    }

    fn import_services(&mut self, module: &str, services: &[(Occurrences, &str)]) -> anyhow::Result<()> {
        let imports = &mut self.modules[module].imports.borrow_mut();
        let mut counts = HashMap::with_capacity(services.len());

        for (module, link_info) in self.modules.iter().filter(|(name, _)| *name != module) {
            let exports = &link_info.exports;
            for (_bounds, service) in services {
                if exports.contains_key(*service) {
                    imports.entry(module.clone()).or_default().push(Import {
                        from: (*service).to_owned(),
                        to: format!("{}/{}", *service, module),
                    });
                    let val: &mut usize = counts.entry((*service).to_owned()).or_default();
                    *val += 1;
                }
            }
        }

        for (bounds, service) in services {
            let count = counts.get(*service).unwrap_or(&0);
            if !bounds.contains(count) {
                bail!(
                    "The number of '{}' ({}) instances doesn't conform to the specification of '{:?}'",
                    *service,
                    count,
                    bounds
                )
            }
        }

        Ok(())
    }
}
