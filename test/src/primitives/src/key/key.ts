import * as _ from "lodash";
import nacl = require("tweetnacl");
import { toArray, toHex } from "../utility";

/**
 * Generates a private key.
 * @returns 64 byte hexadecimal string of private key
 */
export const generatePrivateKey = (): string => {
    return toHex(Buffer.from(nacl.sign.keyPair().secretKey));
};

/**
 * Gets public key from private key.
 * @param priv 64 byte hexadecimal string of private key
 * @returns 32 byte hexadecimal string of public key
 */
export const getPublicFromPrivate = (priv: string): string => {
    return toHex(
        Buffer.from(nacl.sign.keyPair.fromSecretKey(toArray(priv)).publicKey)
    );
};
