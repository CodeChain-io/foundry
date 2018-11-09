"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const _ = require("lodash");
const utils_1 = require("../utils");
const H160_1 = require("./H160");
const H256_1 = require("./H256");
const H512_1 = require("./H512");
const Parcel_1 = require("./Parcel");
const U256_1 = require("./U256");
const RLP = require("rlp");
/**
 * A [Parcel](parcel.html) signed by a private key. It is possible to request
 * the CodeChain network to process this parcel with the
 * [sendSignedParcel](chainrpc.html#sendsignedparcel) function.
 *
 * Parcels signed with a regular key has the same effect as those signed with
 * the original key. The original key is the key of the account that registered
 * the regular key.
 *
 * If any of the following is true, the Parcel will not be processed:
 * - The Parcel's processing fee is less than 10.
 * - A network ID is not identical.
 * - A seq is not identical to the signer's seq.
 */
class SignedParcel {
    // FIXME: any
    /**
     * Create a SignedParcel from a SignedParcel JSON object.
     * @param data A SignedParcel JSON object.
     * @returns A SignedParcel.
     */
    static fromJSON(data) {
        const { sig, blockNumber, blockHash, parcelIndex } = data;
        if (typeof sig !== "string") {
            throw Error("Unexpected type of sig");
        }
        if (blockNumber) {
            return new SignedParcel(Parcel_1.Parcel.fromJSON(data), sig, blockNumber, new H256_1.H256(blockHash), parcelIndex);
        }
        else {
            return new SignedParcel(Parcel_1.Parcel.fromJSON(data), sig);
        }
    }
    /**
     * Convert r, s, v values of an ECDSA signature to a string.
     * @param params.r The r value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.s The s value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.v The recovery parameter of an ECDSA signature.
     * @returns A 65 byte hexadecimal string.
     */
    static convertRsvToSignatureString(params) {
        const { r, s, v } = params;
        return `0x${_.padStart(r, 64, "0")}${_.padStart(s, 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
    }
    static convertSignatureStringToRsv(signature) {
        if (signature.startsWith("0x")) {
            signature = signature.substr(2);
        }
        const r = `0x${signature.substr(0, 64)}`;
        const s = `0x${signature.substr(64, 64)}`;
        const v = Number.parseInt(signature.substr(128, 2), 16);
        return { r, s, v };
    }
    /**
     * @param unsigned A Parcel.
     * @param sig An ECDSA signature which is a 65 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the parcel.
     * @param blockHash The hash of the block that contains the parcel.
     * @param parcelIndex The index(location) of the parcel within the block.
     */
    constructor(unsigned, sig, blockNumber, blockHash, parcelIndex) {
        this.unsigned = unsigned;
        const { r, s, v } = SignedParcel.convertSignatureStringToRsv(sig);
        this.v = v;
        this.r = new U256_1.U256(r);
        this.s = new U256_1.U256(s);
        this.blockNumber = blockNumber === undefined ? null : blockNumber;
        this.blockHash = blockHash || null;
        this.parcelIndex = parcelIndex === undefined ? null : parcelIndex;
    }
    /**
     * Get the signature of a parcel.
     */
    signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        const { unsigned: { seq, fee, action, networkId }, v, r, s } = this;
        const sig = `0x${_.padStart(r.value.toString(16), 64, "0")}${_.padStart(s.value.toString(16), 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
        if (!seq || !fee) {
            throw Error("Seq and fee in the parcel must be present");
        }
        return [
            seq.toEncodeObject(),
            fee.toEncodeObject(),
            networkId,
            action.toEncodeObject(),
            sig
        ];
    }
    /**
     * Convert to RLP bytes.
     */
    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }
    /**
     * Get the hash of a parcel.
     * @returns A parcel hash.
     */
    hash() {
        return new H256_1.H256(utils_1.blake256(this.rlpBytes()));
    }
    /**
     * Get the account ID of a parcel's signer.
     * @returns An account ID.
     * @deprecated
     */
    getSignerAccountId() {
        const { r, s, v, unsigned } = this;
        const publicKey = utils_1.recoverEcdsa(unsigned.hash().value, {
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        return new H160_1.H160(utils_1.blake160(publicKey));
    }
    /**
     * Get the platform address of a parcel's signer.
     * @returns A PlatformAddress.
     * @deprecated
     */
    getSignerAddress(params) {
        return codechain_primitives_1.PlatformAddress.fromAccountId(this.getSignerAccountId(), params);
    }
    /**
     * Get the public key of a parcel's signer.
     * @returns A public key.
     */
    getSignerPublic() {
        const { r, s, v, unsigned } = this;
        return new H512_1.H512(utils_1.recoverEcdsa(unsigned.hash().value, {
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        }));
    }
    /**
     * Convert to a SignedParcel JSON object.
     * @returns A SignedParcel JSON object.
     */
    toJSON() {
        const { blockNumber, blockHash, parcelIndex, unsigned: { seq, fee, networkId, action }, v, r, s } = this;
        const sig = SignedParcel.convertRsvToSignatureString({
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        if (!seq || !fee) {
            throw Error("Seq and fee in the parcel must be present");
        }
        return {
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.value,
            parcelIndex,
            seq: seq.value.toString(),
            fee: fee.value.toString(),
            networkId,
            action: action.toJSON(),
            sig,
            hash: this.hash().value
        };
    }
}
exports.SignedParcel = SignedParcel;
