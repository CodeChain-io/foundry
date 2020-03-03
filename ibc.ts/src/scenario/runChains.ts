import { promises as fs } from "fs";
import {
    spawn,
    ChildProcess,
    ChildProcessWithoutNullStreams
} from "child_process";
import * as readline from "readline";
import Debug from "debug";
import { SDK } from "codechain-sdk";
import { delay } from "../common/util";
import { H256, PlatformAddressValue } from "codechain-primitives/lib";

const debug = Debug("runChains");

async function main() {
    console.log("Build foundry");
    await buildFoundry();
    console.log("Reset DB");
    await resetDB();
    console.log("Run Chain A");
    await runChainA();
    console.log("Run Chain B");
    await runChainB();
    console.log("Check the chains");
    await checkChainAAndBAreRunning();
    console.log("Chains are running!");
}

main().catch(console.error);

async function buildFoundry() {
    const cargoProcess = spawn("cargo", ["build"], {
        cwd: "../"
    });
    streamOutToDebug(cargoProcess);
    await waitForChildProcess(cargoProcess);
}

async function resetDB() {
    debug("Remove chain A DB");
    await fs.rmdir("./chainA/db", {
        recursive: true
    });

    debug("Remove chain B DB");
    await fs.rmdir("./chainB/db", {
        recursive: true
    });
}

async function runChainA() {
    const foundryProcess = spawn(
        "../../target/debug/foundry",
        ["-c", "./chainA.schem.json", "--config", "./chainA.config.toml"],
        {
            cwd: "./chainA"
        }
    );
    streamOutToDebug(foundryProcess);
}

async function runChainB() {
    const foundryProcess = spawn(
        "../../target/debug/foundry",
        ["-c", "./chainB.schem.json", "--config", "./chainB.config.toml"],
        {
            cwd: "./chainB"
        }
    );
    streamOutToDebug(foundryProcess);
}

async function checkChainAAndBAreRunning() {
    // Wait for Foundry to listen on the port, three seconds is an arbitrary value.
    await delay(3000);

    // FIXME: read values from config
    const sdkA = new SDK({
        server: "http://localhost:18080",
        networkId: "ac",
        keyStoreType: { type: "local", path: "./chainA/keystore.db" }
    });
    const sdkB = new SDK({
        server: "http://localhost:18081",
        networkId: "bc",
        keyStoreType: { type: "local", path: "./chainB/keystore.db" }
    });

    debug("Send ping to A");
    await sdkA.rpc.node.ping();
    debug("Send ping to B");
    await sdkB.rpc.node.ping();

    await sendPayTx({
        sdk: sdkA,
        from: "accqym7qmn5yj29cdl405xlmx6awd3f3yz07g7vq2c9",
        chainName: "Chain A"
    });

    await sendPayTx({
        sdk: sdkB,
        from: "bccqygjwzj8wupc9m7du9ccef4j6k2u3erjuv2w8pt0",
        chainName: "Chain B"
    });
}

async function sendPayTx({
    sdk,
    from,
    chainName
}: {
    sdk: SDK;
    from: PlatformAddressValue;
    chainName: string;
}) {
    const pay = sdk.core.createPayTransaction({
        recipient: from,
        quantity: 1000
    });
    const signedPay = await sdk.key.signTransaction(pay, {
        account: from,
        passphrase: "",
        fee: 1000,
        seq: 0
    });
    debug(`Send payTx to ${chainName}`);
    const txhash = await sdk.rpc.chain.sendSignedTransaction(signedPay);
    await waitForTx(sdk, txhash);
}

async function waitForTx(sdk: SDK, txHash: H256) {
    const timeout = delay(10 * 1000).then(() => {
        throw new Error("Timeout");
    });
    const wait = (async () => {
        while (true) {
            debug(`wait tx: ${txHash.toString()}`);
            if (await sdk.rpc.chain.containsTransaction(txHash)) {
                return;
            }
            await delay(500);
        }
    })();
    return Promise.race([timeout, wait]);
}

function streamOutToDebug(childProcess: ChildProcessWithoutNullStreams) {
    const rlStdout = readline.createInterface({
        input: childProcess.stdout,
        terminal: false
    });
    rlStdout.on("line", line => debug(line));

    const rlStdErr = readline.createInterface({
        input: childProcess.stderr,
        terminal: false
    });
    rlStdErr.on("line", line => debug(line));
}

async function waitForChildProcess(childProcess: ChildProcess) {
    let killed = false;
    return new Promise((resolve, reject) => {
        childProcess.on("exit", () => {
            if (!killed) {
                resolve();
                killed = true;
            }
        });
        childProcess.on("error", error => {
            reject(error);
            killed = true;
        });
    });
}
