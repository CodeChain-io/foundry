import * as _ from "lodash";

import { generatePrivateKey, getPublicFromPrivate, signEcdsa } from "../utils";

/**
 * A simple key store for testing purpose.
 */
export class MemoryRawKeyStore {
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

    sign(params: { publicKey: string, message: string, passphrase?: string }): Promise<{ r: string, s: string, v: number }> {
        const { publicKey, message, passphrase = "" } = params;
        if (passphrase !== this.passphraseMap[publicKey]) {
            return Promise.reject("The passphrase does not match");
        }
        return Promise.resolve(signEcdsa(message, this.privateKeyMap[publicKey]));
    }
}
