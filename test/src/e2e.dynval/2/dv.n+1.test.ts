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
import "mocha";
import { SDK } from "../../sdk/src";
import * as stake from "../../stakeholder/src";

import { validators } from "../../../tendermint.dynval/constants";
import { faucetAddress, faucetSecret } from "../../helper/constants";
import { PromiseExpect } from "../../helper/promise";
import { findNode, setTermTestTimeout, withNodes } from "../setup";

describe("Dynamic Validator N -> N+1", function() {
    const promiseExpect = new PromiseExpect();

    const initialValidators = validators.slice(0, 3);
    const betty = validators[3];

    async function beforeInsertionCheck(sdk: SDK) {
        const blockNumber = await sdk.rpc.chain.getBestBlockNumber();
        const termMedata = await stake.getTermMetadata(sdk, blockNumber);
        const currentTermInitialBlockNumber =
            termMedata!.lastTermFinishedBlockNumber + 1;
        const validatorsBefore = (await stake.getPossibleAuthors(
            sdk,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(termMedata!.currentTermId).to.be.equals(1);
        expect(validatorsBefore)
            .to.have.lengthOf(initialValidators.length)
            .and.contains.all.members(
                initialValidators.map(validator =>
                    validator.platformAddress.toString()
                )
            );
    }

    async function bettyInsertionCheck(sdk: SDK) {
        const blockNumber = await sdk.rpc.chain.getBestBlockNumber();
        const termMedata = await stake.getTermMetadata(sdk, blockNumber);
        const currentTermInitialBlockNumber =
            termMedata!.lastTermFinishedBlockNumber + 1;
        const validatorsAfter = (await stake.getPossibleAuthors(
            sdk,
            currentTermInitialBlockNumber
        ))!.map(platformAddr => platformAddr.toString());

        expect(termMedata!.currentTermId).to.be.equals(2);
        expect(validatorsAfter)
            .to.have.lengthOf(initialValidators.length + 1)
            .and.contains.all.members(
                [...initialValidators, betty].map(validator =>
                    validator.platformAddress.toString()
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
            await beforeInsertionCheck(checkingNode.testFramework);
            const bettyNode = findNode(nodes, betty);
            const nominateTx = stake
                .createSelfNominateTransaction(
                    bettyNode.testFramework,
                    11_000_000,
                    ""
                )
                .sign({
                    secret: betty.privateKey,
                    seq: await bettyNode.testFramework.rpc.chain.getSeq(
                        betty.platformAddress
                    ),
                    fee: 10
                });
            const nominateTxHash = bettyNode.testFramework.rpc.chain.sendSignedTransaction(
                nominateTx
            );
            const delegateTx = stake
                .createDelegateCCSTransaction(
                    bettyNode.testFramework,
                    betty.platformAddress,
                    5_000
                )
                .sign({
                    secret: faucetSecret,
                    seq: await bettyNode.testFramework.rpc.chain.getSeq(
                        faucetAddress
                    ),
                    fee: 10
                });
            const delegateTxHash = bettyNode.testFramework.rpc.chain.sendSignedTransaction(
                delegateTx
            );
            await checkingNode.waitForTx([nominateTxHash, delegateTxHash]);

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.testFramework);
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
            await beforeInsertionCheck(checkingNode.testFramework);
            const nominateTx = stake
                .createSelfNominateTransaction(
                    checkingNode.testFramework,
                    10_000,
                    ""
                )
                .sign({
                    secret: betty.privateKey,
                    seq: await checkingNode.testFramework.rpc.chain.getSeq(
                        betty.platformAddress
                    ),
                    fee: 10
                });
            const nominateTxHash = checkingNode.testFramework.rpc.chain.sendSignedTransaction(
                nominateTx
            );
            await checkingNode.waitForTx(nominateTxHash);

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.testFramework);
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
            await beforeInsertionCheck(checkingNode.testFramework);
            const faucetSeq = await checkingNode.testFramework.rpc.chain.getSeq(
                faucetAddress
            );
            const delegateTx = stake
                .createDelegateCCSTransaction(
                    checkingNode.testFramework,
                    betty.platformAddress,
                    2
                )
                .sign({
                    secret: faucetSecret,
                    seq: faucetSeq,
                    fee: 10
                });
            const delegateTxHash = checkingNode.testFramework.rpc.chain.sendSignedTransaction(
                delegateTx
            );
            await checkingNode.waitForTx(delegateTxHash);

            await termWaiter.waitNodeUntilTerm(checkingNode, {
                target: 2,
                termPeriods: 1
            });
            await bettyInsertionCheck(checkingNode.testFramework);
        });
    });

    afterEach(async function() {
        await promiseExpect.checkFulfilled();
    });
});
