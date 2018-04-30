import { H160, U256 } from "./index";

const RLP = require("rlp");

type NoopActionData = {};
type PaymentActionData = {
    address: H160;
    value: U256;
};

// FIXME: Support "set_regular_key", "asset_mint" and etc.
export type ActionType = "noop" | "payment";
export type ActionData = NoopActionData | PaymentActionData;

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
        }
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}
