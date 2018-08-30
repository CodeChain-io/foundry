import { Buffer } from "buffer";

import { H256 } from "../core/H256";
import { toHex } from "../utils";

import { decode, encode, fromWords, toWords } from "./bech32";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";

/**
 * @hidden
 */
const LOCK_SCRIPT_HASH_TYPE = 0x00;
/**
 * @hidden
 */
const PAY_TO_PUBLIC_KEY_HASH_TYPE = 0x01;
/**
 * @hidden
 */
const PAY_TO_PUBLIC_KEY_HASH_BURN_TYPE = 0x02;

/**
 * Substitutes for asset owner data which consists of network id,
 * lockScriptHash, parameters. The network id is represented with prefix
 * "cca"(mainnet) or "tca"(testnet). Currently version 0 exists only.
 *
 * Refer to the spec for the details about AssetTransferAddress.
 * https://github.com/CodeChain-io/codechain/blob/master/spec/CodeChain-Address.md
 */
export class AssetTransferAddress {
    public static fromTypeAndPayload(
        type: number,
        payload: H256 | string,
        options: { networkId?: string; version?: number } = {}
    ) {
        const { networkId = "tc", version = 0 } = options;

        if (version !== 0) {
            throw Error(
                `Unsupported version for asset transfer address: ${version}`
            );
        }

        if (type < 0x00 || type > 0x02) {
            throw Error(`Unsupported type for asset transfer address: ${type}`);
        }

        const words = toWords(
            Buffer.from([
                version,
                type,
                ...Buffer.from(H256.ensure(payload).value, "hex")
            ])
        );
        const address = encode(networkId + "a", words);
        return new AssetTransferAddress(type, payload, address);
    }

    public static fromLockScriptHash(
        lockScriptHash: H256,
        options: { networkId?: string; version?: number } = {}
    ) {
        const { networkId = "tc", version = 0 } = options;
        const type = LOCK_SCRIPT_HASH_TYPE;

        if (version !== 0) {
            throw Error(
                `Unsupported version for asset transfer address: ${version}`
            );
        }

        const words = toWords(
            Buffer.from([
                version,
                type,
                ...Buffer.from(lockScriptHash.value, "hex")
            ])
        );
        const address = encode(networkId + "a", words);
        return new AssetTransferAddress(type, lockScriptHash, address);
    }

    public static fromPublicKeyHash(
        publicKeyHash: H256,
        options: { networkId?: string; version?: number } = {}
    ) {
        const { networkId = "tc", version = 0 } = options;
        const type = PAY_TO_PUBLIC_KEY_HASH_TYPE;

        if (version !== 0) {
            throw Error(
                `Unsupported version for asset transfer address: ${version}`
            );
        }

        const words = toWords(
            Buffer.from([
                version,
                type,
                ...Buffer.from(publicKeyHash.value, "hex")
            ])
        );
        const address = encode(networkId + "a", words);
        return new AssetTransferAddress(type, publicKeyHash, address);
    }

    public static fromString(address: string) {
        if (address.charAt(2) !== "a") {
            throw Error(
                `The prefix is unknown for asset transfer address: ${address}`
            );
        }

        const { words } = decode(address, address.substr(0, 3));
        const bytes = fromWords(words);
        const version = bytes[0];

        if (version !== 0) {
            throw Error(
                `Unsupported version for asset transfer address: ${version}`
            );
        }

        const type = bytes[1];

        if (type < 0x00 || type > 0x02) {
            throw Error(`Unsupported type for asset transfer address: ${type}`);
        }

        const payload = toHex(Buffer.from(bytes.slice(2)));
        return new this(type, new H256(payload), address);
    }

    public static ensure(address: AssetTransferAddress | string) {
        return address instanceof AssetTransferAddress
            ? address
            : AssetTransferAddress.fromString(address);
    }

    public static fromLockScriptHashAndParameters(params: {
        lockScriptHash: H256 | string;
        parameters: Buffer[];
    }) {
        const { lockScriptHash, parameters } = params;
        if (
            H256.ensure(lockScriptHash).value ===
            P2PKH.getLockScriptHash().value
        ) {
            if (parameters.length === 1) {
                return this.fromTypeAndPayload(
                    1,
                    Buffer.from(parameters[0]).toString("hex")
                );
            }
            throw Error("Invalid parameter length");
        } else if (parameters.length === 0) {
            return this.fromLockScriptHash(H256.ensure(lockScriptHash));
        }
        throw Error("Unknown lock script hash");
    }
    public type: number;
    public payload: H256;

    public value: string;

    private constructor(type: number, payload: H256 | string, address: string) {
        this.type = type;
        this.payload = H256.ensure(payload);
        this.value = address;
    }

    public toString(): string {
        return this.value;
    }

    public getLockScriptHashAndParameters(): {
        lockScriptHash: H256;
        parameters: Buffer[];
    } {
        const { type, payload } = this;
        switch (type) {
            case LOCK_SCRIPT_HASH_TYPE:
                return { lockScriptHash: payload, parameters: [] };
            case PAY_TO_PUBLIC_KEY_HASH_TYPE:
                return {
                    lockScriptHash: P2PKH.getLockScriptHash(),
                    parameters: [Buffer.from(payload.value, "hex")]
                };
            case PAY_TO_PUBLIC_KEY_HASH_BURN_TYPE:
                return {
                    lockScriptHash: P2PKHBurn.getLockScriptHash(),
                    parameters: [Buffer.from(payload.value, "hex")]
                };
            default:
                throw Error("Unreachable");
        }
    }
}
