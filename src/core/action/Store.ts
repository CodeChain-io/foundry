import { PlatformAddress } from "codechain-primitives";
import * as _ from "lodash";

import { blake256, getPublicFromPrivate, signEcdsa } from "../../utils";

import { H256 } from "../H256";
import { Text } from "../Text";
import { NetworkId } from "../types";

const RLP = require("rlp");

export class Store {
    public content: string;
    public certifier: PlatformAddress;
    public signature: string;

    constructor(
        params:
            | {
                  content: string;
                  certifier: PlatformAddress;
                  signature: string;
              }
            | {
                  content: string;
                  secret: H256;
                  networkId: NetworkId;
              }
    ) {
        if ("secret" in params) {
            const { content, secret, networkId } = params;
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

    public toEncodeObject(): any[] {
        const { content, certifier, signature } = this;
        return [
            8,
            content,
            certifier.getAccountId().toEncodeObject(),
            `0x${signature}`
        ];
    }

    public toJSON() {
        const { content, certifier, signature } = this;
        return {
            action: "store",
            content,
            certifier: certifier.value,
            signature: `0x${signature}`
        };
    }

    public getText() {
        const { content, certifier } = this;
        return new Text({
            content,
            certifier
        });
    }
}
