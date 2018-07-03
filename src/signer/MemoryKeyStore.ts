import * as _ from "lodash";

import { generatePrivateKey, privateKeyToPublic, signEcdsa } from "../utils";

export class MemoryKeyStore {
    private privateKeyMap: { [publicKey: string]: string } = {};

    getKeyList(): Promise<string[]> {
        return Promise.resolve(_.keys(this.privateKeyMap));
    }

    createKey(): Promise<string> {
        const privateKey = generatePrivateKey();
        const publicKey = privateKeyToPublic(privateKey);
        this.privateKeyMap[publicKey] = privateKey;
        return Promise.resolve(publicKey);
    }

    removeKey(publicKey: string): Promise<boolean> {
        if (this.privateKeyMap[publicKey]) {
            delete this.privateKeyMap[publicKey];
            return Promise.resolve(true);
        } else {
            return Promise.resolve(false);
        }
    }

    sign(params: { publicKey: string, message: string }): Promise<{ r: string, s: string, v: number }> {
        const { publicKey, message } = params;
        return Promise.resolve(signEcdsa(message, this.privateKeyMap[publicKey]));
    }
}
