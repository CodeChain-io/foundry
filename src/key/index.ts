import { Parcel, SignedParcel, U256 } from "../core/classes";
import { NetworkId } from "../core/types";

import { AssetTransferAddress } from "./AssetTransferAddress";
import { KeyStore } from "./KeyStore";
import { LocalKeyStore } from "./LocalKeyStore";
import { MemoryKeyStore } from "./MemoryKeyStore";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";
import { PlatformAddress } from "./PlatformAddress";
import { RemoteKeyStore } from "./RemoteKeyStore";

export class Key {
    public static classes = {
        AssetTransferAddress,
        PlatformAddress,
        RemoteKeyStore,
        LocalKeyStore
    };

    public classes = Key.classes;
    private networkId: NetworkId;

    constructor(options: { networkId: NetworkId }) {
        this.networkId = options.networkId;
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
     * @param dbPath A keystore file path
     */
    public createLocalKeyStore(dbPath?: string): Promise<KeyStore> {
        return LocalKeyStore.create({ dbPath });
    }

    /**
     * Creates a new platform address
     * @returns A new platform address
     */
    public async createPlatformAddress(params: {
        keyStore: KeyStore;
    }): Promise<PlatformAddress> {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const accountId = await keyStore.platform.createKey();
        return PlatformAddress.fromAccountId(accountId);
    }

    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    public createP2PKH(params: { keyStore: KeyStore }): P2PKH {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        const { networkId } = this;
        return new P2PKH({ keyStore, networkId });
    }

    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    public createP2PKHBurn(params: { keyStore: KeyStore }): P2PKHBurn {
        const { keyStore } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
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
        if (!(parcel instanceof Parcel)) {
            throw Error(
                `Expected the first argument of signParcel to be a Parcel instance but found ${parcel}`
            );
        }
        const { account, passphrase, keyStore, fee, nonce } = params;
        if (!isKeyStore(keyStore)) {
            throw Error(
                `Expected keyStore param to be a KeyStore instance but found ${keyStore}`
            );
        }
        if (!PlatformAddress.check(account)) {
            throw Error(
                `Expected account param to be a PlatformAddress value but found ${account}`
            );
        }
        parcel.setFee(fee);
        parcel.setNonce(nonce);
        const accountId = PlatformAddress.ensure(account).getAccountId();
        const sig = await keyStore.platform.sign({
            key: accountId.value,
            message: parcel.hash().value,
            passphrase
        });
        return new SignedParcel(parcel, sig);
    }
}

function isKeyStore(value: any) {
    return (
        value instanceof LocalKeyStore ||
        value instanceof RemoteKeyStore ||
        value instanceof MemoryKeyStore
    );
}
