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

use super::{Import, LinkInfo, Weaver};
use cmodule::link;
use cmodule::link::{Linkable, Linker, Port, LINKERS};
use cmodule::sandbox::Sandbox;
use linkme::distributed_slice;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::*;
use std::sync::Arc;

type LinkRecordPerExporter = HashMap<String, LinkRecord>;

struct MockSandbox {
    name: &'static str,
    linkers: &'static [&'static str],
    // importing module -> exporting module -> list of Imports
    mappings: Arc<RwLock<LinkRecordPerExporter>>,
}

impl MockSandbox {
    fn new(name: &'static str, linkers: &'static [&'static str]) -> Self {
        MockSandbox {
            name,
            linkers,
            mappings: Default::default(),
        }
    }
}

impl Linkable for MockSandbox {
    fn supported_linkers(&self) -> &'static [&'static str] {
        self.linkers
    }
    fn new_port(&mut self) -> Box<dyn Port> {
        Box::new(MockPort::new(self.name.to_owned(), Arc::clone(&self.mappings)))
    }
    fn seal(&mut self) {}
}
impl Sandbox for MockSandbox {}

struct MockPort {
    owner: String,
    mappings: Arc<RwLock<LinkRecordPerExporter>>,
    imports: Vec<String>,
    exports: Vec<usize>,
}

impl MockPort {
    fn new(owner: String, mappings: Arc<RwLock<LinkRecordPerExporter>>) -> MockPort {
        MockPort {
            owner,
            mappings,
            exports: Vec::new(),
            imports: Vec::new(),
        }
    }
}

impl Port for MockPort {
    fn export(&mut self, ids: &[usize]) {
        self.exports = ids.to_owned();
    }
    fn import(&mut self, slots: &[&str]) {
        self.imports = slots.iter().map(|s| (*s).to_owned()).collect();
    }
}

struct LinkRecord {
    linker: &'static str,
    imports: Vec<ImportRecord>,
}

struct ImportRecord {
    from: usize,
    to: String,
}

#[distributed_slice(LINKERS)]
fn mock_linker_a() -> (&'static str, Arc<dyn Linker>) {
    let name = "linker-a";
    (name, Arc::new(MockLinker::new(name)))
}

#[distributed_slice(LINKERS)]
fn mock_linker_b() -> (&'static str, Arc<dyn Linker>) {
    let name = "linker-b";
    (name, Arc::new(MockLinker::new(name)))
}

struct MockLinker {
    id: &'static str,
}

impl MockLinker {
    fn new(id: &'static str) -> Self {
        MockLinker {
            id,
        }
    }
}

impl Linker for MockLinker {
    fn link(&self, a: &mut dyn Port, b: &mut dyn Port) -> Result<(), link::Error> {
        fn push_mappings(linker: &'static str, src_port: &MockPort, dst_port: &MockPort) {
            let mut mappings = dst_port.mappings.write();
            for (from, to) in
                src_port.exports.iter().zip(dst_port.imports.iter()).map(|(export, import)| (*export, import.clone()))
            {
                let link_record = mappings.entry(src_port.owner.clone()).or_insert_with(|| LinkRecord {
                    linker,
                    imports: Vec::new(),
                });
                link_record.imports.push(ImportRecord {
                    from,
                    to,
                });
            }
        }

        let a = a.mut_any();
        let b = b.mut_any();

        let a = a.downcast_ref::<MockPort>().ok_or(link::Error::UnsupportedPortType {
            id: "not MockPort",
        })?;
        let b = b.downcast_ref::<MockPort>().ok_or(link::Error::UnsupportedPortType {
            id: "not MockPort",
        })?;

        push_mappings(self.id, a, b);
        push_mappings(self.id, b, a);

        Ok(())
    }
}

type ModuleDef = (
    &'static ModuleName,
    &'static [&'static LinkerName],
    &'static [&'static ExportName],
    &'static [(&'static ModuleName, &'static [ImportDef])],
);
type ModuleName = str;
type ExportName = str;
type LinkerName = str;
type ImportDef = (&'static str, &'static str);
type TxType = str;

fn build_modules(list: &[ModuleDef]) -> Vec<(String, Arc<RwLock<LinkRecordPerExporter>>, LinkInfo)> {
    list.iter()
        .map(|(name, linkers, exports, imports)| {
            let sandbox = MockSandbox::new(*name, linkers);
            ((*name).to_owned(), Arc::clone(&sandbox.mappings), LinkInfo {
                linkable: RefCell::new(Box::new(sandbox)),
                exports: build_exports(*exports),
                imports: RefCell::new(build_imports(*imports)),
            })
        })
        .collect()
}

