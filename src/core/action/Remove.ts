import * as _ from "lodash";

import { signEcdsa } from "../../utils";

import { H256 } from "../H256";

export class Remove {
    public hash: H256;
    public signature: string;

    constructor(
        params:
            | {
                  hash: H256;
                  signature: string;
              }
            | {
                  hash: H256;
                  secret: H256;
              }
    ) {
        if ("secret" in params) {
            const { hash, secret } = params;
            this.hash = hash;
            const { r, s, v } = signEcdsa(hash.value, secret.value);
            this.signature = `${_.padStart(r, 64, "0")}${_.padStart(
                s,
                64,
                "0"
            )}${_.padStart(v.toString(16), 2, "0")}`;
        } else {
            let signature = params.signature;
            if (signature.startsWith("0x")) {
                signature = signature.substr(2);
            }
            this.hash = params.hash;
            this.signature = signature;
        }
    }

    public toEncodeObject(): any[] {
        const { hash, signature } = this;
        return [9, hash.toEncodeObject(), `0x${signature}`];
    }

    public toJSON() {
        const { hash, signature } = this;
        return {
            action: "remove",
            hash: hash.toEncodeObject(),
            signature: `0x${signature}`
        };
    }
}
