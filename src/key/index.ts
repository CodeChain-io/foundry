import { MemoryKeyStore } from "./MemoryKeyStore";
import { PubkeyAssetAgent } from "./PubkeyAssetAgent";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";

/**
 * hidden
 */
export type KeyStore = MemoryKeyStore;

/**
 * @hidden
 */
export type AssetAgent = PubkeyAssetAgent;

export class Key {
    private assetAgent: AssetAgent;

    constructor() {
        this.assetAgent = new PubkeyAssetAgent({ keyStore: new MemoryKeyStore() });
    }

    /**
     * Gets AssetAgent. AssetAgent manages addresses, scripts and keys for
     * locking/unlocking assets.
     * @returns AssetAgent
     */
    getAssetAgent(): AssetAgent {
        return this.assetAgent;
    }

    /**
     * Creates AssetTransferAddress. AssetTransferAddress is used to receive assets.
     * See AssetScheme.mint() or Asset.transfer().
     * @returns AssetTransferAddress
     */
    createAssetTransferAddress(): Promise<AssetTransferAddress> {
        return this.assetAgent.createAddress();
    }

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        PubkeyAssetAgent,
        MemoryKeyStore,
    };
}
