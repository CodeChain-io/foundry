import { H160, U256, H512, H256 } from "./index";

const RLP = require("rlp");

type NoopActionData = {};
type PaymentActionData = {
    address: H160;
    value: U256;
};
type SetRegularKeyActionData = {
    key: H512;
}
type AssetMintActionData = {
    metadata: string;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number | null;
    registrar: H160 | null;
}

// FIXME: Support "set_regular_key", "asset_mint" and etc.
export type ActionType = "noop"
                         | "payment"
                         | "setRegularKey"
                         | "assetMint";
export type ActionData = NoopActionData
                         | PaymentActionData
                         | SetRegularKeyActionData
                         | AssetMintActionData;

export class Action {
    type: ActionType;
    data: ActionData;

    constructor(type: ActionType, data: ActionData) {
        this.type = type;
        this.data = data;
    }

    toEncodeObject(): string | Array<any> {
        switch(this.type) {
            case "noop":
                return "";
            case "payment":
                const { address, value } = this.data as PaymentActionData;
                return [1, address.toEncodeObject(), value.toEncodeObject()];
            case "setRegularKey":
                const { key } = this.data as SetRegularKeyActionData;
                return [2, key.toEncodeObject()];
            case "assetMint":
                const {
                    metadata,
                    lockScriptHash,
                    parameters,
                    amount,
                    registrar
                } = this.data as AssetMintActionData;
                return [
                    3,
                    metadata,
                    lockScriptHash.toEncodeObject(),
                    parameters,
                    amount ? [amount] : [],
                    registrar ? [registrar] : []
                ];
        }
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
