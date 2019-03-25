import { H160, H256 } from "codechain-primitives";
import { blake256 } from "../../utils";
import { PlatformAddress } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

const RLP = require("rlp");

export interface AssetSchemeChangeTransactionJSON {
    networkId: string;
    shardId: number;
    assetType: string;
    metadata: string;
    approver: string | null;
    registrar: string | null;
    allowedScriptHashes: string[];
}

export interface ChangeAssetSchemeActionJSON
    extends AssetSchemeChangeTransactionJSON {
    approvals: string[];
}
export class ChangeAssetScheme extends Transaction {
    private readonly _transaction: AssetSchemeChangeTransaction;
    private readonly approvals: string[];
    public constructor(input: {
        networkId: NetworkId;
        assetType: H160;
        shardId: number;
        metadata: string;
        approver: PlatformAddress | null;
        registrar: PlatformAddress | null;
        allowedScriptHashes: H160[];
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetSchemeChangeTransaction(input);
        this.approvals = input.approvals;
    }

    /**
     * Get the tracker of an ChangeAssetScheme.
     * @returns A transaction hash.
     */
    public tracker(): H256 {
        return new H256(blake256(this._transaction.rlpBytes()));
    }

    /**
     * Add an approval to transaction.
     * @param approval An approval
     */
    public addApproval(approval: string) {
        this.approvals.push(approval);
    }

    public type(): string {
        return "changeAssetScheme";
    }

    protected actionToEncodeObject(): (any)[] {
        const encoded = this._transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): ChangeAssetSchemeActionJSON {
        const json = this._transaction.toJSON();
        return {
            ...json,
            approvals: this.approvals
        };
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
    public readonly registrar: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];

    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.shardId A shard ID of the asset that this transaction changes.
     * @param params.assetType A asset type of the asset that this transaction changes.
     * @param params.metadata A changed metadata of the asset.
     * @param params.approver A changed approver of the asset.
     * @param params.registrar A changed registrar of the asset.
     * @param params.allowedScriptHashes Allowed lock script hashes of the asset.
     */
    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        assetType: H160;
        metadata: string;
        approver: PlatformAddress | null;
        registrar: PlatformAddress | null;
        allowedScriptHashes: H160[];
    }) {
        const {
            networkId,
            shardId,
            assetType,
            metadata,
            approver,
            registrar,
            allowedScriptHashes
        } = params;
        this.networkId = networkId;
        this.shardId = shardId;
        this.assetType = assetType;
        this.metadata = metadata;
        this.approver =
            approver == null ? null : PlatformAddress.ensure(approver);
        this.registrar =
            registrar == null ? null : PlatformAddress.ensure(registrar);
        this.allowedScriptHashes = allowedScriptHashes;
    }

    /**
     * Convert to an AssetSchemeChangeTransaction JSON object.
     * @returns An AssetSchemeChangeTransaction JSON object.
     */
    public toJSON(): AssetSchemeChangeTransactionJSON {
        return {
            networkId: this.networkId,
            shardId: this.shardId,
            assetType: this.assetType.toEncodeObject(),
            metadata: this.metadata,
            approver: this.approver == null ? null : this.approver.toString(),
            registrar:
                this.registrar == null ? null : this.registrar.toString(),
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
            registrar,
            allowedScriptHashes
        } = this;
        return [
            0x15,
            networkId,
            shardId,
            assetType.toEncodeObject(),
            metadata,
            approver ? [approver.getAccountId().toEncodeObject()] : [],
            registrar ? [registrar.getAccountId().toEncodeObject()] : [],
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
