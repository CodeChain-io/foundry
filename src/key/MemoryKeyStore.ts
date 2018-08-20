import * as _ from "lodash";

import { generatePrivateKey, getPublicFromPrivate, signEcdsa } from "../utils";
import { KeyStore } from "./KeyStore";

/**
 * A simple key store for testing purpose.
 */
export class MemoryKeyStore implements KeyStore {
    private privateKeyMap: { [publicKey: string]: string } = {};
    private passphraseMap: { [publicKey: string]: string } = {};

    getKeyList(): Promise<string[]> {
        return Promise.resolve(_.keys(this.privateKeyMap));
    }

    createKey(params: { passphrase?: string } = {}): Promise<string> {
        const privateKey = generatePrivateKey();
        const publicKey = getPublicFromPrivate(privateKey);
        this.privateKeyMap[publicKey] = privateKey;
        this.passphraseMap[publicKey] = params.passphrase || "";
        return Promise.resolve(publicKey);
    }

    removeKey(params: { publicKey: string, passphrase?: string }): Promise<boolean> {
        const { publicKey, passphrase = "" } = params;
        if (this.privateKeyMap[publicKey] && this.passphraseMap[publicKey] === passphrase) {
            delete this.privateKeyMap[publicKey];
            return Promise.resolve(true);
        } else {
            return Promise.resolve(false);
        }
    }

    sign(params: { publicKey: string, message: string, passphrase?: string }): Promise<string> {
        const { publicKey, message, passphrase = "" } = params;
        if (passphrase !== this.passphraseMap[publicKey]) {
            return Promise.reject("The passphrase does not match");
        }
        const { r, s, v } = signEcdsa(message, this.privateKeyMap[publicKey]);
        const sig = `${_.padStart(r, 64, "0")}${_.padStart(s, 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
        return Promise.resolve(sig);
    }
}
