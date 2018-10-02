import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives";

import { H256 } from "../core/H256";
import { Script } from "../core/Script";
import {
    AssetTransferTransaction,
    TransactionBurnSigner
} from "../core/transaction/AssetTransferTransaction";
import { NetworkId } from "../core/types";

import { KeyStore } from "./KeyStore";

export class P2PKHBurn implements TransactionBurnSigner {
    public static getLockScript(): Buffer {
        const { COPY, BLAKE160, EQ, JZ, CHKSIG, BURN } = Script.Opcode;
        return Buffer.from([
            COPY,
            0x01,
            BLAKE160,
            EQ,
            JZ,
            0xff,
            CHKSIG,
            JZ,
            0xff,
            BURN
        ]);
    }

    public static getLockScriptHash(): H160 {
        return new H160("37572bdcc22d39a59c0d12d301f6271ba3fdd451");
    }
    private keyStore: KeyStore;
    private networkId: NetworkId;

    constructor(params: { keyStore: KeyStore; networkId: NetworkId }) {
        const { keyStore, networkId } = params;
        this.keyStore = keyStore;
        this.networkId = networkId;
    }

    public async createAddress(
        options: { passphrase?: string } = {}
    ): Promise<AssetTransferAddress> {
        const { passphrase } = options;
        const hash = await this.keyStore.asset.createKey({ passphrase });
        return AssetTransferAddress.fromTypeAndPayload(2, hash, {
            networkId: this.networkId
        });
    }

    /**
     * @deprecated Use signTransactionBurn
     */
    public async signBurn(
        transaction: AssetTransferTransaction,
        index: number,
        options: { passphrase?: string } = {}
    ): Promise<void> {
        const { passphrase } = options;
        if (index >= transaction.burns.length) {
            throw Error("Invalid input index");
        }
        const { lockScriptHash, parameters } = transaction.burns[index].prevOut;
        if (lockScriptHash === undefined || parameters === undefined) {
            throw Error("Invalid transaction input");
        }
        if (lockScriptHash.value !== P2PKHBurn.getLockScriptHash().value) {
            throw Error("Unexpected lock script hash");
        }
        if (parameters.length !== 1) {
            throw Error("Unexpected length of parameters");
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");

        transaction.burns[index].setLockScript(P2PKHBurn.getLockScript());
        transaction.burns[index].setUnlockScript(
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
        const publicKey = await this.keyStore.asset.getPublicKey({
            key: publicKeyHash
        });
        if (!publicKey) {
            throw Error(
                `Unable to get original key from the given public key hash: ${publicKeyHash}`
            );
        }
        const signature = await this.keyStore.asset.sign({
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
