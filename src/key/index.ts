import { MemoryKeyStore } from "./MemoryKeyStore";
import { PubkeyAssetAgent } from "./PubkeyAssetAgent";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { PkhAssetAgent } from "./PkhAssetAgent";

/**
 * hidden
 */
export type KeyStore = MemoryKeyStore;

/**
 * @hidden
 */
export type AssetAgent = PubkeyAssetAgent | PkhAssetAgent;

export class Key {
    private pkAssetAgent: PubkeyAssetAgent;
    private pkhAssetAgent: PkhAssetAgent;

    constructor() {
        this.pkAssetAgent = new PubkeyAssetAgent({ keyStore: new MemoryKeyStore() });
        this.pkhAssetAgent = new PkhAssetAgent({ keyStore: new MemoryKeyStore() });
    }

    /**
     * Gets AssetAgent. AssetAgent manages addresses, scripts and keys for
     * locking/unlocking assets.
     * @returns AssetAgent
     */
    getAssetAgent(type: "nonStandardPay2PubKey" | "Pay2PubKeyHash" = "nonStandardPay2PubKey"): AssetAgent {
        if (type === "nonStandardPay2PubKey") {
            return this.pkAssetAgent;
        } else {
            return this.pkhAssetAgent;
        }
    }

    /**
     * Creates AssetTransferAddress for non-standard P2PK asset.
     * To use this address AssetScheme.mint() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createPubKeyAddress(): Promise<AssetTransferAddress> {
        return this.pkAssetAgent.createAddress();
    }

    /**
     * Creates AssetTransferAddress for the standard P2PKH asset.
     * To use this address, see AssetScheme.mint() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createPubKeyHashAddresss(): Promise<AssetTransferAddress> {
        return this.pkhAssetAgent.createAddress();
    }

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        PubkeyAssetAgent,
        MemoryKeyStore,
    };
}
