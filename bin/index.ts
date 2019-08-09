#!/usr/bin/env node

import * as yargs from "yargs";

import { module as delegateModule } from "./cmds/delegate";
import { module as revokeModule } from "./cmds/revoke";
import { module as selfNominateModule } from "./cmds/selfNominate";
import { module as showModule } from "./cmds/show";
import { module as signModule } from "./cmds/sign";
import { module as transferModule } from "./cmds/transfer";
import { module as validators } from "./cmds/validators";

export interface GlobalParams {
    "keys-path": string;
    "rpc-server": string;
}

const _argv = yargs
    .scriptName("ccstake")
    .locale("LC_ALL")
    .option("keys-path", {
        describe: "The path to storing the keys",
        string: true,
        normalize: true,
        default: "./keystore.db"
    })
    .option("rpc-server", {
        describe: "The RPC server URL",
        string: true,
        default: "https://rpc.codechain.io/"
    })
    .command(showModule)
    .command(transferModule)
    .command(delegateModule)
    .command(revokeModule)
    .command(selfNominateModule)
    .command(validators)
    .command(signModule)
    .demandCommand()
    .recommendCommands()
    .showHelpOnFail(true)
    .help()
    .strict().argv;
