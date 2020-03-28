// Copyright 2019-2020 Kodebox, Inc.
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
import "mocha";
import * as stake from "../../stakeholder";

import { H256 } from "foundry-primitives/lib";
import { validators } from "../../../tendermint.dynval/constants";
import { faucetAddress, faucetSecret } from "../../helper/constants";
import { PromiseExpect } from "../../helper/promise";
import { changeParams, setTermTestTimeout, withNodes } from "../setup";
import { ERROR } from "../../helper/error";

chai.use(chaiAsPromised);

describe("Change commonParams that affects validator set", function() {
    const promiseExpect = new PromiseExpect();
    const { nodes, initialParams } = withNodes(this, {
        promiseExpect,
        overrideParams: {
            minNumOfValidators: 3,
            maxNumOfValidators: 5
        },
        validators: validators.slice(0, 5).map((signer, index) => ({
            signer,
            delegation: 5_000,
            deposit: 10_000_000 - index // tie-breaker
        }))
    });

    async function checkValidators(rpc: RPC, term: number, target: string[]) {
        const blockNumber = await rpc.chain.getBestBlockNumber();
        const termMetadata = (await stake.getTermMetadata(rpc, blockNumber))!;
        const currentTermInitialBlockNumber =
            termMetadata.lastTermFinishedBlockNumber + 1;
        const validatorsAfter = (await stake.getPossibleAuthors(
            rpc,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(termMetadata.currentTermId).to.be.equals(term);
        expect(validatorsAfter).to.have.lengthOf(target.length);
        expect(validatorsAfter).contains.all.members(target);
    }

    describe("Change the minimum number of validators", async function() {
        it("Some nodes who have deposit less than delegation threshold should remain as validators", async function() {
            // revoke delegations of alice, betty and charlie but we increased minNumOfValidators to 4,
            // Because alice and betty have more nomination deposit compared to the others, they should remain as validators.
            const termWaiter = setTermTestTimeout(this, {
                terms: 1
            });

            const checkingNode = nodes[0];
            const changeTxHash = await changeParams(checkingNode, 1, {
                ...initialParams,
                minNumOfValidators: 4
            });

            await checkingNode.waitForTx(changeTxHash);

            const faucetSeq = (await checkingNode.rpc.chain.getSeq({
                address: faucetAddress.toString()
            }))!;

            const revoked = validators.slice(0, 3);
            const untouched = validators.slice(3, 5);
            const revokeTxs = revoked.map((signer, idx) =>
                checkingNode.testFramework.core
                    .createRevokeTransaction({
                        delegatee: signer.address,
                        quantity: 4_999
                    })
                    .sign({
                        secret: faucetSecret,
                        seq: faucetSeq + idx,
                        fee: 10
                    })
            );

            const revokeTxHashes = await Promise.all(
                revokeTxs.map(tx =>
                    checkingNode.rpc.mempool.sendSignedTransaction({
                        tx: `${tx.rlpBytes().toString("hex")}`
                    })
                )
            );
            await checkingNode.waitForTx(new H256(revokeTxHashes[0]));
            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });

            const expectedValidators = [
                ...revoked.slice(0, 2),
                ...untouched
            ].map(signer => signer.address.toString());
            await checkValidators(checkingNode.rpc, 2, expectedValidators);
        });
    });

    describe("Change the maximum number of validators", async function() {
        it("Should select only MAX_NUM_OF_VALIDATORS validators", async function() {
            const termWaiter = setTermTestTimeout(this, {
                terms: 2
            });

            const checkingNode = nodes[0];

            await checkValidators(
                checkingNode.rpc,
                1,
                validators.slice(0, 5).map(val => val.address.toString())
            );

            const param1 = {
                ...initialParams,
                maxNumOfValidators: 3
            };
            const decreaseHash = await changeParams(checkingNode, 1, param1);
            await checkingNode.waitForTx(decreaseHash);
            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await checkValidators(
                checkingNode.rpc,
                2,
                validators.slice(0, 3).map(val => val.address.toString())
            );

            const param2 = {
                ...param1,
                maxNumOfValidators: 4
            };
            const increaseHash = await changeParams(checkingNode, 2, param2);
            await checkingNode.waitForTx(increaseHash);
            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 3,
                termPeriods: 1
            });
            await checkValidators(
                checkingNode.rpc,
                3,
                validators.slice(0, 4).map(val => val.address.toString())
            );
        });
    });

    afterEach(function() {
        promiseExpect.checkFulfilled();
    });
});

