import { PlatformAddress } from "codechain-primitives";
import { U256 } from "../U256";
export declare class Payment {
    receiver: PlatformAddress;
    amount: U256;
    constructor(receiver: PlatformAddress, amount: U256);
    toEncodeObject(): any[];
    toJSON(): {
        action: string;
        receiver: string;
        amount: string | number;
    };
}
