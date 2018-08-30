import { Parcel, SignedParcel, U256 } from "../core/classes";
import { Rpc } from "../rpc";

import { getAccountIdFromPublic } from "../utils";
import { AssetTransferAddress } from "./AssetTransferAddress";
import { KeyStore } from "./KeyStore";
import { LocalKeyStore } from "./LocalKeyStore";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";
import { PlatformAddress } from "./PlatformAddress";
import { RemoteKeyStore } from "./RemoteKeyStore";

type NetworkId = string;

export class Key {
    public static classes = {
        AssetTransferAddress,
        PlatformAddress,
        RemoteKeyStore,
        LocalKeyStore
    };

    public classes = Key.classes;
    private rpc: Rpc;
    private networkId: NetworkId;

    constructor(rpc: Rpc, options: { networkId: NetworkId }) {
        this.rpc = rpc;
        this.networkId = options.networkId;
        // FIXME:
        // tslint:disable-next-line:no-unused-expression
        this.rpc;
    }

    /**
     * Creates persistent key store
     * @param keystoreURL key store url (ex http://localhost:7007)
     */
    public createRemoteKeyStore(keystoreURL: string): Promise<KeyStore> {
        return RemoteKeyStore.create(keystoreURL);
    }

    /**
     * Creates persistent key store which stores data in the filesystem.
     */
    public createLocalKeyStore(): Promise<KeyStore> {
        return LocalKeyStore.create();
    }

    /**
     * Creates a new platform address
     * @returns A new platform address
     */
    public async createPlatformAddress(params: {
        keyStore: KeyStore;
    }): Promise<PlatformAddress> {
        const { keyStore } = params;
        const publicKey = await keyStore.platform.createKey();
        const accountId = getAccountIdFromPublic(publicKey);
        keyStore.mapping.add({ key: accountId, value: publicKey });
        return PlatformAddress.fromAccountId(accountId);
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    public createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        const { networkId } = this;
        return new P2PKH({ keyStore, networkId });
    }

    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    public createP2PKHBurn(params: { keyStore: KeyStore }): P2PKHBurn {
        const { keyStore } = params;
        const { networkId } = this;
        return new P2PKHBurn({ keyStore, networkId });
    }

    /**
     * Signs a Parcel with the given account.
     * @param parcel A Parcel
     * @param params.keyStore A key store.
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedParcel
     * @throws When nonce or fee in the Parcel is null
     * @throws When account or its passphrase is invalid
     */
    public async signParcel(
        parcel: Parcel,
        params: {
            keyStore: KeyStore;
            account: PlatformAddress | string;
            passphrase?: string;
            fee: U256 | string | number;
            nonce: U256 | string | number;
        }
    ): Promise<SignedParcel> {
        const { account, passphrase, keyStore, fee, nonce } = params;
        parcel.setFee(fee);
        parcel.setNonce(nonce);
        const accountId = PlatformAddress.ensure(account).getAccountId();
        const publicKey = await keyStore.mapping.get({ key: accountId.value });
        const sig = await keyStore.platform.sign({
            publicKey,
            message: parcel.hash().value,
            passphrase
        });
        return new SignedParcel(parcel, sig);
    }
}
