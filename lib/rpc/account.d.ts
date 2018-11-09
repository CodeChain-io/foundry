import { H256, PlatformAddress, U256 } from "codechain-primitives";
import { Parcel } from "../core/Parcel";
import { Rpc } from ".";
export declare class AccountRpc {
    private rpc;
    private readonly parcelFee?;
    /**
     * @hidden
     */
    constructor(rpc: Rpc, options: {
        parcelFee?: number;
    });
    /**
     * Gets a list of accounts.
     * @returns A list of accounts
     */
    getList(): Promise<string[]>;
    /**
     * Creates a new account.
     * @param passphrase A passphrase to be used by the account owner
     * @returns An account
     */
    create(passphrase?: string): Promise<string>;
    /**
     * Imports a secret key and add the corresponding account.
     * @param secret H256 or hexstring for 256-bit private key
     * @param passphrase A passphrase to be used by the account owner
     * @returns The account
     */
    importRaw(secret: H256 | string, passphrase?: string): Promise<string>;
    /**
     * Calculates the account's signature for a given message.
     * @param messageDigest A message to sign
     * @param address A platform address
     * @param passphrase The account's passphrase
     */
    sign(messageDigest: H256 | string, address: PlatformAddress | string, passphrase?: string): Promise<string>;
    /**
     * Sends a parcel with the account's signature.
     * @param params.parcel A parcel to send
     * @param params.account The platform account to sign the parcel
     * @param params.passphrase The account's passphrase
     */
    sendParcel(params: {
        parcel: Parcel;
        account: PlatformAddress | string;
        passphrase?: string;
    }): Promise<{
        hash: H256;
        seq: U256;
    }>;
    /**
     * Unlocks the account.
     * @param address A platform address
     * @param passphrase The account's passphrase
     * @param duration Time to keep the account unlocked. The default value is 300(seconds). Passing 0 unlocks the account indefinitely.
     */
    unlock(address: PlatformAddress | string, passphrase?: string, duration?: number): Promise<null>;
}
