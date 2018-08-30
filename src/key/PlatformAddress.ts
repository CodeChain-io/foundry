import { Buffer } from "buffer";
import * as _ from "lodash";

import { H160 } from "../core/H160";
import { toHex } from "../utils";

import { decode, encode, fromWords, toWords } from "./bech32";

/**
 * The bech32 form of account id. The human readable part(HRP) is used to
 * represent the network. For platform address, the HRP is "ccc" for mainnet or
 * "tcc" for testnet.
 *
 * Refer to the spec for the details about PlatformAddress.
 * https://github.com/CodeChain-io/codechain/blob/master/spec/CodeChain-Address.md
 */
export class PlatformAddress {
    public static fromAccountId(
        accountId: H160 | string,
        options: { networkId?: string; version?: number } = {}
    ) {
        const { networkId = "tc", version = 0 } = options;

        if (version !== 0) {
            throw Error(`Unsupported version for platform address: ${version}`);
        }

        const words = toWords(
            Buffer.from(
                _.padStart(version.toString(16), 2, "0") +
                    H160.ensure(accountId).value,
                "hex"
            )
        );
        return new PlatformAddress(
            H160.ensure(accountId),
            encode(networkId + "c", words)
        );
    }

    public static fromString(address: string) {
        if (address.charAt(2) !== "c") {
            throw Error(
                `The prefix is unknown for platform address: ${address}`
            );
        }

        const { words } = decode(address, address.substr(0, 3));
        const bytes = fromWords(words);
        const version = bytes[0];

        if (version !== 0) {
            throw Error(`Unsupported version for platform address: ${version}`);
        }

        const accountId = toHex(Buffer.from(bytes.slice(1)));
        return new PlatformAddress(new H160(accountId), address);
    }

    public static ensure(address: PlatformAddress | string): PlatformAddress {
        return address instanceof PlatformAddress
            ? address
            : PlatformAddress.fromString(address);
    }

    public static ensureAccount(
        address: PlatformAddress | H160 | string
    ): H160 {
        if (address instanceof PlatformAddress) {
            // FIXME: verify network id
            return address.getAccountId();
        } else if (address instanceof H160) {
            return address;
        } else if (address.match(`^(0x)?[a-fA-F0-9]{40}$`)) {
            return new H160(address);
        } else {
            return PlatformAddress.fromString(address).getAccountId();
        }
    }
    public readonly accountId: H160;
    public readonly value: string;

    private constructor(accountId: H160, address: string) {
        this.accountId = accountId;
        this.value = address;
    }

    public toString(): string {
        return this.value;
    }

    public getAccountId(): H160 {
        return this.accountId;
    }
}
