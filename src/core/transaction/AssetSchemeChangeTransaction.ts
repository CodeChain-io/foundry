import { H256, PlatformAddress } from "codechain-primitives/lib";
import { blake256 } from "../../utils";
import { NetworkId } from "../types";

const RLP = require("rlp");

export interface AssetSchemeChangeTransactionJSON {
    type: "assetSchemeChange";
    data: {
        networkId: NetworkId;
        assetType: string;
        metadata: string;
        approver?: string | null;
        administrator?: string | null;
    };
}

/**
 * Change asset scheme
 */
export class AssetSchemeChangeTransaction {
    public static fromJSON(obj: AssetSchemeChangeTransactionJSON) {
        const {
            data: { networkId, assetType, metadata, approver, administrator }
        } = obj;
        return new this({
            networkId,
            assetType: new H256(assetType),
            metadata,
            approver:
                approver == null ? null : PlatformAddress.ensure(approver),
            administrator:
                administrator == null
                    ? null
                    : PlatformAddress.ensure(administrator)
        });
    }

    public readonly networkId: NetworkId;
    public readonly assetType: H256;
    public readonly metadata: string;
    public readonly approver: PlatformAddress | null;
    public readonly administrator: PlatformAddress | null;
    public readonly type = "assetSchemeChange";

    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.assetType A asset type that this transaction changes.
     * @param params.metadata A changed metadata of the asset.
     * @param params.approver A changed approver of the asset.
     * @param params.administrator A changed administrator of the asset.
     */
    constructor(params: {
        networkId: NetworkId;
        assetType: H256;
        metadata: string;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
    }) {
        const {
            networkId,
            assetType,
            metadata,
            approver,
            administrator
        } = params;
        this.networkId = networkId;
        this.assetType = assetType;
        this.metadata = metadata;
        this.approver =
            approver === null ? null : PlatformAddress.ensure(approver);
        this.administrator =
            administrator === null
                ? null
                : PlatformAddress.ensure(administrator);
    }

    /**
     * Convert to an AssetSchemeChangeTransaction JSON object.
     * @returns An AssetSchemeChangeTransaction JSON object.
     */
    public toJSON(): AssetSchemeChangeTransactionJSON {
        return {
            type: this.type,
            data: {
                networkId: this.networkId,
                assetType: this.assetType.toEncodeObject(),
                metadata: this.metadata,
                approver:
                    this.approver == null ? null : this.approver.toString(),
                administrator:
                    this.administrator == null
                        ? null
                        : this.administrator.toString()
            }
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            networkId,
            assetType,
            metadata,
            approver,
            administrator
        } = this;
        return [
            5,
            networkId,
            assetType,
            metadata,
            approver ? [approver.getAccountId().toEncodeObject()] : [],
            administrator ? [administrator.getAccountId().toEncodeObject()] : []
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetMintTransaction.
     * @returns A transaction hash.
     */
    public id(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
