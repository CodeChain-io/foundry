import * as _ from "lodash";

import { blake256, getPublicFromPrivate, signEcdsa } from "../../utils";
import { H256, PlatformAddress } from "../classes";
import { Text } from "../Text";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

const RLP = require("rlp");

export class Store extends Transaction {
    private content: string;
    private certifier: PlatformAddress;
    private signature: string;
    public constructor(
        params:
            | {
                  content: string;
                  certifier: PlatformAddress;
                  signature: string;
              }
            | {
                  content: string;
                  secret: H256;
              },
        networkId: NetworkId
    ) {
        super(networkId);

        if ("secret" in params) {
            const { content, secret } = params;
            this.content = content;
            this.certifier = PlatformAddress.fromPublic(
                getPublicFromPrivate(secret.value),
                { networkId }
            );
            const { r, s, v } = signEcdsa(
                blake256(RLP.encode(content)),
                secret.value
            );
            this.signature = `${_.padStart(r, 64, "0")}${_.padStart(
                s,
                64,
                "0"
            )}${_.padStart(v.toString(16), 2, "0")}`;
        } else {
            const { content, certifier } = params;
            let signature = params.signature;
            if (signature.startsWith("0x")) {
                signature = signature.substr(2);
            }
            this.content = content;
            this.certifier = certifier;
            this.signature = signature;
        }
    }

    public getText() {
        const { content, certifier } = this;
        return new Text({
            content,
            certifier
        });
    }

    public action(): string {
        return "store";
    }

    protected actionToEncodeObject(): any[] {
        const { content, certifier, signature } = this;
        return [
            8,
            content,
            certifier.getAccountId().toEncodeObject(),
            `0x${signature}`
        ];
    }

    protected actionToJSON(): any {
        const { content, certifier, signature } = this;
        return {
            content,
            certifier: certifier.value,
            signature: `0x${signature}`
        };
    }
}
