import { Buffer } from "buffer";
import { AssetTransferAddress, H160, U64 } from "codechain-primitives";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";

export interface AssetMintOutputJSON {
    lockScriptHash: string;
    parameters: string[];
    supply: string;
}

export class AssetMintOutput {
    /**
     * Create an AssetMintOutput from an AssetMintOutput JSON object.
     * @param data An AssetMintOutput JSON object.
     * @returns An AssetMintOutput.
     */
    public static fromJSON(data: AssetMintOutputJSON) {
        const { lockScriptHash, parameters, supply } = data;
        return new this({
            lockScriptHash: H160.ensure(lockScriptHash),
            parameters: parameters.map(p => Buffer.from(p, "hex")),
            supply: U64.ensure(supply)
        });
    }

    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly supply: U64;

    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.supply Asset supply of the output.
     */
    constructor(
        data:
            | {
                  lockScriptHash: H160;
                  parameters: Buffer[];
                  supply: U64;
              }
            | {
                  recipient: AssetTransferAddress;
                  supply: U64;
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
        this.supply = data.supply;
    }

    /**
     * Convert to an AssetMintOutput JSON object.
     * @returns An AssetMintOutput JSON object.
     */
    public toJSON(): AssetMintOutputJSON {
        return {
            lockScriptHash: this.lockScriptHash.toJSON(),
            parameters: this.parameters.map((p: Buffer) => p.toString("hex")),
            supply: this.supply.toJSON()
        };
    }
}
