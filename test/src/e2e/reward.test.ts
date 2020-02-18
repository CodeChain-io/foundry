// Copyright 2018-2019 Kodebox, Inc.
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

import { expect } from "chai";
import "mocha";
import {
    aliceAddress,
    aliceSecret,
    bobAddress,
    carolAddress,
    daveAddress,
    faucetAccointId,
    faucetAddress
} from "../helper/constants";
import CodeChain from "../helper/spawn";

describe("Reward = 50, 1 miner", function() {
    // FIXME: Change Number to U64
    const MIN_FEE_PAY = 10;
    const BLOCK_REWARD = 50;
    const FAUCET_INITIAL_CCS = 18000000000000000000;

    let node: CodeChain;

    beforeEach(async function() {
        node = new CodeChain({
            chain: `${__dirname}/../scheme/solo-block-reward-50.json`,
            argv: ["--author", aliceAddress.toString(), "--force-sealing"]
        });
        await node.start();
    });

    it("Mining an empty block", async function() {
        await node.rpc.devel!.startSealing();
        expect(
            +(await node.rpc.chain.getBalance({
                address: faucetAddress.toString()
            }))!
        ).to.equal(FAUCET_INITIAL_CCS);
        expect(
            +(await node.rpc.chain.getBalance({
                address: aliceAddress.toString()
            }))!
        ).to.equal(BLOCK_REWARD);
        expect(
            +(await node.rpc.chain.getBalance({
                address: bobAddress.toString()
            }))!
        ).to.equal(0);
        expect(
            +(await node.rpc.chain.getBalance({
                address: carolAddress.toString()
            }))!
        ).to.equal(0);
        expect(
            +(await node.rpc.chain.getBalance({
                address: daveAddress.toString()
            }))!
        ).to.equal(0);
    });

    it("Mining a block with 1 transaction", async function() {
        await node.sendPayTx({ fee: 10 });

        expect(
            +(await node.rpc.chain.getBalance({
                address: faucetAddress.toString()
            }))!
        ).to.equal(FAUCET_INITIAL_CCS - 10 /* fee */);
        expect(
            +(await node.rpc.chain.getBalance({
                address: aliceAddress.toString()
            }))!
        ).to.equal(4 /* share */ + BLOCK_REWARD);
        expect(
            +(await node.rpc.chain.getBalance({
                address: bobAddress.toString()
            }))!
        ).to.deep.equal(3 /* share */);
        expect(
            +(await node.rpc.chain.getBalance({
                address: carolAddress.toString()
            }))!
        ).to.deep.equal(2 /* share */);
        expect(
            +(await node.rpc.chain.getBalance({
                address: daveAddress.toString()
            }))!
        ).to.deep.equal(1 /* share */);
    });

    it("Mining a block with 3 transactions", async function() {
        await node.rpc.devel!.stopSealing();
        await node.sendPayTx({
            fee: 10,
            seq: 0
        });
        await node.sendPayTx({
            fee: 10,
            seq: 1
        });
        await node.sendPayTx({
            fee: 15,
            seq: 2
        });
        await node.rpc.devel!.startSealing();

        const TOTAL_FEE = 10 + 10 + 15;
        const TOTAL_MIN_FEE = MIN_FEE_PAY * 3;
        expect(
            +(await node.rpc.chain.getBalance({
                address: faucetAddress.toString()
            }))!
        ).to.deep.equal(FAUCET_INITIAL_CCS - TOTAL_FEE);
        expect(
            +(await node.rpc.chain.getBalance({
                address: aliceAddress.toString()
            }))!
        ).to.equal(
            Number(Math.floor((TOTAL_MIN_FEE * 4) / 10) /* share */) +
            TOTAL_FEE - // block author get the remaining fee
                Math.floor((TOTAL_MIN_FEE * 4) / 10) -
                Math.floor((TOTAL_MIN_FEE * 3) / 10) -
                Math.floor((TOTAL_MIN_FEE * 2) / 10) -
                Math.floor((TOTAL_MIN_FEE * 1) / 10) +
                BLOCK_REWARD
        );
        expect(
            +(await node.rpc.chain.getBalance({
                address: bobAddress.toString()
            }))!
        ).to.deep.equal(
            Number(Math.floor((TOTAL_MIN_FEE * 3) / 10) /* share */)
        );
        expect(
            +(await node.rpc.chain.getBalance({
                address: carolAddress.toString()
            }))!
        ).to.deep.equal(
            Number(Math.floor((TOTAL_MIN_FEE * 2) / 10) /* share */)
        );
        expect(
            +(await node.rpc.chain.getBalance({
                address: daveAddress.toString()
            }))!
        ).to.deep.equal(
            Number(Math.floor((TOTAL_MIN_FEE * 1) / 10) /* share */)
        );
    });

    it("Mining a block with a transaction that pays the author", async function() {
        await node.pay(aliceAddress, 100);
        expect(
            +(await node.rpc.chain.getBalance({
                address: faucetAddress.toString()
            }))!
        ).to.equal(FAUCET_INITIAL_CCS + 100 /* pay */ - 10 /* fee */);
        expect(
            +(await node.rpc.chain.getBalance({
                address: aliceAddress.toString()
            }))!
        ).to.equal(
            Number(
                100 /* pay */ +
                Math.floor((10 * 4) / 10) /* share */ +
                    BLOCK_REWARD
            )
        );
        expect(
            +(await node.rpc.chain.getBalance({
                address: bobAddress.toString()
            }))!
        ).to.equal(Number(Math.floor((10 * 3) / 10) /* share */));
        expect(
            +(await node.rpc.chain.getBalance({
                address: carolAddress.toString()
            }))!
        ).to.equal(Number(Math.floor((10 * 2) / 10) /* share */));
        expect(
            +(await node.rpc.chain.getBalance({
                address: daveAddress.toString()
            }))!
        ).to.equal(Number(Math.floor((10 * 1) / 10) /* share */));
    });

    it("Mining a block with a transaction which author pays someone in", async function() {
        await node.sendPayTx({ fee: 10 });
        const faucetBalance = +(await node.rpc.chain.getBalance({
            address: faucetAddress.toString()
        }))!;
        const aliceBalance = +(await node.rpc.chain.getBalance({
            address: aliceAddress.toString()
        }))!;
        const bobBalance = +(await node.rpc.chain.getBalance({
            address: bobAddress.toString()
        }))!;
        const carolBalance = +(await node.rpc.chain.getBalance({
            address: carolAddress.toString()
        }))!;
        const daveBalance = +(await node.rpc.chain.getBalance({
            address: daveAddress.toString()
        }))!;

        expect(faucetBalance).to.equal(FAUCET_INITIAL_CCS - 10 /* fee */);
        expect(aliceBalance).to.equal(
            Number(Math.floor((10 * 4) / 10)) /* share */ + BLOCK_REWARD
        );
        expect(bobBalance).to.equal(
            Number(Math.floor((10 * 3) / 10) /* share */)
        );
        expect(carolBalance).to.equal(
            Number(Math.floor((10 * 2) / 10) /* share */)
        );
        expect(daveBalance).to.equal(
            Number(Math.floor((10 * 1) / 10) /* share */)
        );

        const tx = await node.sdk.core
            .createPayTransaction({
                recipient: faucetAddress,
                quantity: 20
            })
            .sign({ secret: aliceSecret, seq: 0, fee: 10 });
        await node.sdk.rpc.chain.sendSignedTransaction(tx);

        expect(
            +(await node.rpc.chain.getBalance({
                address: faucetAddress.toString()
            }))!
        ).to.deep.equal(faucetBalance + 20 /* pay */);
        expect(
            +(await node.rpc.chain.getBalance({
                address: aliceAddress.toString()
            }))!
        ).to.deep.equal(
            aliceBalance -
            20 /* pay */ -
            10 /* fee */ +
            Math.floor((10 * 4) / 10) /* share */ +
                BLOCK_REWARD
        );
        expect(
            +(await node.rpc.chain.getBalance({
                address: bobAddress.toString()
            }))!
        ).to.deep.equal(Number(bobBalance) + Math.floor((10 * 3) / 10));
        expect(
            +(await node.rpc.chain.getBalance({
                address: carolAddress.toString()
            }))!
        ).to.deep.equal(Number(carolBalance) + Math.floor((10 * 2) / 10));
        expect(
            +(await node.rpc.chain.getBalance({
                address: daveAddress.toString()
            }))!
        ).to.deep.equal(Number(daveBalance) + Math.floor((10 * 1) / 10));
    });

    afterEach(async function() {
        if (this.currentTest!.state === "failed") {
            node.keepLogs();
        }
        await node.clean();
    });
});
