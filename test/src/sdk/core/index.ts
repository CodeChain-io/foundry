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
import { SignedTransaction } from "./SignedTransaction";
import { Transaction } from "./Transaction";
import { ChangeParams } from "./transaction/ChangeParams";
import { DelegateCCS } from "./transaction/DelegateCCS";
import { Pay } from "./transaction/Pay";
import { Redelegate } from "./transaction/Redelegate";
import { Remove } from "./transaction/Remove";
import { ReportDoubleVote } from "./transaction/ReportDoubleVote";
import { Revoke } from "./transaction/Revoke";
import { SelfNominate } from "./transaction/SelfNominate";
import { SetShardOwners } from "./transaction/SetShardOwners";
import { Store } from "./transaction/Store";
import { TransferCCS } from "./transaction/TransferCCS";
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
        SetShardOwners,
        Store,
        Remove,
        DelegateCCS,
        TransferCCS,
        Revoke,
        Redelegate,
        SelfNominate,
        ReportDoubleVote,
        // Script
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

    public createSetShardOwnersTransaction(params: {
        shardId: number;
        owners: Array<AddressValue>;
    }): SetShardOwners {
        const { shardId, owners } = params;
        checkShardId(shardId);
        checkOwners(owners);
        return new SetShardOwners(
            {
                shardId,
                owners: owners.map(Address.ensure)
            },
            this.networkId
        );
    }

    /**
     * Creates Store type which store content with certifier on chain.
     * @param params.content Content to store
     * @param params.secret Secret key to sign
     * @param params.certifier Certifier of the text, which is address
     * @param params.signature Signature on the content by the certifier
     * @throws Given string for secret is invalid for converting it to H256
     */
    public createStoreTransaction(
        params:
            | {
                  content: string;
                  certifier: AddressValue;
                  signature: string;
              }
            | {
                  content: string;
                  secret: H256Value;
              }
    ): Store {
        let storeParams;
        if ("secret" in params) {
            const { content, secret } = params;
            checkSecret(secret);
            storeParams = {
                content,
                secret: H256.ensure(secret)
            };
        } else {
            const { content, certifier, signature } = params;
            checkCertifier(certifier);
            checkSignature(signature);
            storeParams = {
                content,
                certifier: Address.ensure(certifier),
                signature
            };
        }
        return new Store(storeParams, this.networkId);
    }

    /**
     * Creates Remove type which remove the text from the chain.
     * @param params.hash Transaction hash which stored the text
     * @param params.secret Secret key to sign
     * @param params.signature Signature on tx hash by the certifier of the text
     * @throws Given string for hash or secret is invalid for converting it to H256
     */
    public createRemoveTransaction(
        params:
            | {
                  hash: H256Value;
                  secret: H256Value;
              }
            | {
                  hash: H256Value;
                  signature: string;
              }
    ): Remove {
        let removeParam = null;
        if ("secret" in params) {
            const { hash, secret } = params;
            checkTransactionHash(hash);
            checkSecret(secret);
            removeParam = {
                hash: H256.ensure(hash),
                secret: H256.ensure(secret)
            };
        } else {
            const { hash, signature } = params;
            checkTransactionHash(hash);
            checkSignature(signature);
            removeParam = {
                hash: H256.ensure(hash),
                signature
            };
        }
        return new Remove(removeParam, this.networkId);
    }

    public createDelegateCCSTransaction(params: {
        delegatee: AddressValue;
        quantity: U64Value;
    }): DelegateCCS {
        const { delegatee, quantity } = params;
        checkAddressRecipient(delegatee);
        checkAmount(quantity);
        return new DelegateCCS(
            Address.ensure(delegatee),
            U64.ensure(quantity),
            this.networkId
        );
    }

    public createTransferCCSTransaction(params: {
        recipient: AddressValue;
        quantity: U64Value;
    }): TransferCCS {
        const { recipient, quantity } = params;
        checkAddressRecipient(recipient);
        checkAmount(quantity);
        return new TransferCCS(
            Address.ensure(recipient),
            U64.ensure(quantity),
            this.networkId
        );
    }

    public createRevokeTransaction(params: {
        delegatee: AddressValue;
        quantity: U64Value;
    }): Revoke {
        const { delegatee, quantity } = params;
        checkAddressRecipient(delegatee);
        checkAmount(quantity);
        return new Revoke(
            Address.ensure(delegatee),
            U64.ensure(quantity),
            this.networkId
        );
    }

    public createRedelegateTransaction(params: {
        prevDelegator: AddressValue;
        nextDelegator: AddressValue;
        quantity: U64Value;
    }): Redelegate {
        const { prevDelegator, nextDelegator, quantity } = params;
        checkAddressRecipient(prevDelegator);
        checkAddressRecipient(nextDelegator);
        checkAmount(quantity);
        return new Redelegate(
            Address.ensure(prevDelegator),
            Address.ensure(nextDelegator),
            U64.ensure(quantity),
            this.networkId
        );
    }

    public createSelfNominateTransaction(params: {
        deposit: U64Value;
        metadata: Buffer | string;
    }): SelfNominate {
        const { deposit } = params;
        const metadata = Buffer.from(params.metadata);
        checkAmount(deposit);
        checkBytes(metadata);
        return new SelfNominate(U64.ensure(deposit), metadata, this.networkId);
    }

    public createReportDoubleVoteTransaction(params: {
        message1: Buffer | string;
        message2: Buffer | string;
    }): ReportDoubleVote {
        const message1 = Buffer.from(params.message1);
        const message2 = Buffer.from(params.message2);
        checkBytes(message1);
        checkBytes(message2);
        return new ReportDoubleVote(message1, message2, this.networkId);
    }

    public createChangeParamsTransaction(args: {
        metadataSeq: U64Value;
        params: (number | string)[];
        approvals: any[];
    }): ChangeParams {
        const { metadataSeq, params, approvals } = args;
        checkAmount(metadataSeq);
        return new ChangeParams(
            U64.ensure(metadataSeq),
            params,
            approvals,
            this.networkId
        );
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

function checkShardId(shardId: number) {
    if (
        typeof shardId !== "number" ||
        !Number.isInteger(shardId) ||
        shardId < 0 ||
        shardId > 0xffff
    ) {
        throw Error(
            `Expected shardId param to be a number but found ${shardId}`
        );
    }
}

function checkCertifier(certifier: AddressValue) {
    if (!Address.check(certifier)) {
        throw Error(
            `Expected certifier param to be a address but found ${certifier}`
        );
    }
}

function checkOwners(owners: Array<AddressValue>) {
    if (!Array.isArray(owners)) {
        throw Error(`Expected owners param to be an array but found ${owners}`);
    }
    owners.forEach((owner, index) => {
        if (!Address.check(owner)) {
            throw Error(
                `Expected an owner address to be a address value but found ${owner} at index ${index}`
            );
        }
    });
}

function checkUsers(users: Array<AddressValue>) {
    if (!Array.isArray(users)) {
        throw Error(`Expected users param to be an array but found ${users}`);
    }
    users.forEach((user, index) => {
        if (!Address.check(user)) {
            throw Error(
                `Expected a user address to be a address value but found ${user} at index ${index}`
            );
        }
    });
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
