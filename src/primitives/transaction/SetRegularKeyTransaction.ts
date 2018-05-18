import { H512 } from "../index";

export type SetRegularKeyTransactionData = H512;

export class SetRegularKeyTransaction {
    private key: H512;

    constructor(data: SetRegularKeyTransactionData) {
        this.key = data;
    }

    toEncodeObject() {
        return [2, this.key.toEncodeObject()];
    }
}