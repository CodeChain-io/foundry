import { H512 } from "../index";

type SetRegularKeyActionData = H512;

export class SetRegularKeyAction {
    private key: H512;

    constructor(data: SetRegularKeyActionData) {
        this.key = data;
    }

    toEncodeObject() {
        return [2, this.key.toEncodeObject()];
    }
}