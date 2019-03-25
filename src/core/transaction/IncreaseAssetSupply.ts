import { H160, H256 } from "codechain-primitives";
import { blake256 } from "../../utils";
import { Asset } from "../Asset";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";
import { AssetMintOutput, AssetMintOutputJSON } from "./AssetMintOutput";

const RLP = require("rlp");

export interface IncreaseAssetSupplyTransactionJSON {
    networkId: string;
    shardId: number;
    assetType: string;
    output: AssetMintOutputJSON;
}

export interface IncreaseAssetSupplyActionJSON
    extends IncreaseAssetSupplyTransactionJSON {
    approvals: string[];
}

export class IncreaseAssetSupply extends Transaction {
    private readonly transaction: IncreaseAssetSupplyTransaction;
    private readonly approvals: string[];

    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        assetType: H160;
        output: AssetMintOutput;
        approvals: string[];
    }) {
        super(params.networkId);

        this.transaction = new IncreaseAssetSupplyTransaction(params);
        this.approvals = params.approvals;
    }

    public tracker(): H256 {
        return new H256(blake256(this.transaction.rlpBytes()));
    }

    /**
     * Add an approval to transaction.
     * @param approval An approval
     */
    public addApproval(approval: string) {
        this.approvals.push(approval);
    }

    public output(): AssetMintOutput {
        return this.transaction.output;
    }

    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    public getMintedAsset(): Asset {
        const { assetType, shardId, output } = this.transaction;
        const { lockScriptHash, parameters, supply } = output;
        return new Asset({
            assetType,
            shardId,
            lockScriptHash,
            parameters,
            quantity: supply,
            tracker: this.tracker(),
            transactionOutputIndex: 0
        });
    }

    public type(): string {
        return "increaseAssetSupply";
    }

    protected actionToEncodeObject(): any[] {
        const encoded = this.transaction.toEncodeObject();
        encoded.push(this.approvals);
        return encoded;
    }

    protected actionToJSON(): IncreaseAssetSupplyActionJSON {
        const json = this.transaction.toJSON();
        return {
            ...json,
            approvals: this.approvals
        };
    }
}

class IncreaseAssetSupplyTransaction {
    public readonly shardId: number;
    public readonly assetType: H160;
    public readonly output: AssetMintOutput;
    private readonly networkId: NetworkId;

    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        assetType: H160;
        output: AssetMintOutput;
    }) {
        this.networkId = params.networkId;
        this.shardId = params.shardId;
        this.assetType = params.assetType;
        this.output = new AssetMintOutput(params.output);
    }

    public toJSON(): IncreaseAssetSupplyTransactionJSON {
        return {
            networkId: this.networkId,
            shardId: this.shardId,
            assetType: this.assetType.toEncodeObject(),
            output: this.output.toJSON()
        };
    }

    public toEncodeObject(): any[] {
        return [
            0x18,
            this.networkId,
            this.shardId,
            this.assetType.toEncodeObject(),
            this.output.lockScriptHash.toEncodeObject(),
            this.output.parameters.map(parameter => Buffer.from(parameter)),
            this.output.supply.toEncodeObject()
        ];
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
