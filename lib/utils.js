"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
/**
 * Converts buffer to hexadecimal string.
 * @param buffer arbritrary length of data
 * @returns hexadecimal string
 */
exports.toHex = (buffer) => codechain_primitives_1.toHex(buffer);
/**
 * Gets data's 256 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 32 byte hexadecimal string
 */
exports.blake256 = (data) => codechain_primitives_1.blake256(data);
/**
 * Gets data's 160 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
exports.blake160 = (data) => codechain_primitives_1.blake160(data);
/**
 * Gets data's 128 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 16 byte hexadecimal string
 */
exports.blake128 = (data) => codechain_primitives_1.blake128(data);
/**
 * Gets data's 256 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 32 byte hexadecimal string
 */
exports.blake256WithKey = (data, key) => codechain_primitives_1.blake256WithKey(data, key);
/**
 * Gets data's 160 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 20 byte hexadecimal string
 */
exports.blake160WithKey = (data, key) => codechain_primitives_1.blake160WithKey(data, key);
/**
 * Gets data's 128 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 16 byte hexadecimal string
 */
exports.blake128WithKey = (data, key) => codechain_primitives_1.blake128WithKey(data, key);
/**
 * Gets data's 160 bit RIPEMD hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
exports.ripemd160 = (data) => codechain_primitives_1.ripemd160(data);
/**
 * @hidden
 */
exports.encodeSignatureTag = (tag) => {
    const { input, output } = tag;
    if (input !== "all" && input !== "single") {
        throw Error(`Expected the input of the tag to be either "all" or "single" but found ${input}`);
    }
    const inputMask = input === "all" ? 0b01 : 0b00;
    const outputMask = output === "all" ? 0b10 : 0b00;
    if (Array.isArray(output)) {
        // NOTE: Remove duplicates by using Set
        const encoded = encodeSignatureTagOutput(Array.from(new Set(output)).sort((a, b) => a - b));
        if (encoded.length >= 64) {
            throw Error(`The output length is too big`);
        }
        return Buffer.from([
            ...encoded,
            (encoded.length << 2) | outputMask | inputMask
        ]);
    }
    else if (output === "all") {
        return Buffer.from([outputMask | inputMask]);
    }
    else {
        throw Error(`Expected the output of the tag to be either string "all" or an array of number but found ${output}`);
    }
};
/**
 * @hidden
 */
const encodeSignatureTagOutput = (output) => {
    // NOTE: Assume all numbers are distinct and the array is sorted by increasing order.
    if (output[0] < 0) {
        throw Error(`Invalid signature tag. Out of range: ${output[0]}`);
    }
    else if (output[output.length - 1] > 503) {
        throw Error(`Invalid signature tag. Out of range: ${output[output.length - 1]}`);
    }
    let offset = 0;
    let byte = 0;
    const bytes = [];
    for (const index of output) {
        if (typeof index !== "number") {
            throw Error(`Expected an array of number but found ${index} at ${output.indexOf(index)}`);
        }
        if (index < offset + 8) {
            byte |= 1 << (index - offset);
        }
        else {
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
/**
 * Gets signature for message from private key.
 * @param message arbitrary length string
 * @param priv 32 byte hexadecimal string of private key
 * @returns r, s, v of ECDSA signature
 */
exports.signEcdsa = (message, priv) => codechain_primitives_1.signEcdsa(message, priv);
/**
 * Checks if the signature from signEcdsa is correct.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @param pub 64 byte hexadecimal string of public key
 * @returns if signature is valid, true. Else false.
 */
exports.verifyEcdsa = (message, signature, pub) => codechain_primitives_1.verifyEcdsa(message, signature, pub);
/**
 * Gets public key from the message and signature.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @returns 64 byte hexadecimal string public key
 */
exports.recoverEcdsa = (message, signature) => codechain_primitives_1.recoverEcdsa(message, signature);
/**
 * Generates a private key.
 * @returns 32 byte hexadecimal string of private key
 */
exports.generatePrivateKey = () => codechain_primitives_1.generatePrivateKey();
/**
 * Gets account id from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 20 byte hexadecimal string of account id
 */
exports.getAccountIdFromPrivate = (priv) => codechain_primitives_1.getAccountIdFromPrivate(priv);
/**
 * Gets account id from the given public key.
 * @param publicKey 64 byte hexadecimal string of uncompressed public key
 * @returns 20 byte hexadecimal string of account id
 */
exports.getAccountIdFromPublic = (publicKey) => codechain_primitives_1.getAccountIdFromPublic(publicKey);
/**
 * Gets public key from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 64 byte hexadecimal string of public key
 */
exports.getPublicFromPrivate = (priv) => codechain_primitives_1.getPublicFromPrivate(priv);
