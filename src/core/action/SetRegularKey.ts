import { H512 } from "../H512";

export class SetRegularKey {
    public key: H512;

    constructor(key: H512) {
        this.key = key;
    }

    public toEncodeObject(): any[] {
        return [3, this.key.toEncodeObject()];
    }

    public toJSON() {
        return {
            action: "setRegularKey",
            key: this.key.toJSON()
        };
    }
}
