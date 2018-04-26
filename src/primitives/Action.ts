const RLP = require("rlp");

// FIXME: Support "payment", "set_regular_key", "asset_mint" and etc.
type ActionType = "noop";

class Action {
    type: ActionType;

    constructor(type: ActionType) {
        this.type = type;
    }

    rlpBytes(): Buffer {
        // FIXME: noop hard-coded here.
        return RLP.encode("");
    }
}

export default Action;
