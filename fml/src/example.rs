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

/// First, you have to declare your global context
#[global_context]
struct Context {
    //...
}

/// 4 categories of Handle traits

type Transaction = u8;
/// 1. HostCallBack
/// - You implement it, and never call it.
/// - Host might call methods anytime
/// - There will be only one trait
trait HostCallBack {
    fn excute_transaction(tx: Transaction);
}

/// 2. HostCallable
/// - You call it, and never implement it
/// - Host will provide this.
/// - There will be only one trait
trait HostApi {
    fn query_something();
}

/// 3. ApplicationCallBack
/// - You implement it, and never call it
/// - Other applications call methods anytime
/// - There might be multiple traits
trait HandleToBeGivenToStakingModule {
    fn get_balance() -> u64;
}

/// 4. ApplicationCallable
/// - You call it, and never implement it
/// - Application will provide this
/// - There might be multiple traits
trait HandleToBeGivenByStakingModule {
    fn get_delegation() -> u64;
}

// Usage
// Macro will generate corressponding struct for each trait
// To find such identifier of each, there will be another macro to retrieve that
// For 2 Callbacks, they will utilize module's global context

impl HandleToBeGivenToStakingModule for HandleToBeGivenToStakingModule_Type {
    fn get_balance() -> u64 {
        // ...
    }
}

fn main() {
    init_ipc();
    init_server();
    init_else();

    do_some_initializaion_of_your_customized_global_context();
    let host_api = host_initial_handle();

    wait(); // You lose program control here. Now only call-backs are executable.
}
