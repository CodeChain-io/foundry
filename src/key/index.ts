import { Rpc } from "../rpc";
import { Parcel, SignedParcel, H160 } from "../core/classes";

import { MemoryKeyStore } from "./MemoryKeyStore";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { P2PKH } from "./P2PKH";

type KeyStore = MemoryKeyStore;

export class Key {
    private rpc: Rpc;

    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Creates key store which is non-persistent. Do not use in production.
     */
    createMemoryKeyStore(): MemoryKeyStore {
        return new MemoryKeyStore();
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        return new P2PKH({ keyStore });
    }

    /**
     * Signs a Parcel with the given account.
     * @param parcel A Parcel
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedParcel
     * @throws When nonce or fee in the Parcel is null
     * @throws When account or its passphrase is invalid
     */
    async signParcel(parcel: Parcel, params: { account: PlatformAddress | H160 | string, passphrase?: string }): Promise<SignedParcel> {
        const { account, passphrase } = params;
        const address = account instanceof PlatformAddress ? account : PlatformAddress.fromAccountId(account);
        const sig = await this.rpc.account.sign(parcel.hash(), address, passphrase);
        return new SignedParcel(parcel, sig);
    }

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        MemoryKeyStore,
    };
}
