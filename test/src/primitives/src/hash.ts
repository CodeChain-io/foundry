import { enc, RIPEMD160 } from "crypto-js";

import { toHex } from "./utility";

/**
 * @hidden
 */
const blake = require("blakejs");

/**
 * Gets data's 256 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 32 byte hexadecimal string
 */
export const blake256 = (data: Buffer | string): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(32, null);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 256 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 32 byte hexadecimal string
 */
export const blake256WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(32, key);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 160 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export const blake160 = (data: Buffer | string): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(20, null);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 160 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 20 byte hexadecimal string
 */
export const blake160WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(20, key);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 128 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 16 byte hexadecimal string
 */
export const blake128 = (data: Buffer | string): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(16, null);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 128 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 16 byte hexadecimal string
 */
export const blake128WithKey = (
    data: Buffer | string,
    key: Uint8Array
): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(16, key);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 160 bit RIPEMD hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export const ripemd160 = (data: Buffer | string): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    return RIPEMD160(enc.Hex.parse(data.toString("hex"))).toString();
};
