import { Buffer } from "buffer";
import { AssetTransferAddress } from "codechain-primitives";

import { H256 } from "../core/H256";
import { Script } from "../core/Script";
import {
    AssetTransferTransaction,
    TransactionInputSigner
} from "../core/transaction/AssetTransferTransaction";
import { NetworkId } from "../core/types";

import { KeyStore } from "./KeyStore";

/**
 * AssetAgent which supports P2PKH(Pay to Public Key Hash).
 */
export class P2PKH implements TransactionInputSigner {
    public static getLockScript(): Buffer {
        const { COPY, BLAKE160, EQ, JZ, CHKSIG } = Script.Opcode;
        return Buffer.from([COPY, 0x01, BLAKE160, EQ, JZ, 0xff, CHKSIG]);
    }

    public static getLockScriptHash(): H256 {
        return new H256(
            "f42a65ea518ba236c08b261c34af0521fa3cd1aa505e1c18980919cb8945f8f3"
        );
    }
    private rawKeyStore: KeyStore;
    private networkId: NetworkId;

    // FIXME: rename keyStore to rawKeyStore
    constructor(params: { keyStore: KeyStore; networkId: NetworkId }) {
        const { keyStore, networkId } = params;
        this.rawKeyStore = keyStore;
        this.networkId = networkId;
    }

    public async createAddress(
        options: { passphrase?: string } = {}
    ): Promise<AssetTransferAddress> {
        const { passphrase } = options;
        const hash = await this.rawKeyStore.asset.createKey({ passphrase });
        return AssetTransferAddress.fromTypeAndPayload(1, hash, {
            networkId: this.networkId
        });
    }

    /**
     * @deprecated Use signTransactionInput
     */
    public async signInput(
        transaction: AssetTransferTransaction,
        index: number,
        options: { passphrase?: string } = {}
    ): Promise<void> {
        const { passphrase } = options;
        if (index >= transaction.inputs.length) {
            throw Error("Invalid input index");
        }
        const { lockScriptHash, parameters } = transaction.inputs[
            index
        ].prevOut;
        if (lockScriptHash === undefined || parameters === undefined) {
            throw Error("Invalid transaction input");
        }
        if (lockScriptHash.value !== P2PKH.getLockScriptHash().value) {
            throw Error("Unexpected lock script hash");
        }
        if (parameters.length !== 1) {
            throw Error("Unexpected length of parameters");
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");

        transaction.inputs[index].setLockScript(P2PKH.getLockScript());
        transaction.inputs[index].setUnlockScript(
            await this.createUnlockScript(
                publicKeyHash,
                transaction.hashWithoutScript(),
                { passphrase }
            )
        );
    }

    public async createUnlockScript(
        publicKeyHash: string,
        txhash: H256,
        options: { passphrase?: string } = {}
    ): Promise<Buffer> {
        const { passphrase } = options;
        const publicKey = await this.rawKeyStore.asset.getPublicKey({
            key: publicKeyHash
        });
        if (!publicKey) {
            throw Error(
                `Unable to get original key from the given public key hash: ${publicKeyHash}`
            );
        }
        const signature = await this.rawKeyStore.asset.sign({
            key: publicKeyHash,
            message: txhash.value,
            passphrase
        });
        const { PUSHB } = Script.Opcode;
        return Buffer.from([
            PUSHB,
            65,
            ...Buffer.from(signature, "hex"),
            PUSHB,
            64,
            ...Buffer.from(publicKey, "hex")
        ]);
    }
}
