import { H512 } from "../H512";

export class SetRegularKey {
    key: H512;

    constructor(key: H512) {
        this.key = key;
    }

    toEncodeObject(): Array<any> {
        return [3, this.key.toEncodeObject()];
    }

    toJSON() {
        return {
            action: "setRegularKey",
            key: this.key.value
        };
    }
}
