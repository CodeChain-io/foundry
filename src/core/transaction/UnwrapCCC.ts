import {
    blake128,
    blake256,
    blake256WithKey,
    encodeSignatureTag,
    SignatureTag
} from "../../utils";
import { AssetTransferInput, H256 } from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { NetworkId } from "../types";

const RLP = require("rlp");

export class UnwrapCCC extends Transaction implements AssetTransaction {
    private readonly _transaction: AssetUnwrapCCCTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        burn: AssetTransferInput;
        networkId: NetworkId;
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetUnwrapCCCTransaction(input);
        this.approvals = input.approvals;
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
        const { networkId } = this._transaction;
        const { tag = { input: "all", output: "all" } as SignatureTag } =
            params || {};
        if (tag.input !== "all" || tag.output !== "all") {
            throw Error(`Invalid tag input: ${tag}`);
        }

        return new H256(
            blake256WithKey(
                new AssetUnwrapCCCTransaction({
                    burn: this._transaction.burn.withoutScript(),
                    networkId
                }).rlpBytes(),
                Buffer.from(blake128(encodeSignatureTag(tag)), "hex")
            )
        );
    }

    public burn(index: number): AssetTransferInput | null {
        if (0 < index) {
            return null;
        }
        return this._transaction.burn;
    }

    public id() {
        return new H256(blake256(this._transaction.rlpBytes()));
    }

    public action(): string {
        return "unwrapCCC";
    }

    protected actionToEncodeObject(): any[] {
        const transaction = this._transaction.toEncodeObject();
        const approvals = this.approvals;
        return [1, transaction, approvals];
    }

    protected actionToJSON(): any {
        const json = this._transaction.toJSON();
        json.approvals = this.approvals;
        return json;
    }
}

interface AssetUnwrapCCCTransactionData {
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
class AssetUnwrapCCCTransaction {
    public readonly burn: AssetTransferInput;
    public readonly networkId: NetworkId;
    public readonly action = "unwrapCCC";

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
     * Convert to an AssetUnwrapCCCTransactionJSON object.
     * @returns An AssetUnwrapCCCTransactionJSON object.
     */
    public toJSON(): any {
        const { networkId, burn } = this;
        return {
            action: this.action,
            networkId,
            burn: burn.toJSON()
        };
    }
}
