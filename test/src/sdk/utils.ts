import {
    blake128 as _blake128,
    blake128WithKey as _blake128WithKey,
    blake160 as _blake160,
    blake160WithKey as _blake160WithKey,
    blake256 as _blake256,
    blake256WithKey as _blake256WithKey,
    generatePrivateKey as _generatePrivateKey,
    getPublicFromPrivate as _getPublicFromPrivate,
    ripemd160 as _ripemd160,
    signEd25519 as _signEd25519,
    toHex as _toHex,
    verifyEd25519 as _verifyEd25519
} from "../primitives/src";

/**
 * Converts buffer to hexadecimal string.
 * @param buffer arbritrary length of data
 * @returns hexadecimal string
 */
export const toHex = (buffer: Buffer): string => _toHex(buffer);

/**
 * Gets data's 256 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 32 byte hexadecimal string
 */
export const blake256 = (data: Buffer | string): string => _blake256(data);

/**
 * Gets data's 160 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export const blake160 = (data: Buffer | string): string => _blake160(data);

/**
 * Gets data's 128 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 16 byte hexadecimal string
 */
export const blake128 = (data: Buffer | string): string => _blake128(data);

/**
 * Gets data's 256 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 32 byte hexadecimal string
 */
export const blake256WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => _blake256WithKey(data, key);

/**
 * Gets data's 160 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 20 byte hexadecimal string
 */
export const blake160WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => _blake160WithKey(data, key);

/**
 * Gets data's 128 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 16 byte hexadecimal string
 */
export const blake128WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => _blake128WithKey(data, key);

/**
 * Gets data's 160 bit RIPEMD hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export const ripemd160 = (data: Buffer | string): string => _ripemd160(data);

export interface SignatureTag {
    input: "all" | "single";
    output: "all" | number[];
}

/**
 * @hidden
 */
export const encodeSignatureTag = (tag: SignatureTag): Buffer => {
    const { input, output } = tag;
    if (input !== "all" && input !== "single") {
        throw Error(
            `Expected the input of the tag to be either "all" or "single" but found ${input}`
        );
    }

    const inputMask = input === "all" ? 0b01 : 0b00;
    const outputMask = output === "all" ? 0b10 : 0b00;
    if (Array.isArray(output)) {
        // NOTE: Remove duplicates by using Set
        const encoded = encodeSignatureTagOutput(
            Array.from(new Set(output)).sort((a, b) => a - b)
        );
        if (encoded.length >= 64) {
            throw Error(`The output length is too big`);
        }
        return Buffer.from([
            ...encoded,
            (encoded.length << 2) | outputMask | inputMask
        ]);
    } else if (output === "all") {
        return Buffer.from([outputMask | inputMask]);
    } else {
        throw Error(
            `Expected the output of the tag to be either string "all" or an array of number but found ${output}`
        );
    }
};

/**
 * @hidden
 */
const encodeSignatureTagOutput = (output: number[]) => {
    // NOTE: Assume all numbers are distinct and the array is sorted by increasing order.
    if (output[0] < 0) {
        throw Error(`Invalid signature tag. Out of range: ${output[0]}`);
    } else if (output[output.length - 1] > 503) {
        throw Error(
            `Invalid signature tag. Out of range: ${output[output.length - 1]}`
        );
    }
    let offset = 0;
    let byte = 0;
    const bytes = [];
    for (const index of output) {
        if (typeof index !== "number") {
            throw Error(
                `Expected an array of number but found ${index} at ${output.indexOf(
                    index
                )}`
            );
        }
        if (index < offset + 8) {
            byte |= 1 << (index - offset);
        } else {
            bytes.push(byte);
            offset += 8;
            while (index >= offset + 8) {
                bytes.push(0);
                offset += 8;
            }
            byte = 1 << (index - offset);
        }
    }
    if (byte !== 0) {
        bytes.push(byte);
    }
    return bytes.reverse();
};

export type Ed25519Signature = string;

/**
 * Gets signature for message from private key.
 * @param message arbitrary length string
 * @param priv 32 byte hexstring of private key
 * @returns 65 byte hexstring of Ed25519 signature
 */
export const signEd25519 = (
    message: string,
    priv: string
): Ed25519Signature => {
    return _signEd25519(message, priv);
};

/**
 * Checks if the signature from signEd25519 is correct.
 * @param message arbitrary length string
 * @param signature 65 byte hexstring of Ed25519 signature
 * @param pub 64 byte hexstring of public key
 * @returns if signature is valid, true. Else false.
 */
export const verifyEd25519 = (
    message: string,
    signature: Ed25519Signature,
    pub: string
): boolean => {
    if (signature.startsWith("0x")) {
        signature = signature.substr(2);
    }
    return _verifyEd25519(message, signature, pub);
};

/**
 * Generates a private key.
 * @returns 32 byte hexadecimal string of private key
 */
export const generatePrivateKey = (): string => _generatePrivateKey();

/**
 * Gets public key from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 64 byte hexadecimal string of public key
 */
export const getPublicFromPrivate = (priv: string): string =>
    _getPublicFromPrivate(priv);
