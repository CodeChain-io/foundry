import * as _ from "lodash";

import {
    blake128,
    blake256,
    blake256WithKey,
    encodeSignatureTag,
    SignatureTag
} from "../../utils";
import { H256 } from "../H256";
import { NetworkId } from "../types";
import {
    AssetTransferInput,
    AssetTransferInputJSON
} from "./AssetTransferInput";

const RLP = require("rlp");

export interface AssetUnwrapCCCTransactionJSON {
    type: "assetUnwrapCCC";
    data: {
        burn: AssetTransferInputJSON;
        networkId: NetworkId;
    };
}

export interface AssetUnwrapCCCTransactionData {
    burn: AssetTransferInput;
    networkId: NetworkId;
}
/**
 * Spend a wrapped CCC asset and change it to CCC.
 *
 * An AssetUnwrapCCCTransaction consists of:
 *  - An AssetTransferInput of which asset type is wrapped CCC.
 *  - A network ID. This must be identical to the network ID of which the
 *  transaction is being sent to.
 *
 * All inputs must be valid for the transaction to be valid. When each asset
 * types' amount have been summed, the sum of inputs and the sum of outputs
 * must be identical.
 */
export class AssetUnwrapCCCTransaction {
    /** Create an AssetUnwrapCCCTransaction from an AssetUnwrapCCCTransactionJSON object.
     * @param obj An AssetUnwrapCCCTransactionJSON object.
     * @returns An AssetUnwrapCCCTransaction.
     */
    public static fromJSON(obj: AssetUnwrapCCCTransactionJSON) {
        const {
            data: { networkId, burn }
        } = obj;
        return new this({
            burn: AssetTransferInput.fromJSON(burn),
            networkId
        });
    }
    public readonly burn: AssetTransferInput;
    public readonly networkId: NetworkId;
    public readonly type = "assetUnwrapCCC";

    /**
     * @param params.burn An AssetTransferInput of which asset type is wrapped CCC.
     * @param params.networkId A network ID of the transaction.
     */
    constructor(params: AssetUnwrapCCCTransactionData) {
        const { burn, networkId } = params;
        this.burn = burn;
        this.networkId = networkId;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        return [1, this.networkId, this.burn.toEncodeObject()];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetUnwrapCCCTransaction.
     * @returns A transaction hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    public hashWithoutScript(params?: {
        tag: SignatureTag;
        type: "input" | "burn";
        index: number;
    }): H256 {
        const { networkId } = this;
        const { tag = { input: "all", output: "all" } as SignatureTag } =
            params || {};
        if (tag.input !== "all" || tag.output !== "all") {
            throw Error(`Invalid tag input: ${tag}`);
        }

        return new H256(
            blake256WithKey(
                new AssetUnwrapCCCTransaction({
                    burn: this.burn.withoutScript(),
                    networkId
                }).rlpBytes(),
                Buffer.from(blake128(encodeSignatureTag(tag)), "hex")
            )
        );
    }

    /**
     * Convert to an AssetUnwrapCCCTransactionJSON object.
     * @returns An AssetUnwrapCCCTransactionJSON object.
     */
    public toJSON() {
        const { networkId, burn } = this;
        return {
            type: this.type,
            data: {
                networkId,
                burn: burn.toJSON()
            }
        };
    }
}
