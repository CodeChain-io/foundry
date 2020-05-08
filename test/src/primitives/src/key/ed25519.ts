import * as _ from "lodash";
import nacl = require("tweetnacl");
import { toArray, toHex } from "../utility";

export type Ed25519Signature = string;

/**
 * Gets EdDSA(Ed25519) signature for message from private key.
 * @param message 32 byte hexadecimal string
 * @param priv 64 byte hexadecimal string of private key
 * @returns 64 byte hexadecimal string of Ed25519 signature
 */
export const signEd25519 = (
    message: string,
    priv: string
): Ed25519Signature => {
    return toHex(
        Buffer.from(nacl.sign.detached(toArray(message), toArray(priv)))
    );
};

/**
 * Checks if the signature from signEd25519 is valid.
 * @param message 32 byte hexadecimal string
 * @param signature 64 byte hexadecimal string of Ed25519 signature
 * @param pub 32 byte hexadecimal string of public key
 * @returns if signature is valid, true. Else false.
 */
export const verifyEd25519 = (
    message: string,
    signature: Ed25519Signature,
    pub: string
): boolean => {
    return nacl.sign.detached.verify(
        toArray(message),
        toArray(signature),
        toArray(pub)
    );
};
