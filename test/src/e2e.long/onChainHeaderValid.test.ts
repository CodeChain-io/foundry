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

import * as chai from "chai";
import { expect } from "chai";
import "mocha";
import * as RLP from "rlp";
import { similar } from "../helper/chai-similar";
import {
    validator0Address,
    validator1Address,
    validator2Address,
    validator3Address
} from "../helper/constants";
import { Mock } from "../helper/mock";
import { Header } from "../helper/mock/cHeader";
import { PromiseExpect } from "../helper/promise";
import CodeChain from "../helper/spawn";
import { Address, H256, U256 } from "../primitives/src";

chai.use(similar);

const BLAKE_NULL_RLP: H256 = new H256(
    "45b0cfc220ceec5b7c1c62c4d4193d38e4eba48e8815729ce75f9c0ab0e4c1c0"
);
describe("Test onChain header communication", async function() {
    const promiseExpect = new PromiseExpect();
    let nodeA: CodeChain;
    let mock: Mock;
    let genesisHeader: Header;
    let header1: Header;
    let block1Validator: any;
    let header2: Header;
    let block2Validator: any;
    let header3: Header;
    let nodes: CodeChain[];

    before(async function() {
        this.timeout(60_000);

        const validatorAddresses = [
            validator0Address,
            validator1Address,
            validator2Address,
            validator3Address
        ];
        nodes = validatorAddresses.map(address => {
            return new CodeChain({
                chain: `${__dirname}/../scheme/tendermint-int.json`,
                argv: [
                    "--engine-signer",
                    address.toString(),
                    "--password-path",
                    "test/tendermint/password.json",
                    "--force-sealing",
                    "--no-discovery"
                ],
                additionalKeysPath: "tendermint/keys"
            });
        });
        await Promise.all(nodes.map(node => node.start()));

        await promiseExpect.shouldFulfill(
            "connect",
            Promise.all([
                nodes[0].connect(nodes[1]),
                nodes[0].connect(nodes[2]),
                nodes[0].connect(nodes[3]),
                nodes[1].connect(nodes[2]),
                nodes[1].connect(nodes[3]),
                nodes[2].connect(nodes[3])
            ])
        );

        await promiseExpect.shouldFulfill(
            "block generation",
            Promise.all([
                nodes[0].waitBlockNumber(3),
                nodes[1].waitBlockNumber(3),
                nodes[2].waitBlockNumber(3),
                nodes[3].waitBlockNumber(3)
            ])
        );

        const rpc = nodes[0].rpc;

        const genesisBlock = await rpc.chain.getBlockByNumber({
            blockNumber: 0
        });
        if (genesisBlock == null) {
            throw Error("Cannot get the genesis block");
        }
        const genesisSeal = genesisBlock.seal.map(intArray =>
            RLP.decode(new Buffer(intArray))
        );

        const block1 = await rpc.chain.getBlockByNumber({ blockNumber: 1 });
        if (block1 == null) {
            throw Error("Cannot get the first block");
        }
        const block1Seal = block1.seal.map(intArray =>
            RLP.decode(new Buffer(intArray))
        );
        block1Validator = (
            await rpc.call({ method: "chain_getValidatorSet" }, 1)
        ).result;

        const block2 = await rpc.chain.getBlockByNumber({ blockNumber: 2 });
        if (block2 == null) {
            throw Error("Cannot get the second block");
        }
        const block2Seal = block2.seal.map(intArray =>
            RLP.decode(new Buffer(intArray))
        );
        block2Validator = (
            await rpc.call({ method: "chain_getValidatorSet" }, 2)
        ).result;

        const block3 = await rpc.chain.getBlockByNumber({ blockNumber: 3 });
        if (block3 == null) {
            throw Error("Cannot get the second block");
        }
        const block3Seal = block3.seal.map(intArray =>
            RLP.decode(new Buffer(intArray))
        );

        await promiseExpect.shouldFulfill(
            "stop",
            Promise.all([
                nodes[0].clean(),
                nodes[1].clean(),
                nodes[2].clean(),
                nodes[3].clean()
            ])
        );

        const author1PlatformAddr = Address.fromString(genesisBlock.author);
        genesisHeader = new Header(
            new H256(genesisBlock.parentHash),
            new U256(genesisBlock.timestamp),
            new U256(genesisBlock.number),
            author1PlatformAddr.pubkey,
            [], // lastCommittedValidators
            Buffer.from(genesisBlock.extraData),
            BLAKE_NULL_RLP,
            new H256(genesisBlock.transactionsRoot),
            new H256(genesisBlock.stateRoot),
            new H256(genesisBlock.nextValidatorSetHash),
            genesisSeal
        );
        const author2PlatformAddr = Address.fromString(block1.author);
        header1 = new Header(
            genesisHeader.hashing(),
            new U256(block1.timestamp),
            new U256(block1.number),
            author2PlatformAddr.pubkey,
            [], // lastCommittedValidators
            Buffer.from(block1.extraData),
            BLAKE_NULL_RLP,
            new H256(block1.transactionsRoot),
            new H256(block1.stateRoot),
            new H256(block1.nextValidatorSetHash),
            block1Seal
        );
        const author3PlatformAddr = Address.fromString(block2.author);
        header2 = new Header(
            header1.hashing(),
            new U256(block2.timestamp),
            new U256(block2.number),
            author3PlatformAddr.pubkey,
            [], // lastCommittedValidators
            Buffer.from(block2.extraData),
            BLAKE_NULL_RLP,
            new H256(block2.transactionsRoot),
            new H256(block2.stateRoot),
            new H256(block2.nextValidatorSetHash),
            block2Seal
        );
        const author4PlatformAddr = Address.fromString(block3.author);
        header3 = new Header(
            header2.hashing(),
            new U256(block3.timestamp),
            new U256(block3.number),
            author4PlatformAddr.pubkey,
            [], // lastCommittedValidators
            Buffer.from(block3.extraData),
            BLAKE_NULL_RLP,
            new H256(block3.transactionsRoot),
            new H256(block3.stateRoot),
            new H256(block3.nextValidatorSetHash),
            block3Seal
        );

        nodeA = new CodeChain({
            chain: `${__dirname}/../scheme/tendermint-int.json`
        });
        await nodeA.start();
        mock = new Mock("0.0.0.0", nodeA.port, "tc");
        await mock.establish(header2.hashing());
    });

    it("OnChain valid header propagation test", async function() {
        const headerRequest = mock.getBlockHeaderRequest();
        expect(headerRequest!.startNumber).is.similarTo(new U256(0));

        await mock.sendBlockHeaderResponse([
            [genesisHeader.toEncodeObject(), []],
            [header1.toEncodeObject(), []],
            [
                header2.toEncodeObject(),
                validatorToEncodeObject(block1Validator)
            ],
            [header3.toEncodeObject(), validatorToEncodeObject(block2Validator)]
        ]);
        await mock.waitBodyRequest();
        const bodyRequest = mock.getBlockBodyRequest();
        expect(bodyRequest!.data[0]).is.similarTo(header1.hashing());
        expect(bodyRequest!.data[1]).is.similarTo(header2.hashing());
    }).timeout(50_000);

    afterEach(async function() {
        promiseExpect.checkFulfilled();
        if (this.currentTest!.state === "failed") {
            nodeA.keepLogs();
        }
        await mock.end();
        await nodeA.clean();
    });

    function validatorToEncodeObject(validatorSet: any) {
        const result = [];
        for (const entry of validatorSet) {
            result.push(entry.publicKey);
            result.push(entry.delegation);
        }
        return [result];
    }
});
