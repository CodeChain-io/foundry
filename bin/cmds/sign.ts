import { PlatformAddress } from "codechain-sdk/lib/core/classes";
import * as yargs from "yargs";

import { GlobalParams } from "..";
import { askPasspharaseFor, asyncHandler, coerce, newSDK } from "../util";

interface SignParams extends GlobalParams {
    account: PlatformAddress;
    message: string;
}

export const module: yargs.CommandModule<GlobalParams, SignParams> = {
    command: "sign",
    describe: "Sign a message",
    builder(args) {
        return args
            .option("account", {
                describe: "An account to sign with",
                coerce: coerce("account", PlatformAddress.ensure),
                demand: true
            })
            .option("message", {
                describe: "A hex-string message to sign",
                coerce: coerce("message", (msg: any) => {
                    if (typeof msg !== "string") {
                        throw new Error("A message is not a string");
                    }
                    if (!/^(0x)?[0-9a-fA-F]*$/.test(msg)) {
                        throw new Error("A message is not a hex-string");
                    }
                    if (msg.startsWith("0x")) {
                        msg = msg.substr(2);
                    }
                    if (msg.length % 2 !== 0) {
                        throw new Error(
                            "A length of a message length is not even. It is not a valid hex-string"
                        );
                    }
                    return msg;
                }),
                demand: true
            });
    },
    handler: asyncHandler(async argv => {
        const sdk = await newSDK({
            server: argv["rpc-server"],
            keyStoreType: {
                type: "local",
                path: argv["keys-path"]
            }
        });
        const keystore = await sdk.key.createLocalKeyStore(argv["keys-path"]);

        console.log("=== Confirm your action ===");
        console.log("Action:", "Sign");
        console.log("Account:", argv.account.value);
        console.log("Message:", argv.message);

        const passphrase = await askPasspharaseFor(argv.account);

        const signature = await keystore.platform.sign({
            key: argv.account.accountId.value,
            message: sdk.util.blake256(argv.message),
            passphrase
        });

        console.log("Signature:");
        console.log(signature);
    })
};
