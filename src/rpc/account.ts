import { H160 } from "../core/H160";
import { H256 } from "../core/H256";

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
    getList(): Promise<string[]> {
        return this.rpc.sendRpcRequest("account_getList", []);
    }

    /**
     * Creates a new account.
     * @param passphrase A passphrase to be used by the account owner
     * @returns An account
     */
    create(passphrase?: string): Promise<string> {
        return this.rpc.sendRpcRequest("account_create", [passphrase]);
    }

    /**
     * Imports a secret key and add the corresponding account.
     * @param secret H256 or hexstring for 256-bit private key
     * @param passphrase A passphrase to be used by the account owner
     * @returns The account
     */
    importRaw(secret: H256 | string, passphrase?: string): Promise<string> {
        return this.rpc.sendRpcRequest("account_importRaw", [
            `0x${H256.ensure(secret).value}`,
            passphrase
        ]);
    }

    /**
     * Removes the account.
     * @param account An account
     * @param passphrase The account's passphrase
     */
    remove(account: H160 | string, passphrase?: string): Promise<void> {
        return this.rpc.sendRpcRequest("account_remove", [
            `0x${H160.ensure(account).value}`,
            passphrase
        ]);
    }

    /**
     * Calculates the account's signature for a given message.
     * @param messageDigest A message to sign
     * @param account An account
     * @param passphrase The account's passphrase
     */
    sign(messageDigest: H256 | string, account: H160 | string, passphrase?: string): Promise<string> {
        return this.rpc.sendRpcRequest("account_sign", [
            `0x${H256.ensure(messageDigest).value}`,
            `0x${H160.ensure(account).value}`,
            passphrase
        ]);
    }
}
