/// <reference types="node" />
/**
 * Converts buffer to hexadecimal string.
 * @param buffer arbritrary length of data
 * @returns hexadecimal string
 */
export declare const toHex: (buffer: Buffer) => string;
/**
 * Gets data's 256 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 32 byte hexadecimal string
 */
export declare const blake256: (data: string | Buffer) => string;
/**
 * Gets data's 160 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export declare const blake160: (data: string | Buffer) => string;
/**
 * Gets data's 128 bit blake hash.
 * @param data buffer or hexadecimal string
 * @returns 16 byte hexadecimal string
 */
export declare const blake128: (data: string | Buffer) => string;
/**
 * Gets data's 256 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 32 byte hexadecimal string
 */
export declare const blake256WithKey: (data: string | Buffer, key: Uint8Array) => string;
/**
 * Gets data's 160 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 20 byte hexadecimal string
 */
export declare const blake160WithKey: (data: string | Buffer, key: Uint8Array) => string;
/**
 * Gets data's 128 bit blake hash by using the key.
 * @param data buffer or hexadecimal string
 * @param key
 * @returns 16 byte hexadecimal string
 */
export declare const blake128WithKey: (data: string | Buffer, key: Uint8Array) => string;
/**
 * Gets data's 160 bit RIPEMD hash.
 * @param data buffer or hexadecimal string
 * @returns 20 byte hexadecimal string
 */
export declare const ripemd160: (data: string | Buffer) => string;
export interface SignatureTag {
    input: "all" | "single";
    output: "all" | number[];
}
/**
 * @hidden
 */
export declare const encodeSignatureTag: (tag: SignatureTag) => Buffer;
export interface EcdsaSignature {
    r: string;
    s: string;
    v: number;
}
/**
 * Gets signature for message from private key.
 * @param message arbitrary length string
 * @param priv 32 byte hexadecimal string of private key
 * @returns r, s, v of ECDSA signature
 */
export declare const signEcdsa: (message: string, priv: string) => EcdsaSignature;
/**
 * Checks if the signature from signEcdsa is correct.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @param pub 64 byte hexadecimal string of public key
 * @returns if signature is valid, true. Else false.
 */
export declare const verifyEcdsa: (message: string, signature: EcdsaSignature, pub: string) => boolean;
/**
 * Gets public key from the message and signature.
 * @param message arbitrary length string
 * @param signature r, s, v of ECDSA signature
 * @returns 64 byte hexadecimal string public key
 */
export declare const recoverEcdsa: (message: string, signature: EcdsaSignature) => string;
/**
 * Generates a private key.
 * @returns 32 byte hexadecimal string of private key
 */
export declare const generatePrivateKey: () => string;
/**
 * Gets account id from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 20 byte hexadecimal string of account id
 */
export declare const getAccountIdFromPrivate: (priv: string) => string;
/**
 * Gets account id from the given public key.
 * @param publicKey 64 byte hexadecimal string of uncompressed public key
 * @returns 20 byte hexadecimal string of account id
 */
export declare const getAccountIdFromPublic: (publicKey: string) => string;
/**
 * Gets public key from private key.
 * @param priv 32 byte hexadecimal string of private key
 * @returns 64 byte hexadecimal string of public key
 */
export declare const getPublicFromPrivate: (priv: string) => string;
