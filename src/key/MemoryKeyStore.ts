import * as _ from "lodash";

import { generatePrivateKey, getPublicFromPrivate, signEcdsa } from "../utils";
import { KeyManagementAPI, KeyStore } from "./KeyStore";

/**
 * @hidden
 */

class KeyManager implements KeyManagementAPI {
    private privateKeyMap: { [publicKey: string]: string } = {};
    private passphraseMap: { [publicKey: string]: string } = {};

    public getKeyList(): Promise<string[]> {
        return Promise.resolve(_.keys(this.privateKeyMap));
    }

    public createKey(params: { passphrase?: string } = {}): Promise<string> {
        const privateKey = generatePrivateKey();
        const publicKey = getPublicFromPrivate(privateKey);
        this.privateKeyMap[publicKey] = privateKey;
        this.passphraseMap[publicKey] = params.passphrase || "";
        return Promise.resolve(publicKey);
    }

    public removeKey(params: {
        publicKey: string;
        passphrase?: string;
    }): Promise<boolean> {
        const { publicKey, passphrase = "" } = params;
        if (
            this.privateKeyMap[publicKey] &&
            this.passphraseMap[publicKey] === passphrase
        ) {
            delete this.privateKeyMap[publicKey];
            return Promise.resolve(true);
        } else {
            return Promise.resolve(false);
        }
    }

    public sign(params: {
        publicKey: string;
        message: string;
        passphrase?: string;
    }): Promise<string> {
        const { publicKey, message, passphrase = "" } = params;
        if (passphrase !== this.passphraseMap[publicKey]) {
            return Promise.reject("The passphrase does not match");
        }
        const { r, s, v } = signEcdsa(message, this.privateKeyMap[publicKey]);
        const sig = `${_.padStart(r, 64, "0")}${_.padStart(
            s,
            64,
            "0"
        )}${_.padStart(v.toString(16), 2, "0")}`;
        return Promise.resolve(sig);
    }
}

export class MemoryKeyStore implements KeyStore {
    public platform = new KeyManager();
    public asset = new KeyManager();

    public mapping = {
        keyMap: {} as { [publicKeyHash: string]: string },

        add(params: { key: string; value: string }): Promise<void> {
            this.keyMap[params.key] = params.value;
            return Promise.resolve();
        },

        get(params: { key: string }): Promise<string> {
            const value = this.keyMap[params.key];
            return Promise.resolve(value);
        }
    };
}
