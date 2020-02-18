#!/usr/bin/env node

import * as yargs from "yargs";

import { module as batchDelegate } from "./cmds/batchDelegate";
import { module as changeParams } from "./cmds/changeParams";
import { module as delegateModule } from "./cmds/delegate";
import { module as redelegateModule } from "./cmds/redelegate";
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
        describe: `The URL of RPC server (example: "https://localhost:8080)"`,
        string: true,
        demand: true
    })
    .group(["keys-path", "rpc-server", "version", "help"], "Common:")
    .command(showModule)
    .command(transferModule)
    .command(delegateModule)
    .command(batchDelegate)
    .command(revokeModule)
    .command(redelegateModule)
    .command(selfNominateModule)
    .command(validators)
    .command(signModule)
    .command(changeParams)
    .demandCommand()
    .recommendCommands()
    .showHelpOnFail(true)
    .help()
    .strict().argv;
