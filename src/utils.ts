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
/**
 * @hidden
 */
const RLP = require("rlp");

const toHexByte = (byte: number) => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16);

const toHex = (buffer: Buffer): string => {
    return Array.from(buffer).map(toHexByte).join("");
};

export const blake256 = (buffer: Buffer | string): string => {
    if (!(buffer instanceof Buffer)) {
        buffer = Buffer.from(buffer, "hex");
    }
    const context = blake.blake2bInit(32, null);
    blake.blake2bUpdate(context, buffer);
    return toHex(blake.blake2bFinal(context));
};

export const blake256WithKey = (buffer: Buffer | string, key: Uint8Array): string => {
    if (!(buffer instanceof Buffer)) {
        buffer = Buffer.from(buffer, "hex");
    }
    const context = blake.blake2bInit(32, key);
    blake.blake2bUpdate(context, buffer);
    return toHex(blake.blake2bFinal(context));
};

export const ripemd160 = (buffer: Buffer | string): string => {
    if (!(buffer instanceof Buffer)) {
        buffer = Buffer.from(buffer, "hex");
    }
    return new ripemd().update(buffer).digest("hex");
};

export type ECDSASignature = {
    r: string;
    s: string;
    v: number;
};

export const signEcdsa = (message: string, priv: string): ECDSASignature => {
    const key = secp256k1.keyFromPrivate(priv);
    const { r, s, recoveryParam: v } = key.sign(message, { "canonical": true });
    return {
        r: r.toString("hex"),
        s: s.toString("hex"),
        v
    };
};

export const verifyEcdsa = (message: string, signature: ECDSASignature, pub: string) => {
    const key = secp256k1.keyFromPublic("04" + pub, "hex");
    return key.verify(message, signature);
};

export const recoverPublic = (message: string, signature: ECDSASignature) => {
    return secp256k1.recoverPubKey(
        secp256k1.keyFromPrivate(message, "hex").getPrivate().toString(10),
        signature,
        signature.v
    ).encode("hex").slice(2);
};

export const privateKeyToAddress = (priv: string) => {
    const key = secp256k1.keyFromPrivate(priv);
    return ripemd160(blake256(key.getPublic().encode("hex").slice(2)));
};

export const privateKeyToPublic = (priv: string): string => {
    const key = secp256k1.keyFromPrivate(priv);
    // Remove prefix "04" which represents it's uncompressed form.
    return key.getPublic().encode("hex").slice(2);
};
