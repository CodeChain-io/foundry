import { AssetTransferAddress, H160, H256, U64 } from "codechain-primitives";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";

import { Asset } from "../Asset";

export interface WrapCCCData {
    shardId: number;
    lockScriptHash: H160;
    parameters: Buffer[];
    amount: U64;
}

export interface WrapCCCAddressData {
    shardId: number;
    recipient: AssetTransferAddress;
    amount: U64;
}

export class WrapCCC {
    public readonly shardId: number;
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly amount: U64;

    constructor(data: WrapCCCData | WrapCCCAddressData) {
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
        const { shardId, amount } = data;
        this.shardId = shardId;
        this.amount = amount;
    }

    /**
     * Get the address of the asset scheme of the wrapped CCC asset. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    public getAssetSchemeAddress(): H256 {
        const shardPrefix = convertU16toHex(this.shardId);
        const prefix = `5300${shardPrefix}`;
        const hash = prefix.concat("0".repeat(56));
        return new H256(hash);
    }

    /**
     * Get the wrapped CCC asset output of this parcel.
     * @param parcelHash A hash value of containing parcel
     * @returns An Asset.
     */
    public getAsset(parcelHash: H256): Asset {
        const { lockScriptHash, parameters, amount } = this;
        return new Asset({
            assetType: this.getAssetSchemeAddress(),
            lockScriptHash,
            parameters,
            amount,
            transactionHash: parcelHash,
            transactionOutputIndex: 0
        });
    }

    public toEncodeObject(): any[] {
        const { shardId, lockScriptHash, parameters, amount } = this;
        return [
            7,
            shardId,
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            amount.toEncodeObject()
        ];
    }

    public toJSON() {
        const { shardId, lockScriptHash, parameters, amount } = this;
        return {
            action: "wrapCCC",
            shardId,
            lockScriptHash: lockScriptHash.toJSON(),
            parameters: parameters.map(parameter => [...parameter]),
            amount: amount.toJSON()
        };
    }
}

// FIXME: Need to move the function to the external file. Also used in AssetMintTransaction.
function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
