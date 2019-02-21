import { H160, PlatformAddress, U64 } from "codechain-primitives";
import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";
import { Asset } from "../Asset";
import { AssetTransferAddress } from "../classes";
import { AssetTransaction, Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface WrapCCCData {
    shardId: number;
    lockScriptHash: H160;
    parameters: Buffer[];
    quantity: U64;
    payer: PlatformAddress;
}

export interface WrapCCCAddressData {
    shardId: number;
    recipient: AssetTransferAddress;
    quantity: U64;
    payer: PlatformAddress;
}

export interface WrapCCCActionJSON {
    shardId: number;
    lockScriptHash: string;
    parameters: string[];
    quantity: string;
    payer: string;
}

export class WrapCCC extends Transaction implements AssetTransaction {
    private readonly shardId: number;
    private readonly lockScriptHash: H160;
    private readonly parameters: Buffer[];
    private readonly quantity: U64;
    private readonly payer: PlatformAddress;

    public constructor(
        data: WrapCCCData | WrapCCCAddressData,
        networkId: NetworkId
    ) {
        super(networkId);
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
            if ("pubkeys" in payload) {
                throw Error("Multisig payload is not supported yet");
            }
            switch (type) {
                case 0x00: // LOCK_SCRIPT_HASH ONLY
                    this.lockScriptHash = payload;
                    this.parameters = [];
                    break;
                case 0x01: // P2PKH
                    this.lockScriptHash = P2PKH.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                case 0x02: // P2PKHBurn
                    this.lockScriptHash = P2PKHBurn.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                default:
                    throw Error(
                        `Unexpected type of AssetTransferAddress: ${type}, ${
                            data.recipient
                        }`
                    );
            }
        } else {
            const { lockScriptHash, parameters } = data;
            this.lockScriptHash = lockScriptHash;
            this.parameters = parameters;
        }
        const { shardId, quantity, payer } = data;
        this.shardId = shardId;
        this.quantity = quantity;
        this.payer = payer;
    }

    /**
     * Get the asset type of the output.
     * @returns An asset type which is H160.
     */
    public getAssetType(): H160 {
        return H160.zero();
    }

    /**
     * Get the wrapped CCC asset output of this tx.
     * @returns An Asset.
     */
    public getAsset(): Asset {
        const { shardId, lockScriptHash, parameters, quantity } = this;
        return new Asset({
            assetType: this.getAssetType(),
            shardId,
            lockScriptHash,
            parameters,
            quantity,
            tracker: this.tracker(),
            transactionOutputIndex: 0
        });
    }

    public tracker() {
        return this.unsignedHash();
    }

    public type(): string {
        return "wrapCCC";
    }

    protected actionToEncodeObject(): any[] {
        const { shardId, lockScriptHash, parameters, quantity, payer } = this;
        return [
            7,
            shardId,
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            quantity.toEncodeObject(),
            payer.getAccountId().toEncodeObject()
        ];
    }

    protected actionToJSON(): WrapCCCActionJSON {
        const { shardId, lockScriptHash, parameters, quantity, payer } = this;
        return {
            shardId,
            lockScriptHash: lockScriptHash.toJSON(),
            parameters: parameters.map((p: Buffer) => p.toString("hex")),
            quantity: quantity.toJSON(),
            payer: payer.toString()
        };
    }
}
