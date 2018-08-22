import { Rpc } from "../rpc";
import { Parcel, SignedParcel, H160 } from "../core/classes";

import { AssetTransferAddress } from "./AssetTransferAddress";
import { PlatformAddress } from "./PlatformAddress";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";
import { KeyStore } from "./KeyStore";
import { RemoteKeyStore } from "./RemoteKeyStore";
import { LocalKeyStore } from "./LocalKeyStore";

type NetworkId = string;

export class Key {
    private rpc: Rpc;
    private networkId: NetworkId;

    constructor(rpc: Rpc, options: { networkId: NetworkId }) {
        this.rpc = rpc;
        this.networkId = options.networkId;
    }

    /**
     * Creates persistent key store
     * @param keystoreURL key store url (ex http://localhost:7007)
     */
    createRemoteKeyStore(keystoreURL: string): Promise<KeyStore> {
        return RemoteKeyStore.create(keystoreURL);
    }

    /**
     * Creates persistent key store which stores data in the filesystem.
     */
    createLocalKeyStore(): Promise<KeyStore> {
        return LocalKeyStore.create();
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        const { networkId } = this;
        return new P2PKH({ keyStore, networkId });
    }

    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    createP2PKHBurn(params: { keyStore: KeyStore }): P2PKHBurn {
        const { keyStore } = params;
        const { networkId } = this;
        return new P2PKHBurn({ keyStore, networkId });
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
        const address = PlatformAddress.fromAccountId(PlatformAddress.ensureAccount(account));
        const sig = await this.rpc.account.sign(parcel.hash(), address, passphrase);
        return new SignedParcel(parcel, sig);
    }

    public classes = Key.classes;
    static classes = {
        AssetTransferAddress,
        PlatformAddress,
        RemoteKeyStore,
        LocalKeyStore,
    };
}
