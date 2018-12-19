import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives/lib";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";
import { U64 } from "../U64";

export interface AssetMintOutputJSON {
    lockScriptHash: string;
    parameters: number[][];
    amount?: string | null;
}

export class AssetMintOutput {
    /**
     * Create an AssetMintOutput from an AssetMintOutput JSON object.
     * @param data An AssetMintOutput JSON object.
     * @returns An AssetMintOutput.
     */
    public static fromJSON(data: AssetMintOutputJSON) {
        const { lockScriptHash, parameters, amount } = data;
        return new this({
            lockScriptHash: H160.ensure(lockScriptHash),
            parameters: parameters.map(p => Buffer.from(p)),
            amount: amount == null ? null : U64.ensure(amount)
        });
    }

    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly amount?: U64 | null;

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
                  amount?: U64 | null;
              }
            | {
                  recipient: AssetTransferAddress;
                  amount?: U64 | null;
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
    public toJSON(): AssetMintOutputJSON {
        return {
            lockScriptHash: this.lockScriptHash.toJSON(),
            parameters: this.parameters.map(p => [...p]),
            amount: this.amount == null ? undefined : this.amount.toJSON()
        };
    }
}
