import { Buffer } from "buffer";
import { AssetTransferAddress, H160, H256 } from "codechain-primitives";

import { Script } from "../core/Script";
import { NetworkId } from "../core/types";
import { encodeSignatureTag, SignatureTag } from "../utils";

import { KeyStore } from "./KeyStore";

export class P2PKHBurn {
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

    public async createUnlockScript(
        publicKeyHash: string,
        txhash: H256,
        options: { passphrase?: string; signatureTag?: SignatureTag } = {}
    ): Promise<Buffer> {
        const {
            passphrase,
            signatureTag = { input: "all", output: "all" } as SignatureTag
        } = options;
        const publicKey = await this.keyStore.asset.getPublicKey({
            key: publicKeyHash,
            passphrase
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
        const encodedTag = encodeSignatureTag(signatureTag);
        const { PUSHB } = Script.Opcode;
        return Buffer.from([
            PUSHB,
            65,
            ...Buffer.from(signature, "hex"),
            PUSHB,
            encodedTag.byteLength,
            ...encodedTag,
            PUSHB,
            64,
            ...Buffer.from(publicKey, "hex")
        ]);
    }
}
