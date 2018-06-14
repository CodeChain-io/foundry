import { H160 } from "../H160";
import { H256 } from "../H256";
import { U256 } from "../U256";
import { blake256WithKey, blake256 } from "../../utils";

const RLP = require("rlp");

export type AssetMintTransactionData = {
    metadata: string;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number | null;
    registrar: H160 | null;
    nonce: number;
};

/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by lockScriptHash and parameters.
 * - metadata is a string that explains the asset's type.
 * - amount defines the quantity of asset to be created. If set as null, it will be set as the maximum value of a 64-bit unsigned integer by default.
 * - If registrar exists, the registrar must be the Signer of the Parcel when sending the created asset through AssetTransferTransaction.
 * - Transaction hash can be changed by changing nonce.
 * - If an identical transaction hash already exists, then the change fails. In this situation, a transaction can be created again by arbitrarily changing the nonce.
 */
export class AssetMintTransaction {
    private data: AssetMintTransactionData;
    private type = "assetMint";

    constructor(data: AssetMintTransactionData) {
        this.data = data;
    }

    static fromJSON(data: any) {
        const { metadata, lockScriptHash, parameters, amount, registrar, nonce } = data["assetMint"];
        return new this({
            metadata,
            lockScriptHash: new H256(lockScriptHash),
            parameters,
            amount: amount === null ? null : amount,
            registrar: registrar === null ? null : new H160(registrar),
            nonce,
        });
    }

    toJSON() {
        const { metadata, lockScriptHash, parameters, amount, registrar, nonce } = this.data;
        return {
            [this.type]: {
                metadata,
                lockScriptHash: lockScriptHash.value,
                parameters,
                amount,
                registrar: registrar === null ? null : registrar.value,
                nonce,
            }
        };
    }

    toEncodeObject() {
        const { metadata, lockScriptHash, parameters, amount, registrar, nonce } = this.data;
        return [
            3,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters,
            amount ? [amount] : [],
            registrar ? [registrar.toEncodeObject()] : [],
            nonce
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    getAssetSchemeAddress(): H256 {
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ]));
        const prefix = "5300000000000000";
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }

    getAssetAddress(): H256 {
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]));
        const prefix = "4100000000000000";
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}
