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

import { faucetAddress, faucetSecret } from "../helper/constants";
import { wait } from "../helper/promise";
import { makeRandomH256 } from "../helper/random";
import CodeChain from "../helper/spawn";

(async () => {
    const numTransactions = parseInt(process.env.TEST_NUM_TXS || "10000", 10);
    const rpcPort = parseInt(process.env.TEST_RPC_PORT || "8080", 10);

    const node = new CodeChain({
        argv: ["--reseal-min-period", "0"],
        rpcPort
    });

    const transactions = [];
    const baseSeq = (await node.rpc.chain.getSeq({
        address: faucetAddress.toString()
    }))!;

    for (let i = 0; i < numTransactions; i++) {
        const value = makeRandomH256();
        const pubkey = node.testFramework.util.getPublicFromPrivate(value);
        const recipient = node.testFramework.core.classes.Address.fromPublic(
            pubkey,
            { networkId: "tc" }
        );
        const transaciton = node.testFramework.core
            .createPayTransaction({
                recipient,
                quantity: 1
            })
            .sign({
                secret: faucetSecret,
                seq: baseSeq + i,
                fee: 10
            });
        transactions.push(transaciton);
    }

    for (let i = numTransactions - 1; i > 0; i--) {
        await node.rpc.mempool.sendSignedTransaction({
            tx: transactions[i].rlpBytes().toString("hex")
        });
    }
    const startTime = new Date();
    console.log(`Start at: ${startTime}`);
    await node.rpc.mempool.sendSignedTransaction({
        tx: transactions[0].rlpBytes().toString("hex")
    });

    while (true) {
        const hash = transactions[numTransactions - 1].hash();
        const result = await node.rpc.chain.containsTransaction({
            transactionHash: hash.toString()
        });
        console.log(`Node result: ${result}`);
        if (result) {
            break;
        }

        await wait(500);
    }
    const endTime = new Date();
    console.log(`End at: ${endTime}`);
    const tps =
        (numTransactions * 1000.0) / (endTime.getTime() - startTime.getTime());
    console.log(
        `Elapsed time (ms): ${endTime.getTime() - startTime.getTime()}`
    );
    console.log(`TPS: ${tps}`);
})().catch(console.error);
