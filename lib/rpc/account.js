"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
class AccountRpc {
    /**
     * @hidden
     */
    constructor(rpc, options) {
        const { parcelFee } = options;
        this.rpc = rpc;
        this.parcelFee = parcelFee;
    }
    /**
     * Gets a list of accounts.
     * @returns A list of accounts
     */
    getList() {
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_getList", [])
                .then((accounts) => {
                try {
                    if (Array.isArray(accounts)) {
                        resolve(accounts.map(account => codechain_primitives_1.PlatformAddress.ensure(account).toString()));
                    }
                    else {
                        reject(Error(`Expected account_getList to return an array but it returned ${accounts}`));
                    }
                }
                catch (e) {
                    reject(Error(`Expected account_getList to return an array of PlatformAddress string, but an error occurred: ${e.toString()}`));
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
    create(passphrase) {
        if (passphrase && typeof passphrase !== "string") {
            throw Error(`Expected the first argument to be a string but given ${passphrase}`);
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_create", [passphrase])
                .then(account => {
                try {
                    resolve(codechain_primitives_1.PlatformAddress.ensure(account).toString());
                }
                catch (e) {
                    reject(Error(`Expected account_create to return PlatformAddress string but an error occurred: ${e.toString()}`));
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
    importRaw(secret, passphrase) {
        if (!codechain_primitives_1.H256.check(secret)) {
            throw Error(`Expected the first argument to be an H256 value but found ${secret}`);
        }
        if (passphrase && typeof passphrase !== "string") {
            throw Error(`Expected the second argument to be a string but found ${passphrase}`);
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_importRaw", [
                `0x${codechain_primitives_1.H256.ensure(secret).value}`,
                passphrase
            ])
                .then(account => {
                try {
                    resolve(codechain_primitives_1.PlatformAddress.ensure(account).toString());
                }
                catch (e) {
                    reject(Error(`Expected account_importRaw to return PlatformAddress string but an error occurred: ${e.toString()}`));
                }
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
    sign(messageDigest, address, passphrase) {
        if (!codechain_primitives_1.H256.check(messageDigest)) {
            throw Error(`Expected the first argument to be an H256 value but found ${messageDigest}`);
        }
        if (!codechain_primitives_1.PlatformAddress.check(address)) {
            throw Error(`Expected the second argument to be a PlatformAddress value but found ${address}`);
        }
        if (passphrase && typeof passphrase !== "string") {
            throw Error(`Expected the third argument to be a string but found ${passphrase}`);
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_sign", [
                `0x${codechain_primitives_1.H256.ensure(messageDigest).value}`,
                codechain_primitives_1.PlatformAddress.ensure(address).toString(),
                passphrase
            ])
                .then(result => {
                if (typeof result === "string" &&
                    result.match(/0x[0-9a-f]{130}/)) {
                    return resolve(result);
                }
                reject(Error(`Expected account_sign to return a 65 byte hexstring but it returned ${result}`));
            })
                .catch(reject);
        });
    }
    /**
     * Sends a parcel with the account's signature.
     * @param params.parcel A parcel to send
     * @param params.account The platform account to sign the parcel
     * @param params.passphrase The account's passphrase
     */
    sendParcel(params) {
        const { parcel, account, passphrase } = params;
        if (!codechain_primitives_1.PlatformAddress.check(account)) {
            throw Error(`Expected account is a PlatformAddress value but found ${account}`);
        }
        if (passphrase && typeof passphrase !== "string") {
            throw Error(`Expected the third argument to be a string but found ${passphrase}`);
        }
        if (!parcel.fee && this.parcelFee != null) {
            parcel.setFee(this.parcelFee);
        }
        return this.rpc
            .sendRpcRequest("account_sendParcel", [
            parcel.toJSON(),
            codechain_primitives_1.PlatformAddress.ensure(account).toString(),
            passphrase
        ])
            .then(result => {
            return {
                hash: codechain_primitives_1.H256.ensure(result.hash),
                seq: codechain_primitives_1.U256.ensure(result.seq)
            };
        });
    }
    /**
     * Unlocks the account.
     * @param address A platform address
     * @param passphrase The account's passphrase
     * @param duration Time to keep the account unlocked. The default value is 300(seconds). Passing 0 unlocks the account indefinitely.
     */
    unlock(address, passphrase, duration) {
        if (!codechain_primitives_1.PlatformAddress.check(address)) {
            throw Error(`Expected the first argument to be a PlatformAddress value but found ${address}`);
        }
        if (passphrase && typeof passphrase !== "string") {
            throw Error(`Expected the second argument to be a string but found ${passphrase}`);
        }
        if (duration !== undefined &&
            (typeof duration !== "number" ||
                !Number.isInteger(duration) ||
                duration < 0)) {
            throw Error(`Expected the third argument to be non-negative integer but found ${duration}`);
        }
        return new Promise((resolve, reject) => {
            this.rpc
                .sendRpcRequest("account_unlock", [
                codechain_primitives_1.PlatformAddress.ensure(address).toString(),
                passphrase || "",
                duration
            ])
                .then(result => {
                if (result === null) {
                    return resolve(null);
                }
                reject(Error(`Expected account_unlock to return null but it returned ${result}`));
            })
                .catch(reject);
        });
    }
}
exports.AccountRpc = AccountRpc;
