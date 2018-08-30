import { Buffer } from "buffer";

import { H256 } from "../core/H256";
import { Script } from "../core/Script";
import {
    AssetTransferTransaction,
    TransactionBurnSigner
} from "../core/transaction/AssetTransferTransaction";
import { blake256 } from "../utils";

import { AssetTransferAddress } from "./AssetTransferAddress";
import { KeyStore } from "./KeyStore";

type NetworkId = string;

export class P2PKHBurn implements TransactionBurnSigner {
    public static getLockScript(): Buffer {
        const { COPY, BLAKE256, EQ, JZ, CHKSIG, BURN } = Script.Opcode;
        return Buffer.from([
            COPY,
            0x01,
            BLAKE256,
            EQ,
            JZ,
            0xff,
            CHKSIG,
            JZ,
            0xff,
            BURN
        ]);
    }

    public static getLockScriptHash(): H256 {
        return new H256(
            "41a872156efc1dbd45a85b49896e9349a4e8f3fb1b8f3ed38d5e13ef675bcd5a"
        );
    }
    private keyStore: KeyStore;
    private networkId: NetworkId;

    constructor(params: { keyStore: KeyStore; networkId: NetworkId }) {
        const { keyStore, networkId } = params;
        this.keyStore = keyStore;
        this.networkId = networkId;
    }

    public async createAddress(): Promise<AssetTransferAddress> {
        const publicKey = await this.keyStore.asset.createKey();
        const hash = H256.ensure(blake256(publicKey));
        await this.keyStore.mapping.add({ key: hash.value, value: publicKey });
        return AssetTransferAddress.fromTypeAndPayload(2, hash, {
            networkId: this.networkId
        });
    }

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
        console.log(lockScriptHash);
        console.log(P2PKHBurn.getLockScriptHash().value);
        if (lockScriptHash.value !== P2PKHBurn.getLockScriptHash().value) {
            throw Error("Unexpected lock script hash");
        }
        if (parameters.length !== 1) {
            throw Error("Unexpected length of parameters");
        }
        const publicKeyHash = Buffer.from(parameters[0]).toString("hex");
        const publicKey = await this.keyStore.mapping.get({
            key: publicKeyHash
        });
        if (!publicKey) {
            throw Error(
                `Unable to get original key from the given public key hash: ${publicKeyHash}`
            );
        }

        transaction.burns[index].setLockScript(P2PKHBurn.getLockScript());
        transaction.burns[index].setUnlockScript(
            await this.getUnlockScript(
                publicKey,
                transaction.hashWithoutScript(),
                { passphrase }
            )
        );
    }

    private async getUnlockScript(
        publicKey: string,
        txhash: H256,
        options: { passphrase?: string } = {}
    ): Promise<Buffer> {
        const { passphrase } = options;
        const signature = await this.keyStore.asset.sign({
            publicKey,
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
