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
    const chainA = await runChainA();
    console.log("Run Chain B");
    const chainB = await runChainB();

    process.stdin.resume();
    let sentKillSignal = false;
    process.on("SIGINT", () => {
        if (sentKillSignal === false) {
            sentKillSignal = true;
            chainA.kill("SIGINT");
            chainB.kill("SIGINT");
            console.log("Sent kill signal to Foundry");
        } else if (!chainA.killed || !chainB.killed) {
            console.log("Waiting for foundry is killed");
        } else {
            process.exit();
        }
    });
    chainA.on("close", () => {
        console.log("Chain A is killed");
        if (chainA.killed && chainB.killed && sentKillSignal) {
            process.exit();
        }
    });
    chainA.on("close", () => {
        console.log("Chain B is killed");
        if (chainA.killed && chainB.killed && sentKillSignal) {
            process.exit();
        }
    });

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
            env: {
                ...process.env,
                WAIT_1_SEC_BEFORE_CREATING_A_BLOCK: "true",
                BYPASS_VERIFICATION_IN_STATIC_VALIDATOR: "true"
            },
            cwd: "./chainA"
        }
    );
    streamOutToDebug(foundryProcess);
    return foundryProcess;
}

async function runChainB() {
    const foundryProcess = spawn(
        "../../target/debug/foundry",
        ["-c", "./chainB.schem.json", "--config", "./chainB.config.toml"],
        {
            env: {
                ...process.env,
                WAIT_1_SEC_BEFORE_CREATING_A_BLOCK: "true",
                BYPASS_VERIFICATION_IN_STATIC_VALIDATOR: "true"
            },
            cwd: "./chainB"
        }
    );
    streamOutToDebug(foundryProcess);
    return foundryProcess;
}

async function checkChainAAndBAreRunning() {
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

    // Wait for Foundry to listen on the port, three seconds is an arbitrary value.
    let retryCount = 0;
    while (true) {
        try {
            debug("Send ping to A");
            await sdkA.rpc.node.ping();
            debug("Send ping to B");
            await sdkB.rpc.node.ping();
            break;
        } catch (err) {
            if (retryCount < 10) {
                retryCount += 1;
                debug("Failed to send ping. I will retry");
                await delay(1000);
            } else {
                throw err;
            }
        }
    }

    debug("Delete pending Txs in A");
    await sdkA.rpc.sendRpcRequest("mempool_deleteAllPendingTransactions", []);
    debug("Delete pending Txs in B");
    await sdkB.rpc.sendRpcRequest("mempool_deleteAllPendingTransactions", []);

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
        seq: await sdk.rpc.chain.getSeq(from)
    });
    debug(`Send payTx to ${chainName}`);
    const txhash = await sdk.rpc.chain.sendSignedTransaction(signedPay);
    await waitForTx(sdk, txhash);
}

async function waitForTx(sdk: SDK, txHash: H256) {
    const timeout = delay(30 * 1000).then(() => {
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
