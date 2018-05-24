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

    static fromJSON(data: any) {
        const { nonce, key } = data;
        return new this({
            nonce: new U256(nonce),
            key: new H512(key),
        });
    }

    toEncodeObject() {
        const { nonce, key } = this.data;
        return [2, nonce.toEncodeObject(), key.toEncodeObject()];
    }
}