fn build_exports(list: &[&ExportName]) -> BTreeMap<String, usize> {
    list.iter().enumerate().map(|(id, export)| ((*export).to_owned(), id)).collect()
}

fn build_imports(list: &[(&ModuleName, &[ImportDef])]) -> HashMap<String, Vec<Import>> {
    list.iter()
        .map(|(module, import_pairs)| {
            (
                (*module).to_owned(),
                (*import_pairs)
                    .iter()
                    .map(|(from, to)| Import {
                        from: (*from).to_owned(),
                        to: (*to).to_owned(),
                    })
                    .collect(),
            )
        })
        .collect()
}

fn tx_owners(list: &[(&TxType, &ModuleName)]) -> HashMap<String, String> {
    list.iter().map(|(ty, owner)| ((*ty).to_owned(), (*owner).to_owned())).collect()
}

fn new_test_weaver_with_exports() -> Weaver {
    let modules = build_modules(&[
        ("a", &[], &["service-a", "service-b.tx-type-a", "service-b.tx-type-b"], &[]),
        ("b", &[], &["service-a", "service-b"], &[]),
        ("c", &[], &["service-a", "service-b", "service-c"], &[]),
        ("d", &[], &[], &[]),
    ])
    .into_iter()
    .map(|(module, _, link_info)| (module, link_info))
    .collect();

    let tx_owners = tx_owners(&[("tx-type-a", "a"), ("tx-type-b", "a"), ("tx-type-c", "b")]);

    Weaver {
        services: Default::default(),
        modules,
        tx_owners,
    }
}

#[test]
fn import_single_tx_service_from_multi_tx_owner() {
    let mut weaver = new_test_weaver_with_exports();
    weaver.import_tx_services("c", &["service-a"]);
    let imports = weaver.modules["c"].imports.borrow();

    assert!(!imports.contains_key("c"));
    assert!(!imports.contains_key("d"));

    let import_list = imports.get("a").expect("should have imported from module 'a'");
    assert!(import_list.iter().all(
        |Import {
             from,
             to,
         }| from == "service-a" && (to == "@tx/tx-type-a/service-a" || to == "@tx/tx-type-b/service-a")
    ));

    let import_list = imports.get("b").expect("should have imported from module 'b'");
    assert!(import_list.iter().all(
        |Import {
             from,
             to,
         }| from == "service-a" && to == "@tx/tx-type-c/service-a"
    ));
}

#[test]
fn import_per_tx_service_from_multi_tx_owner() {
    let mut weaver = new_test_weaver_with_exports();
    weaver.import_tx_services("c", &["service-b"]);
    let imports = weaver.modules["c"].imports.borrow();

    assert!(!imports.contains_key("c"));
    assert!(!imports.contains_key("d"));

    let import_list = imports.get("a").expect("should have imported from module 'a'");
    assert!(import_list.iter().all(
        |Import {
             from,
             to,
         }| (from == "service-b.tx-type-a" && to == "@tx/tx-type-a/service-b")
            || (from == "service-b.tx-type-b" && to == "@tx/tx-type-b/service-b")
    ));

    let import_list = imports.get("b").expect("should have imported from module 'b'");
    assert!(import_list.iter().all(
        |Import {
             from,
             to,
         }| from == "service-b" && to == "@tx/tx-type-c/service-b"
    ));
}

#[test]
fn host_possibly_imports_no_service() {
    let mut weaver = new_test_weaver_with_exports();
    weaver
        .import_services("d", &[((Included(0), Excluded(2)), "non-existing-service")])
        .expect("should be ok without an instance of the designated service");
}

#[test]
fn host_must_import_at_least_a_service() {
    let mut weaver = new_test_weaver_with_exports();
    weaver
        .import_services("d", &[((Included(1), Unbounded), "non-existing-service")])
        .expect_err("should be at least one instance of the designated service");
    weaver
        .import_services("d", &[((Included(1), Unbounded), "service-c")])
        .expect("should be at least one instance of the designated service");

    let imports = weaver.modules["d"].imports.borrow();
    let imports_from_c = imports.get("c").expect("there must be an import from 'c'");

    assert_eq!(imports_from_c.len(), 1);

    let Import {
        from,
        to,
    } = imports_from_c.first().unwrap();

    assert_eq!(from, "service-c");
    assert_eq!(to, "service-c/c");
}

