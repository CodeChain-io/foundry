const RLP = require("rlp");

// FIXME: Support "payment", "set_regular_key", "asset_mint" and etc.
type ActionType = "noop";

class Action {
    type: ActionType;

    constructor(type: ActionType) {
        this.type = type;
    }

    toEncodeObject(): string {
        // FIXME: noop hard-coded here.
        return "";
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }
}

export default Action;
