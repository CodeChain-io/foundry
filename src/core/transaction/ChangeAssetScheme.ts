import { H160, PlatformAddress } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

const RLP = require("rlp");

export class ChangeAssetScheme extends Transaction {
    private readonly _transaction: AssetSchemeChangeTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        networkId: NetworkId;
        assetType: H160;
        shardId: number;
        metadata: string;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
        allowedScriptHashes: H160[];
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetSchemeChangeTransaction(input);
        this.approvals = input.approvals;
    }

    public type(): string {
        return "changeAssetScheme";
    }

    protected actionToEncodeObject(): any[] {
        const encoded = this._transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): any {
        const json = this._transaction.toJSON();
        json.approvals = this.approvals;
        return json;
    }
}

/**
 * Change asset scheme
 */
class AssetSchemeChangeTransaction {
    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly assetType: H160;
    public readonly metadata: string;
    public readonly approver: PlatformAddress | null;
    public readonly administrator: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];

    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.shardId A shard ID of the asset that this transaction changes.
     * @param params.assetType A asset type of the asset that this transaction changes.
     * @param params.metadata A changed metadata of the asset.
     * @param params.approver A changed approver of the asset.
     * @param params.administrator A changed administrator of the asset.
     * @param params.allowedScriptHashes Allowed lock script hashes of the asset.
     */
    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        assetType: H160;
        metadata: string;
        approver: PlatformAddress | null;
        administrator: PlatformAddress | null;
        allowedScriptHashes: H160[];
    }) {
        const {
            networkId,
            shardId,
            assetType,
            metadata,
            approver,
            administrator,
            allowedScriptHashes
        } = params;
        this.networkId = networkId;
        this.shardId = shardId;
        this.assetType = assetType;
        this.metadata = metadata;
        this.approver =
            approver === null ? null : PlatformAddress.ensure(approver);
        this.administrator =
            administrator === null
                ? null
                : PlatformAddress.ensure(administrator);
        this.allowedScriptHashes = allowedScriptHashes;
    }

    /**
     * Convert to an AssetSchemeChangeTransaction JSON object.
     * @returns An AssetSchemeChangeTransaction JSON object.
     */
    public toJSON(): any {
        return {
            networkId: this.networkId,
            shardId: this.shardId,
            assetType: this.assetType.toEncodeObject(),
            metadata: this.metadata,
            approver: this.approver == null ? null : this.approver.toString(),
            administrator:
                this.administrator == null
                    ? null
                    : this.administrator.toString(),
            allowedScriptHashes: this.allowedScriptHashes.map(hash =>
                hash.toJSON()
            )
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            networkId,
            shardId,
            assetType,
            metadata,
            approver,
            administrator,
            allowedScriptHashes
        } = this;
        return [
            0x15,
            networkId,
            shardId,
            assetType,
            metadata,
            approver ? [approver.getAccountId().toEncodeObject()] : [],
            administrator
                ? [administrator.getAccountId().toEncodeObject()]
                : [],
            allowedScriptHashes.map(hash => hash.toEncodeObject())
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
