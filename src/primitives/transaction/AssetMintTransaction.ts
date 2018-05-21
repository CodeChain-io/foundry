import { H160, H256, U256 } from "../index";
import { blake256WithKey, blake256 } from "../../utils";

const RLP = require("rlp");

export type AssetMintTransactionData = {
    metadata: string;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number | null;
    registrar: H160 | null;
};

export class AssetMintTransaction {
    private data: AssetMintTransactionData;

    constructor(data: AssetMintTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { metadata, lockScriptHash, parameters, amount, registrar } = this.data;
        return [
            3,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters,
            amount ? [amount] : [],
            registrar ? [registrar] : []
        ];
    }

    rlpBytes() {
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
}
