import { blake160, blake256, blake256WithKey } from "../../utils";
import { Asset } from "../Asset";
import { AssetScheme, H160, H256, PlatformAddress } from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { NetworkId } from "../types";
import { AssetMintOutput, AssetMintOutputJSON } from "./AssetMintOutput";

const RLP = require("rlp");

export interface AssetMintTransactionJSON {
    networkId: string;
    shardId: number;
    metadata: string;
    output: AssetMintOutputJSON;
    approver: string | null;
    registrar: string | null;
    allowedScriptHashes: string[];
}
export interface MintAssetActionJSON extends AssetMintTransactionJSON {
    approvals: string[];
}

export class MintAsset extends Transaction implements AssetTransaction {
    private readonly _transaction: AssetMintTransaction;
    private readonly approvals: string[];

    public constructor(input: {
        networkId: NetworkId;
        shardId: number;
        metadata: string;
        output: AssetMintOutput;
        approver: PlatformAddress | null;
        registrar: PlatformAddress | null;
        allowedScriptHashes: H160[];
        approvals: string[];
    }) {
        super(input.networkId);

        this._transaction = new AssetMintTransaction(input);
        this.approvals = input.approvals;
    }

    /**
     * Get the tracker of an AssetMintTransaction.
     * @returns A transaction tracker.
     */
    public tracker(): H256 {
        return new H256(blake256(this._transaction.rlpBytes()));
    }

    public output(): AssetMintOutput {
        return this._transaction.output;
    }

    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    public getMintedAsset(): Asset {
        const { lockScriptHash, parameters, supply } = this._transaction.output;
        if (supply == null) {
            throw Error("not implemented");
        }
        return new Asset({
            assetType: this.getAssetType(),
            shardId: this._transaction.shardId,
            lockScriptHash,
            parameters,
            quantity: supply,
            tracker: this.tracker(),
            transactionOutputIndex: 0
        });
    }

    /**
     * Get the asset scheme of this transaction.
     * @return An AssetScheme.
     */
    public getAssetScheme(): AssetScheme {
        const {
            networkId,
            shardId,
            metadata,
            output: { supply },
            approver,
            registrar,
            allowedScriptHashes
        } = this._transaction;
        if (supply == null) {
            throw Error("not implemented");
        }
        return new AssetScheme({
            networkId,
            shardId,
            metadata,
            supply,
            approver,
            registrar,
            allowedScriptHashes,
            pool: []
        });
    }

    /**
     * Get the asset type of the output.
     * @returns An asset type which is H160.
     */
    public getAssetType(): H160 {
        const blake = blake160(this.tracker().value);
        return new H160(blake);
    }

    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    public getAssetAddress(): H256 {
        const { shardId } = this._transaction;
        const blake = blake256WithKey(
            this.tracker().value,
            new Uint8Array([
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00
            ])
        );
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new H256(
            blake.replace(new RegExp(`^.{${prefix.length}}`), prefix)
        );
    }

    public type(): string {
        return "mintAsset";
    }

    protected actionToEncodeObject(): any[] {
        const encoded = this._transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): MintAssetActionJSON {
        const json = this._transaction.toJSON();
        return {
            ...json,
            approvals: this.approvals
        };
    }
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}

/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by a lock script hash and parameters.
 *  - A metadata is a string that explains the asset's type.
 *  - Supply defines the quantity of asset to be created. If set as null, it
 *  will be set as the maximum value of a 64-bit unsigned integer by default.
 *  - If approver exists, the approver must be the Signer of the Transaction when
 *  sending the created asset through AssetTransferTransaction.
 *  - If registrar exists, the registrar can transfer without unlocking.
 */
class AssetMintTransaction {
    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly metadata: string;
    public readonly output: AssetMintOutput;
    public readonly approver: PlatformAddress | null;
    public readonly registrar: PlatformAddress | null;
    public readonly allowedScriptHashes: H160[];

    /**
     * @param data.networkId A network ID of the transaction.
     * @param data.shardId A shard ID of the transaction.
     * @param data.metadata A metadata of the asset.
     * @param data.output.lockScriptHash A lock script hash of the output.
     * @param data.output.parameters Parameters of the output.
     * @param data.output.supply Asset supply of the output.
     * @param data.approver A approver of the asset.
     * @param data.registrar A registrar of the asset.
     * @param data.allowedScriptHashes Allowed lock script hashes of the asset.
     */
    constructor(data: {
        networkId: NetworkId;
        shardId: number;
        metadata: string;
        output: AssetMintOutput;
        approver: PlatformAddress | null;
        registrar: PlatformAddress | null;
        allowedScriptHashes: H160[];
    }) {
        const {
            networkId,
            shardId,
            metadata,
            output,
            approver,
            registrar,
            allowedScriptHashes
        } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.metadata = metadata;
        this.output = output;
        this.approver = approver;
        this.registrar = registrar;
        this.allowedScriptHashes = allowedScriptHashes;
    }

    /**
     * Convert to an AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction JSON object.
     */
    public toJSON(): AssetMintTransactionJSON {
        const {
            networkId,
            shardId,
            metadata,
            output,
            approver,
            registrar,
            allowedScriptHashes
        } = this;
        return {
            networkId,
            shardId,
            metadata,
            output: output.toJSON(),
            approver: approver == null ? null : approver.toString(),
            registrar: registrar == null ? null : registrar.toString(),
            allowedScriptHashes: allowedScriptHashes.map(hash => hash.toJSON())
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            networkId,
            shardId,
            metadata,
            output: { lockScriptHash, parameters, supply },
            approver,
            registrar,
            allowedScriptHashes
        } = this;
        return [
            0x13,
            networkId,
            shardId,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            supply.toEncodeObject(),
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
