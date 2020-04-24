import { Buffer } from "buffer";
import * as _ from "lodash";

import { toHex } from "../utility";
import { H256, H256Value } from "../value/H256";

import { decode, encode, fromWords, toWords } from "./bech32";

export type AddressValue = Address | string;

/**
 * The bech32 form of account id. The human readable part(HRP) is used to
 * represent the network. For address, the HRP is "ccc" for mainnet or
 * "tcc" for testnet.
 *
 * Refer to the spec for the details about Address.
 * https://github.com/CodeChain-io/codechain/blob/master/spec/CodeChain-Address.md
 */
export class Address {
    public static fromPublic(
        pubkey: H256Value,
        options: { networkId: string; version?: number }
    ) {
        const { networkId, version = 1 } = options;

        if (!H256.check(pubkey)) {
            throw Error(`Invalid public key for creating Address: "${pubkey}"`);
        }
        if (version !== 1) {
            throw Error(`Unsupported version for Address: "${version}"`);
        }
        if (typeof networkId !== "string" || networkId.length !== 2) {
            throw Error(`Unsupported networkId for Address: "${networkId}"`);
        }

        const words = toWords(
            Buffer.from(
                _.padStart(version.toString(16), 2, "0") +
                    H256.ensure(pubkey).value,
                "hex"
            )
        );
        return new Address(H256.ensure(pubkey), encode(networkId + "c", words));
    }

    public static fromString(address: string) {
        if (typeof address !== "string") {
            throw Error(`Expected Address string but found: "${address}"`);
        } else if (address.charAt(2) !== "c") {
            throw Error(`Unknown prefix for Address: ${address}`);
        }

        const { words } = decode(address, address.substr(0, 3));
        const bytes = fromWords(words);
        const version = bytes[0];

        if (version !== 1) {
            throw Error(`Unsupported version for Address: ${version}`);
        }

        const pubkey = toHex(Buffer.from(bytes.slice(1)));
        return new Address(new H256(pubkey), address);
    }

    public static check(address: any): boolean {
        return address instanceof Address ? true : Address.checkString(address);
    }

    public static ensure(address: AddressValue): Address {
        if (address instanceof Address) {
            return address;
        } else if (typeof address === "string") {
            return Address.fromString(address);
        } else {
            throw Error(
                `Expected either Address or string but found ${address}`
            );
        }
    }

    public static ensureAccount(address: Address | H256 | string): H256 {
        if (address instanceof Address) {
            // FIXME: verify network id
            return address.getPubKey();
        } else if (address instanceof H256) {
            return address;
        } else if (address.match(`^(0x)?[a-fA-F0-9]{40}$`)) {
            return new H256(address);
        } else {
            return Address.fromString(address).getPubKey();
        }
    }

    private static checkString(value: string): boolean {
        // FIXME: verify checksum
        return /^.{2}c[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{59}$/.test(value);
    }

    public readonly pubkey: H256;
    public readonly value: string;

    private constructor(pubkey: H256, address: string) {
        this.pubkey = pubkey;
        this.value = address;
    }

    public toString(): string {
        return this.value;
    }

    public getPubKey(): H256 {
        return this.pubkey;
    }
}
