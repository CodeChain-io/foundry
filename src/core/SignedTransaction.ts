import { H160, H256, H512, PlatformAddress, U256 } from "codechain-primitives";
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
    /**
     * Convert r, s, v values of an ECDSA signature to a string.
     * @param params.r The r value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.s The s value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.v The recovery parameter of an ECDSA signature.
     * @returns A 65 byte hexadecimal string.
     */
    public static convertRsvToSignatureString(params: {
        r: string;
        s: string;
        v: number;
    }) {
        const { r, s, v } = params;
        return `0x${_.padStart(r, 64, "0")}${_.padStart(
            s,
            64,
            "0"
        )}${_.padStart(v.toString(16), 2, "0")}`;
    }

    private static convertSignatureStringToRsv(
        signature: string
    ): { r: string; s: string; v: number } {
        if (signature.startsWith("0x")) {
            signature = signature.substr(2);
        }
        const r = `0x${signature.substr(0, 64)}`;
        const s = `0x${signature.substr(64, 64)}`;
        const v = Number.parseInt(signature.substr(128, 2), 16);
        return { r, s, v };
    }
    public unsigned: Transaction;
    public v: number;
    public r: U256;
    public s: U256;
    public blockNumber: number | null;
    public blockHash: H256 | null;
    public transactionIndex: number | null;

    /**
     * @param unsigned A Transaction.
     * @param sig An ECDSA signature which is a 65 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the tx.
     * @param blockHash The hash of the block that contains the tx.
     * @param transactionIndex The index(location) of the tx within the block.
     */
    constructor(
        unsigned: Transaction,
        sig: string,
        blockNumber?: number,
        blockHash?: H256,
        transactionIndex?: number
    ) {
        this.unsigned = unsigned;
        const { r, s, v } = SignedTransaction.convertSignatureStringToRsv(sig);
        this.v = v;
        this.r = new U256(r);
        this.s = new U256(s);
        this.blockNumber = blockNumber === undefined ? null : blockNumber;
        this.blockHash = blockHash || null;
        this.transactionIndex =
            transactionIndex === undefined ? null : transactionIndex;
    }

    /**
     * Get the signature of a tx.
     */
    public signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject(): any[] {
        const { unsigned, v, r, s } = this;
        const sig = `0x${_.padStart(r.value.toString(16), 64, "0")}${_.padStart(
            s.value.toString(16),
            64,
            "0"
        )}${_.padStart(v.toString(16), 2, "0")}`;
        const result = unsigned.toEncodeObject();
        result.push(sig);
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
        const { r, s, v, unsigned } = this;
        const publicKey = recoverEcdsa(unsigned.hash().value, {
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
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
        const { r, s, v, unsigned } = this;
        return new H512(
            recoverEcdsa(unsigned.hash().value, {
                r: r.value.toString(16),
                s: s.value.toString(16),
                v
            })
        );
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
            v,
            r,
            s
        } = this;
        const sig = SignedTransaction.convertRsvToSignatureString({
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        const result = unsigned.toJSON();
        result.blockNumber = blockNumber;
        result.blockHash = blockHash === null ? null : blockHash.toJSON();
        result.transactionIndex = transactionIndex;
        result.sig = sig;
        result.hash = this.hash().toJSON();
        return result;
    }
}
