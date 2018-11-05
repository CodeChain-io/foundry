import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives/lib";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";
import { U256 } from "../U256";

export class AssetMintOutput {
    /**
     * Create an AssetMintOutput from an AssetMintOutput JSON object.
     * @param data An AssetMintOutput JSON object.
     * @returns An AssetMintOutput.
     */
    public static fromJSON(data: {
        lockScriptHash: string;
        parameters: Buffer[];
        amount?: string | null;
    }) {
        const { lockScriptHash, parameters, amount } = data;
        return new this({
            lockScriptHash: H160.ensure(lockScriptHash),
            parameters: parameters.map(p => Buffer.from(p)),
            amount: amount == null ? null : U256.ensure(amount)
        });
    }

    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly amount?: U256 | null;

    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.amount Asset amount of the output.
     */
    constructor(
        data:
            | {
                  lockScriptHash: H160;
                  parameters: Buffer[];
                  amount?: U256 | null;
              }
            | {
                  recipient: AssetTransferAddress;
                  amount?: U256 | null;
              }
    ) {
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
            if ("pubkeys" in payload) {
                throw Error(`Multisig payload is not supported yet`);
            }
            switch (type) {
                case 0x00: // LOCK_SCRIPT_HASH ONLY
                    this.lockScriptHash = payload;
                    this.parameters = [];
                    break;
                case 0x01: // P2PKH
                    this.lockScriptHash = P2PKH.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                case 0x02: // P2PKHBurn
                    this.lockScriptHash = P2PKHBurn.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                default:
                    throw Error(
                        `Unexpected type of AssetTransferAddress: ${type}, ${
                            data.recipient
                        }`
                    );
            }
        } else {
            const { lockScriptHash, parameters } = data;
            this.lockScriptHash = lockScriptHash;
            this.parameters = parameters;
        }
        this.amount = data.amount;
    }

    /**
     * Convert to an AssetMintOutput JSON object.
     * @returns An AssetMintOutput JSON object.
     */
    public toJSON() {
        return {
            lockScriptHash: this.lockScriptHash.value,
            parameters: this.parameters.map(p => [...p]),
            amount:
                this.amount == null ? undefined : this.amount.toEncodeObject()
        };
    }
}
