import * as _ from "lodash";
import * as RLP from "rlp";
import { blake256, getPublicFromPrivate, signEd25519 } from "../../utils";
import { Address, H256 } from "../classes";
import { Text } from "../Text";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface StoreActionJSON {
    content: string;
    certifier: string;
    signature: string;
}

export class Store extends Transaction {
    private content: string;
    private certifier: Address;
    private signature: string;
    public constructor(
        params:
            | {
                  content: string;
                  certifier: Address;
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
            this.certifier = Address.fromPublic(
                getPublicFromPrivate(secret.value),
                { networkId }
            );
            this.signature = signEd25519(
                blake256(RLP.encode(content)),
                secret.value
            );
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

    public type(): string {
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

    protected actionToJSON(): StoreActionJSON {
        const { content, certifier, signature } = this;
        return {
            content,
            certifier: certifier.value,
            signature: `0x${signature}`
        };
    }
}
