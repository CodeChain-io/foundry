import { H256 } from "../core/H256";
import { toHex } from "../utils";

import { encode, toWords, decode, fromWords } from "./bech32";

/**
 * @hidden
 */
const LOCK_SCRIPT_HASH_TYPE = 0x00;
/**
 * @hidden
 */
const PAY_TO_PUBLIC_KEY_HASH_TYPE = 0x01;

/**
 * Substitutes for asset owner data which consists of network id,
 * lockScriptHash, parameters. The network id is represented with prefix
 * "cca"(mainnet) or "tca"(testnet). Currently version 0 exists only.
 *
 * Refer to the wiki for the details about AssetTransferAddress.
 * https://github.com/CodeChain-io/codechain/wiki/CodeChain-Address
 */
export class AssetTransferAddress {
    type: number;
    payload: H256;

    value: string;

    private constructor(type: number, payload: H256, address: string) {
        this.type = type;
        this.payload = payload;
        this.value = address;
    }

    static fromTypeAndPayload(type: number, payload: H256, options: { isTestnet?: boolean, version?: number } = {}) {
        const { isTestnet = false, version = 0 } = options;

        if (version !== 0) {
            throw `Unsupported version for asset transfer address: ${version}`;
        }

        if (type < 0x00 || type > 0x01) {
            throw `Unsupported type for asset transfer address: ${type}`;
        }

        const words = toWords(Buffer.from([version, type, ...Buffer.from(payload.value, "hex")]));
        const address = encode(isTestnet ? "tca" : "cca", words);
        return new AssetTransferAddress(type, payload, address);
    }

    static fromLockScriptHash(lockScriptHash: H256, options: { isTestnet?: boolean, version?: number } = {}) {
        const { isTestnet = false, version = 0 } = options;
        const type = LOCK_SCRIPT_HASH_TYPE;

        if (version !== 0) {
            throw `Unsupported version for asset transfer address: ${version}`;
        }

        const words = toWords(Buffer.from([version, type, ...Buffer.from(lockScriptHash.value, "hex")]));
        const address = encode(isTestnet ? "tca" : "cca", words);
        return new AssetTransferAddress(type, lockScriptHash, address);
    }

    static fromPublicKeyHash(publicKeyHash: H256, options: { isTestnet?: boolean, version?: number } = {}) {
        const { isTestnet = false, version = 0 } = options;
        const type = PAY_TO_PUBLIC_KEY_HASH_TYPE;

        if (version !== 0) {
            throw `Unsupported version for asset transfer address: ${version}`;
        }

        const words = toWords(Buffer.from([version, type, ...Buffer.from(publicKeyHash.value, "hex")]));
        const address = encode(isTestnet ? "tca" : "cca", words);
        return new AssetTransferAddress(type, publicKeyHash, address);
    }

    static fromAddress(address: string) {
        if (!address.startsWith("cca") && !address.startsWith("tca")) {
            throw `The prefix is unknown for asset transfer address: ${address}`;
        }

        const { words } = decode(address, address.substr(0, 3));
        const bytes = fromWords(words);
        const version = bytes[0];

        if (version !== 0) {
            throw `Unsupported version for asset transfer address: ${version}`;
        }

        const type = bytes[1];

        if (type < 0x00 || type > 0x01) {
            throw `Unsupported type for asset transfer address: ${type}`;
        }

        const payload = toHex(Buffer.from(bytes.slice(2)));
        return new this(type, new H256(payload), address);
    }

    getLockScriptHashAndParameters(): { lockScriptHash: H256, parameters: Buffer[] } {
        const { type, payload } = this;
        switch (type) {
            case 0x00:
                return { lockScriptHash: payload, parameters: [] };
            case 0x01:
                // FIXME:
                const lockScriptHash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
                return { lockScriptHash, parameters: [Buffer.from(payload.value, "hex")] };
            default:
                throw "Unreachable";
        }
    }
}
