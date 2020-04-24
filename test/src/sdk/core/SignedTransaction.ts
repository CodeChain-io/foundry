import * as RLP from "rlp";
import { Address, H256 } from "../../primitives/src";
import { blake256 } from "../utils";
import { Transaction, TransactionJSON } from "./Transaction";
import { NetworkId } from "./types";

export interface SignedTransactionJSON extends TransactionJSON {
    blockNumber: number | null;
    blockHash: string | null;
    transactionIndex: number | null;
    sig: string;
    signerPublic: string;
    hash: string;
}

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
    public signerPublic: string;
    public blockNumber: number | null;
    public blockHash: H256 | null;
    public transactionIndex: number | null;
    private _signature: string;

    /**
     * @param unsigned A Transaction.
     * @param signature An Ed25519 signature which is a 64 byte hexadecimal string.
     * @param signerPublic An Ed25519 public key which is a 32 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the tx.
     * @param blockHash The hash of the block that contains the tx.
     * @param transactionIndex The index(location) of the tx within the block.
     */
    constructor(
        unsigned: Transaction,
        signature: string,
        signerPublic: string,
        blockNumber?: number,
        blockHash?: H256,
        transactionIndex?: number
    ) {
        this.unsigned = unsigned;
        this._signature = signature.startsWith("0x")
            ? signature.substr(2)
            : signature;
        this.signerPublic = signerPublic.startsWith("0x")
            ? signerPublic.substr(2)
            : signerPublic;
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
        const { unsigned, _signature, signerPublic } = this;
        const result = unsigned.toEncodeObject();
        result.push(`0x${_signature}`);
        result.push(`${H256.ensure(signerPublic).toEncodeObject()}`);
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

    /**
     * Get the platform address of a tx's signer.
     * @returns A address.
     * @deprecated
     */
    public getSignerAddress(params: { networkId: NetworkId }): Address {
        return Address.fromPublic(this.getSignerPublic(), params);
    }

    /**
     * Get the public key of a tx's signer.
     * @returns A public key.
     */
    public getSignerPublic(): H256 {
        return new H256(this.signerPublic);
    }

    /**
     * Convert to a SignedTransaction JSON object.
     * @returns A SignedTransaction JSON object.
     */
    public toJSON(): SignedTransactionJSON {
        const {
            blockNumber,
            blockHash,
            transactionIndex,
            unsigned,
            _signature,
            signerPublic
        } = this;
        const json = {
            ...unsigned.toJSON(),
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.toJSON(),
            transactionIndex,
            sig: `0x${_signature}`,
            signerPublic: `0x${signerPublic}`,
            hash: this.hash().toJSON()
        };
        return json;
    }
}
