/**
 * @hidden
 */
const blake = require("blakejs");
/**
 * @hidden
 */
const ripemd = require("ripemd160");
/**
 * @hidden
 */
const EC = require("elliptic").ec;
/**
 * @hidden
 */
const secp256k1 = new EC("secp256k1");

const toHexByte = (byte: number) => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16);
/**
 * Converts buffer to hexadecimal string.
 * @param buffer arbritrary length of data
 * @returns hexadecimal string
 */
export const toHex = (buffer: Buffer): string => {
    return Array.from(buffer).map(toHexByte).join("");
};

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
 * Gets data's 208 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 26 byte hexadecimal string
 */
export const blake208 = (data: Buffer | string): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(26, null);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 256 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 32 byte hexadecimal string
 */
export const blake256WithKey = (data: Buffer | string, key: Uint8Array): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(32, key);
    blake.blake2bUpdate(context, data);
    return toHex(blake.blake2bFinal(context));
};

/**
 * Gets data's 208 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 26 byte hexadecimal string
 */
export const blake208WithKey = (data: Buffer | string, key: Uint8Array): string => {
    if (!(data instanceof Buffer)) {
        data = Buffer.from(data, "hex");
    }
    const context = blake.blake2bInit(26, key);
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
    return new ripemd().update(data).digest("hex");
};

export type EcdsaSignature = {
    r: string;
    s: string;
    v: number;
};

/**
 * Gets signature for message from private key.
 * @param message arbitrary length string
 * @param priv 32 byte hexadecimal string of private key
 * @returns r, s, v of ECDSA signature
 */
export const signEcdsa = (message: string, priv: string): EcdsaSignature => {
    const key = secp256k1.keyFromPrivate(priv);
    const { r, s, recoveryParam: v } = key.sign(message, { "canonical": true });
    return {
        r: r.toString("hex"),
        s: s.toString("hex"),
        v
    };
};

/**
 * Checks if the signature from signEcdsa is correct.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @param pub 64 byte hexadecimal string of public key
 * @returns if signature is valid, true. Else false.
 */
export const verifyEcdsa = (message: string, signature: EcdsaSignature, pub: string): boolean => {
    const key = secp256k1.keyFromPublic("04" + pub, "hex");
    return key.verify(message, signature);
};

/**
 * Gets public key from the message and signature.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @returns 64 byte hexadecimal string public key
 */
export const recoverEcdsa = (message: string, signature: EcdsaSignature): string => {
    return secp256k1.recoverPubKey(
        secp256k1.keyFromPrivate(message, "hex").getPrivate().toString(10),
        signature,
        signature.v
    ).encode("hex").slice(2);
};

/**
 * Generates a private key.
 * @returns 32 byte hexadecimal string of private key
 */
export const generatePrivateKey = (): string => {
    return secp256k1.genKeyPair().priv.toString("hex");
};

/**
 * Gets account id from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 20 byte hexadecimal string of account id
 */
export const getAccountIdFromPrivate = (priv: string): string => {
    const publicKey = getPublicFromPrivate(priv);
    return getAccountIdFromPublic(publicKey);
};

/**
 * Gets account id from the given public key.
 * @param publicKey 64 byte hexadecimal string of uncompressed public key
 * @returns 20 byte hexadecimal string of account id
 */
export const getAccountIdFromPublic = (publicKey: string): string => {
    return ripemd160(blake256(publicKey));
};

/**
 * Gets public key from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 64 byte hexadecimal string of public key
 */
export const getPublicFromPrivate = (priv: string): string => {
    const key = secp256k1.keyFromPrivate(priv);
    // Remove prefix "04" which represents it's uncompressed form.
    return key.getPublic().encode("hex").slice(2);
};
