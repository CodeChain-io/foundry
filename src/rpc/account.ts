import { H256 } from "../core/classes";
import { PlatformAddress } from "../key/classes";

import { Rpc } from ".";

export class AccountRpc {
    private rpc: Rpc;

    /**
     * @hidden
     */
    constructor(rpc: Rpc) {
        this.rpc = rpc;
    }

    /**
     * Gets a list of accounts.
     * @returns A list of accounts
     */
    public getList(): Promise<string[]> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_getList", [])
                .then((accounts: string[]) => {
                    try {
                        if (Array.isArray(accounts)) {
                            resolve(
                                accounts.map(account =>
                                    PlatformAddress.ensure(account).toString()
                                )
                            );
                        } else {
                            reject(
                                Error(
                                    `Expected account_getList to return an array but it returned ${accounts}`
                                )
                            );
                        }
                    } catch (e) {
                        reject(
                            Error(
                                `Expected account_getList to return an array of PlatformAddress string, but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Creates a new account.
     * @param passphrase A passphrase to be used by the account owner
     * @returns An account
     */
    public create(passphrase?: string): Promise<string> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_create", [passphrase])
                .then(account => {
                    try {
                        resolve(PlatformAddress.ensure(account).toString());
                    } catch (e) {
                        reject(
                            Error(
                                `Expected account_create to return PlatformAddress string but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Imports a secret key and add the corresponding account.
     * @param secret H256 or hexstring for 256-bit private key
     * @param passphrase A passphrase to be used by the account owner
     * @returns The account
     */
    public importRaw(
        secret: H256 | string,
        passphrase?: string
    ): Promise<string> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_importRaw", [
                    `0x${H256.ensure(secret).value}`,
                    passphrase
                ])
                .then(account => {
                    try {
                        resolve(PlatformAddress.ensure(account).toString());
                    } catch (e) {
                        reject(
                            Error(
                                `Expected account_importRaw to return PlatformAddress string but an error occurred: ${e.toString()}`
                            )
                        );
                    }
                })
                .catch(reject);
        });
    }

    /**
     * Removes the account.
     * @param address A platform address
     * @param passphrase The account's passphrase
     */
    public remove(
        address: PlatformAddress | string,
        passphrase?: string
    ): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_remove", [
                    PlatformAddress.ensure(address).toString(),
                    passphrase
                ])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected account_remove to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Calculates the account's signature for a given message.
     * @param messageDigest A message to sign
     * @param address A platform address
     * @param passphrase The account's passphrase
     */
    public sign(
        messageDigest: H256 | string,
        address: PlatformAddress | string,
        passphrase?: string
    ): Promise<string> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_sign", [
                    `0x${H256.ensure(messageDigest).value}`,
                    PlatformAddress.ensure(address).toString(),
                    passphrase
                ])
                .then(result => {
                    if (
                        typeof result === "string" &&
                        result.match(/0x[0-9a-f]{130}/)
                    ) {
                        return resolve(result);
                    }
                    reject(
                        Error(
                            `Expected account_sign to return a 65 byte hexstring but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }

    /**
     * Unlocks the account.
     * @param address A platform address
     * @param passphrase The account's passphrase
     * @param duration Time to keep the account unlocked. The default value is 300(seconds). Passing 0 unlocks the account indefinitely.
     */
    public unlock(
        address: PlatformAddress | string,
        passphrase?: string,
        duration?: number
    ): Promise<null> {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_unlock", [
                    PlatformAddress.ensure(address).toString(),
                    passphrase || "",
                    duration
                ])
                .then(result => {
                    if (result === null) {
                        return resolve(null);
                    }
                    reject(
                        Error(
                            `Expected account_unlock to return null but it returned ${result}`
                        )
                    );
                })
                .catch(reject);
        });
    }
}
