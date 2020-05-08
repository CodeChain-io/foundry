// Copyright 2018-2020 Kodebox, Inc.
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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import {
    faucetAddress,
    faucetSecret,
    validator0Address,
    validator1Address,
    validator2Address,
    validator3Address
} from "../helper/constants";
import { wait } from "../helper/promise";
import { makeRandomH256 } from "../helper/random";
import CodeChain from "../helper/spawn";

(async () => {
    let nodes: CodeChain[];

    const validatorAddresses = [
        validator0Address,
        validator1Address,
        validator2Address,
        validator3Address
    ];
    nodes = validatorAddresses.map(address => {
        return new CodeChain({
            chain: `${__dirname}/../scheme/tendermint-tps.json`,
            argv: [
                "--engine-signer",
                address.toString(),
                "--password-path",
                "test/tendermint/password.json",
                "--force-sealing",
                "--no-discovery",
                "--enable-devel-api"
            ],
            additionalKeysPath: "tendermint/keys"
        });
    });
    await Promise.all(nodes.map(node => node.start()));

    await Promise.all([
        nodes[0].connect(nodes[1]),
        nodes[0].connect(nodes[2]),
        nodes[0].connect(nodes[3]),
        nodes[1].connect(nodes[2]),
        nodes[1].connect(nodes[3]),
        nodes[2].connect(nodes[3])
    ]);
    await Promise.all([
        nodes[0].waitPeers(4 - 1),
        nodes[1].waitPeers(4 - 1),
        nodes[2].waitPeers(4 - 1),
        nodes[3].waitPeers(4 - 1)
    ]);

    const transactions = [];
    const numTransactions = parseInt(process.env.TEST_NUM_TXS || "10000", 10);
    const baseSeq = (await nodes[0].rpc.chain.getSeq({
        address: faucetAddress.toString()
    }))!;

    for (let i = 0; i < numTransactions; i++) {
        const value = makeRandomH256();
        const pubkey = nodes[0].testFramework.util.getPublicFromPrivate(value);
        const recipient = nodes[0].testFramework.core.classes.Address.fromPublic(
            pubkey,
            { networkId: "tc" }
        );
        const transaction = nodes[0].testFramework.core
            .createPayTransaction({
                recipient,
                quantity: 1
            })
            .sign({
                secret: faucetSecret,
                seq: baseSeq + i,
                fee: 10
            });
        transactions.push(transaction);
    }

    for (let i = numTransactions - 1; i > 0; i--) {
        await nodes[0].rpc.mempool.sendSignedTransaction({
            tx: transactions[i].rlpBytes().toString("hex")
        });
    }
    const startTime = new Date();
    console.log(`Start at: ${startTime}`);
    await nodes[0].rpc.mempool.sendSignedTransaction({
        tx: transactions[0].rlpBytes().toString("hex")
    });

    while (true) {
        let flag = true;
        for (let i = 0; i < 4; i++) {
            const hash = transactions[numTransactions - 1].hash();
            const result = await nodes[i].rpc.chain.containsTransaction({
                transactionHash: `0x${hash.toString()}`
            });

            console.log(`Node ${i} result: ${result}`);
            if (!result) {
                flag = false;
                break;
            }
        }
        if (flag) {
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

    await Promise.all(nodes.map(node => node.clean()));
})().catch(console.error);
