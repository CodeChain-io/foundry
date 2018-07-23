import { Buffer } from "buffer";

import { H160 } from "../core/H160";
import { toHex } from "../utils";

import { toWords, encode, decode, fromWords } from "./bech32";

/**
 * Substitutes for platform token owner which consists of network id and account
 * id. . The network id is represented with prefix "ccc"(mainnet) or
 * "tcc"(testnet). Currently version 0 exists only.
 *
 * Refer to the wiki for the details about PlatformAddress.
 * https://github.com/CodeChain-io/codechain/wiki/CodeChain-Address
 */
export class PlatformAddress {
    accountId: H160;

    value: string;

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

    static ensure(address: PlatformAddress | string) {
        return address instanceof PlatformAddress ? address : PlatformAddress.fromString(address);
    }
}
