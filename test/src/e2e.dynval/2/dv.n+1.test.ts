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

import { expect } from "chai";
import RPC from "foundry-rpc";
import "mocha";
import * as stake from "../../stakeholder";

import { validators } from "../../../tendermint.dynval/constants";
import { faucetAddress, faucetSecret } from "../../helper/constants";
import { PromiseExpect } from "../../helper/promise";
import { findNode, setTermTestTimeout, withNodes } from "../setup";
import { H256 } from "foundry-primitives/lib";

describe("Dynamic Validator N -> N+1", function() {
    const promiseExpect = new PromiseExpect();

    const initialValidators = validators.slice(0, 3);
    const betty = validators[3];

    async function beforeInsertionCheck(rpc: RPC) {
        const blockNumber = await rpc.chain.getBestBlockNumber();
        const termMedata = await stake.getTermMetadata(rpc, blockNumber);
        const currentTermInitialBlockNumber =
            termMedata!.lastTermFinishedBlockNumber + 1;
        const validatorsBefore = (await stake.getPossibleAuthors(
            rpc,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(termMedata!.currentTermId).to.be.equals(1);
        expect(validatorsBefore)
            .to.have.lengthOf(initialValidators.length)
            .and.contains.all.members(
                initialValidators.map(validator => validator.address.toString())
            );
    }

    async function bettyInsertionCheck(rpc: RPC) {
        const blockNumber = await rpc.chain.getBestBlockNumber();
        const termMedata = await stake.getTermMetadata(rpc, blockNumber);
        const currentTermInitialBlockNumber =
            termMedata!.lastTermFinishedBlockNumber + 1;
        const validatorsAfter = (await stake.getPossibleAuthors(
            rpc,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(termMedata!.currentTermId).to.be.equals(2);
        expect(validatorsAfter)
            .to.have.lengthOf(initialValidators.length + 1)
            .and.contains.all.members(
                [...initialValidators, betty].map(validator =>
                    validator.address.toString()
                )
            );
    }

    describe("Nominate a new candidate and delegate", async function() {
        const { nodes } = withNodes(this, {
            promiseExpect,
            validators: [
                ...initialValidators.map((signer, index) => ({
                    signer,
                    delegation: 5_000,
                    deposit: 10_000_000 - index // tie-breaker
                })),
                { signer: betty }
            ]
        });

        it("betty should be included in validators", async function() {
            const termWaiter = setTermTestTimeout(this, {
                terms: 1
            });

            const checkingNode = nodes[0];
            await beforeInsertionCheck(checkingNode.rpc);
            const bettyNode = findNode(nodes, betty);
            const nominateTx = stake
                .createSelfNominateTransaction(
                    bettyNode.testFramework,
                    11_000_000,
                    ""
                )
                .sign({
                    secret: betty.privateKey,
                    seq: (await bettyNode.rpc.chain.getSeq({
                        address: betty.address.toString()
                    }))!,
                    fee: 10
                });
            const nominateTxHash = await bettyNode.rpc.mempool.sendSignedTransaction(
                { tx: nominateTx.rlpBytes().toString("hex") }
            );
            const delegateTx = stake
                .createDelegateCCSTransaction(
                    bettyNode.testFramework,
                    betty.address,
                    5_000
                )
                .sign({
                    secret: faucetSecret,
                    seq: (await bettyNode.rpc.chain.getSeq({
                        address: faucetAddress.toString()
                    }))!,
                    fee: 10
                });
            const delegateTxHash = await bettyNode.rpc.mempool.sendSignedTransaction(
                { tx: delegateTx.rlpBytes().toString("hex") }
            );
            await checkingNode.waitForTx([
                new H256(nominateTxHash),
                new H256(delegateTxHash)
            ]);

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.rpc);
        });
    });

    describe("Increase one candidate's deposit which is less than the minimum deposit", async function() {
        const { nodes } = withNodes(this, {
            promiseExpect,
            validators: [
                ...initialValidators.map((signer, index) => ({
                    signer,
                    delegation: 5_000,
                    deposit: 10_000_000 - index // tie-breaker
                })),
                { signer: betty, delegation: 5_000, deposit: 9999 }
            ]
        });

        it("betty should be included in validators", async function() {
            const termWaiter = setTermTestTimeout(this, {
                terms: 1
            });

            const checkingNode = nodes[0];
            await beforeInsertionCheck(checkingNode.rpc);
            const nominateTx = stake
                .createSelfNominateTransaction(
                    checkingNode.testFramework,
                    10_000,
                    ""
                )
                .sign({
                    secret: betty.privateKey,
                    seq: (await checkingNode.rpc.chain.getSeq({
                        address: betty.address.toString()
                    }))!,
                    fee: 10
                });
            const nominateTxHash = await checkingNode.rpc.mempool.sendSignedTransaction(
                { tx: nominateTx.rlpBytes().toString("hex") }
            );
            await checkingNode.waitForTx(new H256(nominateTxHash));

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.rpc);
        });
    });

    describe("Delegate more stake to whose stake is less than the minimum delegation", async function() {
        const { nodes } = withNodes(this, {
            promiseExpect,
            validators: [
                ...initialValidators.map((signer, index) => ({
                    signer,
                    delegation: 5_000,
                    deposit: 10_000_000 - index // tie-breaker
                })),
                { signer: betty, delegation: 999, deposit: 10_000_000 }
            ]
        });

        it("betty should be included in validators", async function() {
            const termWaiter = setTermTestTimeout(this, {
                terms: 1
            });

            const checkingNode = nodes[0];
            await beforeInsertionCheck(checkingNode.rpc);
            const faucetSeq = (await checkingNode.rpc.chain.getSeq({
                address: faucetAddress.toString()
            }))!;
            const delegateTx = stake
                .createDelegateCCSTransaction(
                    checkingNode.testFramework,
                    betty.address,
                    2
                )
                .sign({
                    secret: faucetSecret,
                    seq: faucetSeq,
                    fee: 10
                });
            const delegateTxHash = await checkingNode.rpc.mempool.sendSignedTransaction(
                { tx: delegateTx.rlpBytes().toString("hex") }
            );
            await checkingNode.waitForTx(new H256(delegateTxHash));

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.rpc);
        });
    });

    afterEach(async function() {
        await promiseExpect.checkFulfilled();
    });
});
