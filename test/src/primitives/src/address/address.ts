import * as Base64 from "crypto-js/enc-base64";
import * as Hex from "crypto-js/enc-hex";
import * as _ from "lodash";
import { H256, H256Value } from "../value/H256";
import { calculate as calculateChecksum } from "./checksum";

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
        const { networkId, version = 0 } = options;

        if (!H256.check(pubkey)) {
            throw Error(`Invalid public key for creating Address: "${pubkey}"`);
        }
        if (version !== 0) {
            throw Error(`Unsupported version for Address: "${version}"`);
        }
        if (typeof networkId !== "string" || networkId.length !== 2) {
            throw Error(`Unsupported networkId for Address: "${networkId}"`);
        }

        const original = H256.ensure(pubkey).value;
        const hex = Hex.parse(original);
        const base64Encoded = Base64.stringify(hex);
        const base64EncodedWithoutPad = base64Encoded.substr(
            0,
            base64Encoded.length - 1
        );
        const base64urlEncoded = base64EncodedWithoutPad
            .replace(/\//g, "_")
            .replace(/\+/g, "-");

        const checksum = calculateChecksum(
            H256.ensure(pubkey),
            networkId,
            version
        );
        return new Address(
            H256.ensure(pubkey),
            checksum + base64urlEncoded + networkId + version.toString(16)
        );
    }

    public static fromString(address: string) {
        if (typeof address !== "string") {
            throw Error(`Expected Address string but found: "${address}"`);
        }
        const version = parseInt(address.substr(address.length - 1, 1), 16);
        if (version !== 0) {
            throw Error(`Unsupported version for Address: ${version}`);
        }

        const base64urlEncodedWithoutPad = address.substr(8, 43);
        const base64Encoded =
            base64urlEncodedWithoutPad.replace(/_/g, "/").replace(/-/g, "+") +
            "=";
        const decoded = Base64.parse(base64Encoded);
        const pubkey = new H256(Hex.stringify(decoded));

        const networkId = address.substr(8 + 43, 2);
        const receivedChecksum = address.substr(0, 8);
        const calculatedChecksum = calculateChecksum(
            pubkey,
            networkId,
            version
        );
        if (receivedChecksum !== calculatedChecksum) {
            throw Error(
                `The invalid checksum. ${calculatedChecksum} expected but ${receivedChecksum} received`
            );
        }

        return new Address(pubkey, address);
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
        return /^[0123456789bcdefghjkmnpqrstuvwxyz]{8}[A-Za-z0-9\-_]{43}[a-z]{2}[0-9a-f]$/.test(
            value
        );
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
