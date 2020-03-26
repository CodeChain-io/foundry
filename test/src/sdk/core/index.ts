import {
    Address,
    AddressValue,
    H128,
    H160,
    H256,
    H256Value,
    H512,
    U256,
    U64,
    U64Value
} from "foundry-primitives";

import { Block } from "./Block";
import { Script } from "./Script";
import { SignedTransaction } from "./SignedTransaction";
import { Transaction } from "./Transaction";
import { Custom } from "./transaction/Custom";
import { Pay } from "./transaction/Pay";
import { NetworkId } from "./types";

export class Core {
    public static classes = {
        // Data
        H128,
        H160,
        H256,
        H512,
        U256,
        U64,
        // Block
        Block,
        // Transaction
        Transaction,
        SignedTransaction,
        // Transaction
        Pay,
        Custom,
        // Script
        Script,
        Address
    };

    public classes = Core.classes;
    private networkId: NetworkId;

    /**
     * @param params.networkId The network id of CodeChain.
     */
    constructor(params: { networkId: NetworkId }) {
        const { networkId } = params;
        this.networkId = networkId;
    }

    /**
     * Creates Pay type which pays the value quantity of CCC(CodeChain Coin)
     * from the tx signer to the recipient. Who is signing the tx will pay.
     * @param params.recipient The platform account who receives CCC
     * @param params.quantity quantity of CCC to pay
     * @throws Given string for recipient is invalid for converting it to address
     * @throws Given number or string for quantity is invalid for converting it to U64
     */
    public createPayTransaction(params: {
        recipient: AddressValue;
        quantity: U64Value;
    }): Pay {
        const { recipient, quantity } = params;
        checkAddressRecipient(recipient);
        checkAmount(quantity);
        return new Pay(
            Address.ensure(recipient),
            U64.ensure(quantity),
            this.networkId
        );
    }

    /**
     * Creates Custom type that will be handled by a specified type handler
     * @param params.handlerId An Id of an type handler which will handle a custom transaction
     * @param params.bytes A custom transaction body
     * @throws Given number for handlerId is invalid for converting it to U64
     */
    public createCustomTransaction(params: {
        handlerId: number;
        bytes: Buffer;
    }): Custom {
        const { handlerId, bytes } = params;
        checkHandlerId(handlerId);
        checkBytes(bytes);
        const customParam = {
            handlerId: U64.ensure(handlerId),
            bytes
        };
        return new Custom(customParam, this.networkId);
    }
}

function checkAddressRecipient(recipient: AddressValue) {
    if (!Address.check(recipient)) {
        throw Error(
            `Expected recipient param to be a address but found ${recipient}`
        );
    }
}

function checkAmount(amount: U64Value) {
    if (!U64.check(amount)) {
        throw Error(
            `Expected amount param to be a U64 value but found ${amount}`
        );
    }
}

function checkTransactionHash(value: H256Value) {
    if (!H256.check(value)) {
        throw Error(
            `Expected hash param to be an H256 value but found ${value}`
        );
    }
}

function checkSecret(value: H256Value) {
    if (!H256.check(value)) {
        throw Error(
            `Expected secret param to be an H256 value but found ${value}`
        );
    }
}

function checkSignature(signature: string) {
    // Ed25519 Signature
    if (
        typeof signature !== "string" ||
        !/^(0x)?[0-9a-fA-F]{130}$/.test(signature)
    ) {
        throw Error(
            `Expected signature param to be a 65 byte hexstring but found ${signature}`
        );
    }
}

function checkHandlerId(handlerId: number) {
    if (
        typeof handlerId !== "number" ||
        !Number.isInteger(handlerId) ||
        handlerId < 0
    ) {
        throw Error(
            `Expected handlerId param to be a non-negative number value but found ${handlerId}`
        );
    }
}

function checkBytes(bytes: Buffer) {
    if (!(bytes instanceof Buffer)) {
        throw Error(
            `Expected bytes param to be an instance of Buffer but found ${bytes}`
        );
    }
}
