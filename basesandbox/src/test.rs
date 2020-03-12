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

#[cfg(test)]
mod tests {
    use crate::execution::executor;
    use crate::ipc::Ipc;

    #[test]
    fn simple_rs() {
        let mut ctx =
            executor::execute::<crate::execution::IpcUnixDomainSocket>("./../target/debug/tm_simple_rs", "unittest")
                .unwrap();

        ctx.ipc.send(b"Hello?\0");
        let r = ctx.ipc.recv();
        assert_eq!(r, b"I'm here!\0");
    }
}
