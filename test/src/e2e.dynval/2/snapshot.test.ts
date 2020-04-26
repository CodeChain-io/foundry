// Copyright 2019 Kodebox, Inc.
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

import * as chai from "chai";
import { expect } from "chai";
import * as chaiAsPromised from "chai-as-promised";
import RPC from "foundry-rpc";
import * as fs from "fs";
import "mocha";
import * as path from "path";
import * as stake from "../../stakeholder";
import * as mkdirp from "mkdirp";
import { validators } from "../../../tendermint.dynval/constants";
import { faucetAddress, faucetSecret } from "../../helper/constants";
import { PromiseExpect } from "../../helper/promise";
import CodeChain, { Signer } from "../../helper/spawn";
import { setTermTestTimeout, withNodes } from "../setup";
import { H256 } from "../../primitives/src";

chai.use(chaiAsPromised);

const SNAPSHOT_CONFIG = `${__dirname}/../../../tendermint.dynval/snapshot-config.yml`;
const SNAPSHOT_PATH = `${__dirname}/../../../../snapshot/`;

describe("Snapshot for Tendermint with Dynamic Validator", function() {
    const promiseExpect = new PromiseExpect();
    const snapshotValidators = validators.slice(0, 3);
    const freshNodeValidator = validators[3];
    const { nodes } = withNodes(this, {
        promiseExpect,
        overrideParams: {
            maxNumOfValidators: 3
        },
        validators: snapshotValidators.map((signer, index) => ({
            signer,
            delegation: 5000,
            deposit: 10_000_000 - index // tie-breaker
        })),
        modify: () => {
            mkdirp.sync(SNAPSHOT_PATH);
            const snapshotPath = fs.mkdtempSync(SNAPSHOT_PATH);
            return {
                additionalArgv: [
                    "--snapshot-path",
                    snapshotPath,
                    "--config",
                    SNAPSHOT_CONFIG
                ],
                nodeAdditionalProperties: {
                    snapshotPath
                }
            };
        }
    });

    it("should be exist after some time", async function() {
        const termWaiter = setTermTestTimeout(this, {
            terms: 2
        });
        const termMetadata = await termWaiter.waitNodeUntilTerm(nodes[0], {
            target: 2,
            termPeriods: 1
        });
        const snapshotBlock = await getSnapshotBlock(nodes[0], termMetadata);
        expect(
            path.join(
                nodes[0].snapshotPath,
                remove_0x_prefix(snapshotBlock.hash),
                remove_0x_prefix(snapshotBlock.stateRoot)
            )
        ).to.satisfy(fs.existsSync);
    });

    it.skip("should be able to boot with the snapshot", async function() {
        const termWaiter = setTermTestTimeout(this, {
            terms: 3
        });
        const termMetadata1 = await termWaiter.waitNodeUntilTerm(nodes[0], {
            target: 2,
            termPeriods: 1
        });
        const snapshotBlock = await getSnapshotBlock(nodes[0], termMetadata1);
        await makeItValidator(nodes[0], freshNodeValidator);
        const snapshotPath = fs.mkdtempSync(SNAPSHOT_PATH);
        const node = new CodeChain({
            chain: `${__dirname}/../../scheme/tendermint-dynval.json`,
            argv: [
                "--engine-signer",
                freshNodeValidator.address.toString(),
                "--password-path",
                `test/tendermint.dynval/${freshNodeValidator.address.value}/password.json`,
                "--force-sealing",
                "--snapshot-path",
                snapshotPath,
                "--config",
                SNAPSHOT_CONFIG,
                "--snapshot-hash",
                remove_0x_prefix(snapshotBlock.hash),
                "--snapshot-number",
                snapshotBlock.number.toString()
            ],
            additionalKeysPath: `tendermint.dynval/${freshNodeValidator.address.value}/keys`
        });
        try {
            await node.start();
            await node.connect(nodes[0]);
            await termWaiter.waitNodeUntilTerm(node, {
                target: 4,
                termPeriods: 2
            });

            await freshValidatorCheck(nodes[0].rpc);

            expect(
                await node.rpc.chain.getBlockByNumber({
                    blockNumber: snapshotBlock.number - 1
                })
            ).to.be.null;
            expect(
                await node.rpc.chain.getBlockByNumber({
                    blockNumber: snapshotBlock.number
                })
            ).not.to.be.null;
            // Check that the freshNodeValidator is still a validator & make sure it doesn't have a block/header before termMetadata1.
        } catch (e) {
            node.keepLogs();
            throw e;
        } finally {
            await node.clean();
        }
    });

    afterEach(async function() {
        promiseExpect.checkFulfilled();
    });

    async function freshValidatorCheck(rpc: RPC) {
        const blockNumber = await rpc.chain.getBestBlockNumber();
        const termMedata = await stake.getTermMetadata(rpc, blockNumber);
        const currentTermInitialBlockNumber =
            termMedata!.lastTermFinishedBlockNumber + 1;
        const validatorsAfter = (await stake.getPossibleAuthors(
            rpc,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(validatorsAfter).and.contains(
            freshNodeValidator.address.toString()
        );
    }
});

async function getSnapshotBlock(
    node: CodeChain,
    termMetadata: stake.TermMetadata
) {
    const blockNumber = termMetadata.lastTermFinishedBlockNumber + 1;
    await node.waitBlockNumber(blockNumber);
    return (await node.rpc.chain.getBlockByNumber({ blockNumber }))!;
}

async function makeItValidator(node: CodeChain, freshNodeValidator: Signer) {
    const faucetSeq = (await node.rpc.chain.getSeq({
        address: faucetAddress.toString()
    }))!;
    const payTx = node.testFramework.core
        .createPayTransaction({
            recipient: freshNodeValidator.address,
            quantity: 200000000
        })
        .sign({
            secret: faucetSecret,
            seq: faucetSeq,
            fee: 10
        });
    await node.waitForTx(
        new H256(
            await node.rpc.mempool.sendSignedTransaction({
                tx: payTx.rlpBytes().toString("hex")
            })
        )
    );
    const selfNominateTx = node.testFramework.core
        .createSelfNominateTransaction({ deposit: 10000000, metadata: "" })
        .sign({
            secret: freshNodeValidator.privateKey,
            seq: (await node.rpc.chain.getSeq({
                address: freshNodeValidator.address.toString()
            }))!,
            fee: 10
        });
    await node.waitForTx(
        new H256(
            await node.rpc.mempool.sendSignedTransaction({
                tx: selfNominateTx.rlpBytes().toString("hex")
            })
        )
    );
    const delegateTx = node.testFramework.core
        .createDelegateCCSTransaction({
            delegatee: freshNodeValidator.address,
            quantity: 10000
        })
        .sign({
            secret: faucetSecret,
            seq: faucetSeq + 1,
            fee: 10
        });
    await node.waitForTx(
        new H256(
            await node.rpc.mempool.sendSignedTransaction({
                tx: delegateTx.rlpBytes().toString("hex")
            })
        )
    );
}

const remove_0x_prefix = (target: string): string => {
    if (target.startsWith("0x")) {
        return target.slice(2);
    } else {
        return target;
    }
};