describe("Change commonParams that doesn't affects validator set", function() {
    const promiseExpect = new PromiseExpect();
    const { nodes, initialParams } = withNodes(this, {
        promiseExpect,
        overrideParams: {
            termSeconds: 10,
            maxCandidateMetadataSize: 128
        },
        validators: validators.slice(0, 3).map((signer, index) => ({
            signer,
            delegation: 5_000,
            deposit: 10_000_000 - index // tie-breaker
        }))
    });

    describe("Change term seconds", async function() {
        it("should be applied after a term seconds", async function() {
            const initialTermSeconds = initialParams.termSeconds;
            const newTermSeconds = 5;
            const margin = 1.3;

            this.slow((initialTermSeconds + newTermSeconds) * 1000 * margin);
            this.timeout((initialTermSeconds + newTermSeconds) * 1000 * 2);

            const term1Metadata = (await stake.getTermMetadata(nodes[0].rpc))!;
            expect(term1Metadata.currentTermId).to.be.equal(1);

            await nodes[0].waitForTx(
                changeParams(nodes[0], 1, {
                    ...initialParams,
                    termSeconds: newTermSeconds
                })
            );

            await nodes[0].waitForTermChange(2, initialTermSeconds * margin);

            const term2Metadata = (await stake.getTermMetadata(nodes[0].rpc))!;
            expect(term2Metadata.currentTermId).to.be.equal(2);

            await nodes[0].waitForTermChange(3, newTermSeconds * margin);

            const term3Metadata = (await stake.getTermMetadata(nodes[0].rpc))!;
            expect(term3Metadata.currentTermId).to.be.equal(3);

            const [ts1, ts2, ts3] = await Promise.all(
                [term1Metadata, term2Metadata, term3Metadata].map(m =>
                    nodes[0].rpc.chain
                        .getBlockByNumber({
                            blockNumber: m.lastTermFinishedBlockNumber
                        })
                        .then(block => block!.timestamp)
                )
            );

            // allows +- 30% error
            expect(ts2 - ts1)
                .is.approximately(initialTermSeconds, initialTermSeconds * 0.3)
                .but.not.approximately(newTermSeconds, newTermSeconds * 0.3);
            expect(ts3 - ts2)
                .is.approximately(newTermSeconds, newTermSeconds * 0.3)
                .but.not.approximately(
                    initialTermSeconds,
                    initialTermSeconds * 0.3
                );
        });
    });

    describe("Change the maximum size of candidate metadata", async function() {
        function nominationWithMetadata(size: number) {
            return nodes[0].testFramework.core.createSelfNominateTransaction({
                deposit: 1,
                metadata: " ".repeat(size)
            });
        }

        it("Should apply larger metadata limit after increment", async function() {
            this.slow(6_000);
            this.timeout(9_000);

            const alice = validators[0];
            const checkingNode = nodes[0];
            const changeTxHash = await changeParams(checkingNode, 1, {
                ...initialParams,
                maxCandidateMetadataSize: 256
            });
            await checkingNode.waitForTx(changeTxHash);
            const normalNomination = nominationWithMetadata(129);
            const seq = (await checkingNode.rpc.chain.getSeq({
                address: alice.address.toString()
            }))!;
            const normalHash = await checkingNode.rpc.mempool.sendSignedTransaction(
                {
                    tx: normalNomination
                        .sign({
                            secret: alice.privateKey,
                            seq,
                            fee: 10
                        })
                        .rlpBytes()
                        .toString("hex")
                }
            );
            await checkingNode.waitForTx(new H256(normalHash));

            const largeNomination = nominationWithMetadata(257);
            try {
                await checkingNode.rpc.mempool.sendSignedTransaction({
                    tx: largeNomination
                        .sign({
                            secret: alice.privateKey,
                            seq: seq + 1,
                            fee: 10
                        })
                        .rlpBytes()
                        .toString("hex")
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });

        it("Should apply smaller metadata limit after decrement", async function() {
            this.slow(6_000);
            this.timeout(9_000);

            const alice = validators[0];
            const checkingNode = nodes[0];
            const changeTxHash = await changeParams(checkingNode, 1, {
                ...initialParams,
                maxCandidateMetadataSize: 64
            });
            await checkingNode.waitForTx(changeTxHash);
            const normalNomination = nominationWithMetadata(63);
            const seq = (await checkingNode.rpc.chain.getSeq({
                address: alice.address.toString()
            }))!;
            const normalHash = await checkingNode.rpc.mempool.sendSignedTransaction(
                {
                    tx: normalNomination
                        .sign({
                            secret: alice.privateKey,
                            seq,
                            fee: 10
                        })
                        .rlpBytes()
                        .toString("hex")
                }
            );
            await checkingNode.waitForTx(new H256(normalHash));

            const largeNomination = nominationWithMetadata(127);
            try {
                await checkingNode.rpc.mempool.sendSignedTransaction({
                    tx: largeNomination
                        .sign({
                            secret: alice.privateKey,
                            seq: seq + 1,
                            fee: 10
                        })
                        .rlpBytes()
                        .toString("hex")
                });
                expect.fail();
            } catch (e) {
                expect(e).is.similarTo(ERROR.ACTION_DATA_HANDLER_NOT_FOUND);
            }
        });
    });

    afterEach(function() {
        promiseExpect.checkFulfilled();
    });
});
