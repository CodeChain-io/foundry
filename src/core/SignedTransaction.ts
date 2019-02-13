import { H160, H256, H512, PlatformAddress } from "codechain-primitives";
import * as _ from "lodash";

import { blake160, blake256, recoverEcdsa } from "../utils";

import { Asset } from "./Asset";
import { Transaction } from "./Transaction";
import { NetworkId } from "./types";

const RLP = require("rlp");

/**
 * A [Transaction](tx.html) signed by a private key. It is possible to request
 * the CodeChain network to process this tx with the
 * [sendSignedTransaction](chainrpc.html#sendsignedtransaction) function.
 *
 * Transactions signed with a regular key has the same effect as those signed with
 * the original key. The original key is the key of the account that registered
 * the regular key.
 *
 * If any of the following is true, the Transaction will not be processed:
 * - The Transaction's processing fee is less than 10.
 * - A network ID is not identical.
 * - A seq is not identical to the signer's seq.
 */
export class SignedTransaction {
    public unsigned: Transaction;
    public blockNumber: number | null;
    public blockHash: H256 | null;
    public transactionIndex: number | null;
    private _signature: string;

    /**
     * @param unsigned A Transaction.
     * @param signature An ECDSA signature which is a 65 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the tx.
     * @param blockHash The hash of the block that contains the tx.
     * @param transactionIndex The index(location) of the tx within the block.
     */
    constructor(
        unsigned: Transaction,
        signature: string,
        blockNumber?: number,
        blockHash?: H256,
        transactionIndex?: number
    ) {
        this.unsigned = unsigned;
        this._signature = signature.startsWith("0x")
            ? signature.substr(2)
            : signature;
        this.blockNumber = blockNumber === undefined ? null : blockNumber;
        this.blockHash = blockHash || null;
        this.transactionIndex =
            transactionIndex === undefined ? null : transactionIndex;
    }

    /**
     * Get the signature of a tx.
     */
    public signature() {
        return this._signature;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject(): any[] {
        const { unsigned, _signature } = this;
        const result = unsigned.toEncodeObject();
        result.push(`0x${_signature}`);
        return result;
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of a tx.
     * @returns A tx hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    public getAsset(): Asset {
        // FIXME: Only UnwrapCCC has getAsset method
        return (this.unsigned as any).getAsset();
    }

    /**
     * Get the account ID of a tx's signer.
     * @returns An account ID.
     * @deprecated
     */
    public getSignerAccountId(): H160 {
        const { _signature, unsigned } = this;
        const publicKey = recoverEcdsa(unsigned.hash().value, _signature);
        return new H160(blake160(publicKey));
    }

    /**
     * Get the platform address of a tx's signer.
     * @returns A PlatformAddress.
     * @deprecated
     */
    public getSignerAddress(params: { networkId: NetworkId }): PlatformAddress {
        return PlatformAddress.fromAccountId(this.getSignerAccountId(), params);
    }

    /**
     * Get the public key of a tx's signer.
     * @returns A public key.
     */
    public getSignerPublic(): H512 {
        const { _signature, unsigned } = this;
        return new H512(recoverEcdsa(unsigned.hash().value, _signature));
    }

    /**
     * Convert to a SignedTransaction JSON object.
     * @returns A SignedTransaction JSON object.
     */
    public toJSON() {
        const {
            blockNumber,
            blockHash,
            transactionIndex,
            unsigned,
            _signature
        } = this;
        const result = unsigned.toJSON();
        result.blockNumber = blockNumber;
        result.blockHash = blockHash === null ? null : blockHash.toJSON();
        result.transactionIndex = transactionIndex;
        result.sig = `0x${_signature}`;
        result.hash = this.hash().toJSON();
        return result;
    }
}
