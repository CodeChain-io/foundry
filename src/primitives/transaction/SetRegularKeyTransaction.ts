import { H512, U256 } from "../index";

export type SetRegularKeyTransactionData = {
    nonce: U256;
    key: H512;
};

export class SetRegularKeyTransaction {
    private data: SetRegularKeyTransactionData;
    private type = "setRegularKey";

    constructor(data: SetRegularKeyTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { nonce, key } = this.data;
        return [2, nonce.toEncodeObject(), key.toEncodeObject()];
    }
}