#[test]
fn host_imports_multiple_instances_of_a_service() {
    let mut weaver = new_test_weaver_with_exports();
    weaver
        .import_services("d", &[((Included(1), Unbounded), "service-a")])
        .expect("should be at least one instance of the designated service");
    weaver
        .import_services("d", &[((Included(1), Unbounded), "service-b")])
        .expect("should be at least one instance of the designated service");

    let imports = weaver.modules["d"].imports.borrow();

    let imports_from_a = imports.get("a").expect("there must be an import from 'a'");
    let imports_from_b = imports.get("b").expect("there must be an import from 'b'");
    let imports_from_c = imports.get("c").expect("there must be an import from 'c'");

    assert!(imports_from_a.iter().all(
        |Import {
             from,
             to,
         }| (from == "service-a" && to == "service-a/a")
    ));
    assert!(imports_from_b.iter().all(
        |Import {
             from,
             to,
         }| (from == "service-a" && to == "service-a/b") || (from == "service-b" && to == "service-b/b")
    ));
    assert!(imports_from_c.iter().all(
        |Import {
             from,
             to,
         }| (from == "service-a" && to == "service-a/c") || (from == "service-b" && to == "service-b/c")
    ));
}

#[test]
#[allow(clippy::many_single_char_names)]
fn link_complex() {
    let modules = build_modules(&[
        ("a", &["linker-9", "linker-a"], &["0", "1", "2"], &[
            ("b", &[("0", "import-b0"), ("1", "import-b1")]),
            ("c", &[("1", "import-c1")]),
        ]),
        ("b", &["linker-a", "linker-b"], &["0", "1"], &[]),
        ("c", &["linker-a"], &["0", "1", "2", "3"], &[("a", &[("2", "import-a2")]), ("b", &[("1", "import-b1")])]),
        ("d", &["linker-a"], &[], &[("a", &[("1", "import-a1")])]),
        ("e", &["linker-a"], &[], &[]),
    ]);

    let records = modules
        .iter()
        .map(|(module, record_map, _)| (module.clone(), Arc::clone(record_map)))
        .collect::<HashMap<_, _>>();

    let modules = modules.into_iter().map(|(module, _, link_info)| (module, link_info)).collect();

    let tx_owners = tx_owners(&[]);

    let mut weaver = Weaver {
        services: Default::default(),
        modules,
        tx_owners,
    };

    weaver.link_all().expect("should complete without an error");

    let a = records.get("a").expect("must be a LinkRecord for a").read();
    assert_eq!(a.len(), 2);
    let link_record = a.get("b").expect("must import from b");
    assert_eq!(link_record.linker, "linker-a");
    assert!(link_record.imports.iter().all(
        |ImportRecord {
             from,
             to,
         }| *from == 0 && to == "import-b0" || *from == 1 && to == "import-b1"
    ));
    let link_record = a.get("c").expect("must import from c");
    assert_eq!(link_record.linker, "linker-a");
    assert!(link_record.imports.iter().all(
        |ImportRecord {
             from,
             to,
         }| *from == 1 && to == "import-c1"
    ));

    let b = records.get("b").expect("must be a LinkRecord for b").read();
    assert_eq!(b.len(), 0);

    let c = records.get("c").expect("must be a LinkRecord for c").read();
    assert_eq!(c.len(), 2);
    let link_record = c.get("a").expect("must import from a");
    assert_eq!(link_record.linker, "linker-a");
    assert!(link_record.imports.iter().all(
        |ImportRecord {
             from,
             to,
         }| *from == 2 && to == "import-a2"
    ));
    let link_record = c.get("b").expect("must import from b");
    assert_eq!(link_record.linker, "linker-a");
    assert!(link_record.imports.iter().all(
        |ImportRecord {
             from,
             to,
         }| *from == 1 && to == "import-b1"
    ));

    let d = records.get("d").expect("must be a LinkRecord for d").read();
    assert_eq!(d.len(), 1);
    let link_record = d.get("a").expect("must import from a");
    assert_eq!(link_record.linker, "linker-a");
    assert!(link_record.imports.iter().all(
        |ImportRecord {
             from,
             to,
         }| *from == 1 && to == "import-a1"
    ));

    let e = records.get("e").expect("must be a LinkRecord for e").read();
    assert_eq!(e.len(), 0);
}
