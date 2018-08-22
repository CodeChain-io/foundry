import * as _ from "lodash";

import { generatePrivateKey, getPublicFromPrivate, signEcdsa, blake256 } from "../utils";
import { KeyStore, KeyManagementAPI } from "./KeyStore";
import { H256 } from "../core/H256";

/**
 * @hidden
 */

class KeyManager implements KeyManagementAPI {
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

export class MemoryKeyStore implements KeyStore {
    platform = new KeyManager();
    asset = new KeyManager();

    pkh = {
        publicKeyMap: {} as { [publicKeyHash: string]: string },

        addPKH(params: { publicKey: string; }): Promise<string> {
            const publicKeyHash = H256.ensure(blake256(params.publicKey));
            this.publicKeyMap[publicKeyHash.value] = params.publicKey;
            return Promise.resolve(publicKeyHash.value);
        },

        getPK(params: { hash: string; }): Promise<string> {
            const publicKey = this.publicKeyMap[params.hash];
            return Promise.resolve(publicKey);
        }
    };
}
