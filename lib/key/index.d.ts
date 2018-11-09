import { AssetTransferAddress, PlatformAddress } from "codechain-primitives";
import { AssetComposeTransaction, AssetDecomposeTransaction, AssetTransferTransaction, Parcel, SignedParcel, U256 } from "../core/classes";
import { NetworkId } from "../core/types";
import { SignatureTag } from "../utils";
import { KeyStore } from "./KeyStore";
import { LocalKeyStore } from "./LocalKeyStore";
import { P2PKH } from "./P2PKH";
import { P2PKHBurn } from "./P2PKHBurn";
import { RemoteKeyStore } from "./RemoteKeyStore";
export declare type KeyStoreType = "local" | "memory" | {
    type: "remote";
    url: string;
} | {
    type: "local";
    path: string;
};
export declare class Key {
    static classes: {
        RemoteKeyStore: typeof RemoteKeyStore;
        LocalKeyStore: typeof LocalKeyStore;
    };
    classes: {
        RemoteKeyStore: typeof RemoteKeyStore;
        LocalKeyStore: typeof LocalKeyStore;
    };
    private networkId;
    private keyStore;
    private keyStoreType;
    constructor(options: {
        networkId: NetworkId;
        keyStoreType: KeyStoreType;
    });
    /**
     * Creates persistent key store
     * @param keystoreURL key store url (ex http://localhost:7007)
     */
    createRemoteKeyStore(keystoreURL: string): Promise<KeyStore>;
    /**
     * Creates persistent key store which stores data in the filesystem.
     * @param dbPath A keystore file path
     */
    createLocalKeyStore(dbPath?: string): Promise<KeyStore>;
    /**
     * Creates a new platform address
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    createPlatformAddress(params?: {
        keyStore?: KeyStore;
        passphrase?: string;
    }): Promise<PlatformAddress>;
    /**
     * Creates a new asset transfer address
     * @param params.type The type of AssetTransferAddress. The default value is "P2PKH".
     * @param params.keyStore A key store.
     * @returns A new platform address
     */
    createAssetTransferAddress(params?: {
        type?: "P2PKH" | "P2PKHBurn";
        keyStore?: KeyStore;
        passphrase?: string;
    }): Promise<AssetTransferAddress>;
    /**
     * Creates P2PKH script generator.
     * @returns new instance of P2PKH
     */
    createP2PKH(params: {
        keyStore: KeyStore;
    }): P2PKH;
    /**
     * Creates P2PKHBurn script generator.
     * @returns new instance of P2PKHBurn
     */
    createP2PKHBurn(params: {
        keyStore: KeyStore;
    }): P2PKHBurn;
    /**
     * Signs a Parcel with the given account.
     * @param parcel A Parcel
     * @param params.keyStore A key store.
     * @param params.account An account.
     * @param params.passphrase The passphrase for the given account
     * @returns A SignedParcel
     * @throws When seq or fee in the Parcel is null
     * @throws When account or its passphrase is invalid
     */
    signParcel(parcel: Parcel, params: {
        keyStore?: KeyStore;
        account: PlatformAddress | string;
        passphrase?: string;
        fee: U256 | string | number;
        seq: U256 | string | number;
    }): Promise<SignedParcel>;
    /**
     * Signs a transaction's input with a given key store.
     * @param tx An AssetTransferTransaction.
     * @param index The index of an input to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given input.
     */
    signTransactionInput(tx: AssetTransferTransaction | AssetComposeTransaction | AssetDecomposeTransaction, index: number, params?: {
        keyStore?: KeyStore;
        passphrase?: string;
        signatureTag?: SignatureTag;
    }): Promise<void>;
    /**
     * Signs a transaction's burn with a given key store.
     * @param tx An AssetTransferTransaction.
     * @param index The index of a burn to sign.
     * @param params.keyStore A key store.
     * @param params.passphrase The passphrase for the given burn.
     */
    signTransactionBurn(tx: AssetTransferTransaction, index: number, params?: {
        keyStore?: KeyStore;
        passphrase?: string;
        signatureTag?: SignatureTag;
    }): Promise<void>;
    private ensureKeyStore;
}
