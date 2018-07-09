import { MemoryKeyStore } from "./MemoryKeyStore";
import { PubkeyAssetAgent } from "./PubkeyAssetAgent";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
// FIXME:
import { AssetAgent } from "../core/Asset";

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

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        PubkeyAssetAgent,
        MemoryKeyStore,
    };
}
