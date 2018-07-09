import { MemoryKeyStore } from "../signer/MemoryKeyStore";
import { PubkeyAssetAgent } from "../signer/PubkeyAssetAgent";
import { AssetAgent } from "../primitives/Asset";
import { AssetTransferAddress } from "../AssetTransferAddress";
import { PlatformAddress } from "../PlatformAddress";

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
