import { Buffer } from "buffer";

import { H160 } from "../core/H160";
import { toHex } from "../utils";

import { toWords, encode, decode, fromWords } from "./bech32";

/**
 * The bech32 form of account id. The human readable part(HRP) is used to
 * represent the network. For platform address, the HRP is "ccc" for mainnet or
 * "tcc" for testnet.
 *
 * Refer to the spec for the details about PlatformAddress.
 * https://github.com/CodeChain-io/codechain/blob/master/spec/CodeChain-Address.md
 */
export class PlatformAddress {
    readonly accountId: H160;
    readonly value: string;

    private constructor(accountId: H160, address: string) {
        this.accountId = accountId;
        this.value = address;
    }

    static fromAccountId(accountId: H160, options: { isTestnet?: boolean, version?: number } = {}) {
        const { isTestnet = false, version = 0 } = options;

        if (version !== 0) {
            throw `Unsupported version for platform address: ${version}`;
        }

        const words = toWords(Buffer.from([version, ...Buffer.from(accountId.value, "hex")]));
        return new PlatformAddress(accountId, encode(isTestnet ? "tcc" : "ccc", words));
    }

    static fromString(address: string) {
        if (!address.startsWith("ccc") && !address.startsWith("tcc")) {
            throw `The prefix is unknown for platform address: ${address}`;
        }

        const { words } = decode(address, address.substr(0, 3));
        const bytes = fromWords(words);
        const version = bytes[0];

        if (version !== 0) {
            throw `Unsupported version for platform address: ${version}`;
        }

        const accountId = toHex(Buffer.from(bytes.slice(1)));
        return new PlatformAddress(new H160(accountId), address);
    }

    toString(): string {
        return this.value;
    }

    getAccountId(): H160 {
        return this.accountId;
    }

    static ensure(address: PlatformAddress | string) {
        return address instanceof PlatformAddress ? address : PlatformAddress.fromString(address);
    }
}
