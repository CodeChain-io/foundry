import { H512 } from "../H512";
export declare class SetRegularKey {
    key: H512;
    constructor(key: H512);
    toEncodeObject(): any[];
    toJSON(): {
        action: string;
        key: string;
    };
}
