import * as _ from "lodash";

import { signEcdsa } from "../../utils";
import { H256 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export class Remove extends Transaction {
    private readonly _hash: H256;
    private readonly signature: string;

    public constructor(
        params:
            | {
                  hash: H256;
                  signature: string;
              }
            | {
                  hash: H256;
                  secret: H256;
              },
        networkId: NetworkId
    ) {
        super(networkId);

        if ("secret" in params) {
            const { hash, secret } = params;
            this._hash = hash;
            this.signature = signEcdsa(hash.value, secret.value);
        } else {
            let signature = params.signature;
            if (signature.startsWith("0x")) {
                signature = signature.substr(2);
            }
            this._hash = params.hash;
            this.signature = signature;
        }
    }

    public type(): string {
        return "remove";
    }

    protected actionToEncodeObject(): any[] {
        const { _hash, signature } = this;
        return [9, _hash.toEncodeObject(), `0x${signature}`];
    }

    protected actionToJSON(): any {
        const { _hash, signature } = this;
        return {
            hash: _hash.toEncodeObject(),
            signature: `0x${signature}`
        };
    }
}
