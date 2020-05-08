import { toArray, toHex } from "../utility";

/**
 * @hidden
 */
const nacl = require("tweetnacl");

export type X25519Public = string;
export type X25519Private = string;

/**
 * Gets an ECDH session key for encryption and decryption between two parties
 * @param otherPublic 32 byte hexadecimal string of the other side public key
 * @param myPrivate 32 byte hexadecimal string of my side private key
 * @returns 32 byte hexadecimal string of the shared secret
 */
export const exchange = (
    otherPublic: X25519Public,
    myPrivate: X25519Private
): string => {
    const groupElement = toArray(otherPublic);
    const scalar = toArray(myPrivate);
    const sharedSecret = nacl.scalarMult(scalar, groupElement);
    return toHex(sharedSecret);
};

/**
 * Gets the X25519 public key(on Curve25519) for a private key
 * @param x25519Private 32 byte hexadecimal string of a secret key
 * @returns 32 byte hexadecimal string of the public key
 */
export const x25519GetPublicFromPrivate = (x25519Private: string): string => {
    const scalar = toArray(x25519Private);
    const x25519Public = nacl.scalarMult.base(scalar);
    return toHex(x25519Public);
};
