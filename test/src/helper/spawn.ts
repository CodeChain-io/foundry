// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import { expect } from "chai";
import { ChildProcess, spawn } from "child_process";
import RPC from "foundry-rpc";
import { createWriteStream, mkdtempSync, unlinkSync } from "fs";
import * as getPort from "get-port";
import * as mkdirp from "mkdirp";
import { ncp } from "ncp";
import { createInterface as createReadline, ReadLine } from "readline";
import * as WebSocket from "ws";
import { SDK } from "../sdk";
import {
    Address,
    H256,
    SignedTransaction,
    Transaction,
    U64
} from "../sdk/core/classes";
import * as stake from "../stakeholder";
import { faucetAddress, faucetSecret } from "./constants";
import { wait } from "./promise";

const projectRoot = `${__dirname}/../../..`;

export type SchemeFilepath = string;
export type ChainType =
    | "solo"
    | "tendermint"
    | "cuckoo"
    | "blake_pow"
    | "husky"
    | SchemeFilepath;
type ProcessState =
    | { state: "stopped" }
    | { state: "initializing" }
    | { state: "running"; process: ChildProcess; readline: ReadLine }
    | { state: "stopping" }
    | { state: "error"; message: string; source?: Error };
export class ProcessStateError extends Error {
    constructor(nodeId: number, state: ProcessState) {
        super(
            `CodeChain(${nodeId}) process state is invalid: ${JSON.stringify(
                state,
                undefined,
                4
            )}`
        );
    }
}
export interface Signer {
    privateKey: string;
    publicKey: string;
    address: Address;
}
export default class CodeChain {
    private static idCounter = 0;
    private readonly _id: number;
    private readonly _testFramework: SDK;
    private readonly _localKeyStorePath: string;
    private readonly _dbPath: string;
    private readonly _snapshotPath: string;
    private readonly _ipcPath: string;
    private readonly _keysPath: string;
    private readonly _logFile: string;
    private readonly _logPath: string;
    private readonly _chain: ChainType;
    private readonly argv: string[];
    private readonly env: { [key: string]: string };
    private _port?: number;
    private _rpcPort?: number;
    private _rpc?: RPC;
    private _informerPort?: number;
    private process: ProcessState;
    private restarts: number;
    private _keepLogs: boolean;
    private readonly keyFileMovePromise?: Promise<{}>;
    private _signer?: Signer;

    public get id(): number {
        return this._id;
    }
    public get testFramework(): SDK {
        if (this.process.state === "running") {
            return this._testFramework;
        } else {
            throw new ProcessStateError(this.id, this.process);
        }
    }
    public get rpc(): RPC {
        if (this.process.state === "running") {
            return this._rpc!;
        } else {
            throw new ProcessStateError(this.id, this.process);
        }
    }
    public get localKeyStorePath(): string {
        return this._localKeyStorePath;
    }
    public get dbPath(): string {
        return this._dbPath;
    }
    public get snapshotPath(): string {
        return this._snapshotPath;
    }
    public get ipcPath(): string {
        return this._ipcPath;
    }
    public get keysPath(): string {
        return this._keysPath;
    }
    public get logFile(): string {
        return this._logFile;
    }
    public get logPath(): string {
        return this._logPath;
    }
    public get rpcPort(): number {
        return this._rpcPort!;
    }
    public get port(): number {
        return this._port!;
    }
    public get informerPort(): number {
        return this._informerPort!;
    }
    public get secretKey(): number {
        return 1 + this.id;
    }
    public get chain(): ChainType {
        return this._chain;
    }
    public get isRunning(): boolean {
        return this.process.state === "running";
    }
    public get signer(): Signer {
        if (!this._signer) {
            throw new Error("Signer for a node is not set");
        }
        return this._signer;
    }

    public set signer(signer: Signer) {
        this._signer = signer;
    }

    constructor(
        options: {
            chain?: ChainType;
            argv?: string[];
            additionalKeysPath?: string;
            rpcPort?: number;
            env?: { [key: string]: string };
        } = {}
    ) {
        const { chain, argv, additionalKeysPath, env } = options;
        this._id = CodeChain.idCounter++;

        mkdirp.sync(`${projectRoot}/db/`);
        mkdirp.sync(`${projectRoot}/keys/`);
        mkdirp.sync(`${projectRoot}/test/log/`);
        mkdirp.sync(`${projectRoot}/snapshot/`);
        this._dbPath = mkdtempSync(`${projectRoot}/db/`);
        this._snapshotPath = mkdtempSync(`${projectRoot}/db/`);
        this._ipcPath = `/tmp/jsonrpc.${new Date()
            .toISOString()
            .replace(/[-:.]/g, "_")}.${this.id}.ipc`;
        this._keysPath = mkdtempSync(`${projectRoot}/keys/`);
        if (additionalKeysPath) {
            this.keyFileMovePromise = new Promise((resolve, reject) => {
                ncp(additionalKeysPath, this._keysPath, err => {
                    if (err) {
                        console.error(err);
                        reject(err);
                        return;
                    }
                    resolve();
                });
            });
        }
        this._localKeyStorePath = `${this.keysPath}/keystore.db`;
        this._logFile = `${new Date().toISOString().replace(/[-:.]/g, "_")}.${
            this.id
        }.log`;
        this._logPath = `${projectRoot}/test/log/${this._logFile}`;
        this._testFramework = new SDK({});
        this._chain = chain || "solo";
        this.argv = argv || [];
        this.env = env || {};
        this.process = { state: "stopped" };
        this.restarts = 0;
        this._keepLogs = false;
    }

    public async start(params?: {
        argv?: string[];
        logLevel?: string;
        disableLog?: boolean;
        disableIpc?: boolean;
    }) {
        if (this.process.state !== "stopped") {
            throw new ProcessStateError(this.id, this.process);
        }

        await this.initialize_port();

        this._rpc = new RPC(`http://localhost:${this.rpcPort}`, {
            devel: true
        });

        const {
            argv = [],
            logLevel = "trace,mio=warn,tokio=warn,hyper=warn,timer=warn",
            disableLog = false,
            disableIpc = true
        } = params || {};
        if (this.keyFileMovePromise) {
            await this.keyFileMovePromise;
        }
        const useDebugBuild = process.env.NODE_ENV !== "production";
        process.env.RUST_LOG = logLevel;

        const baseArgs = [...this.argv, ...argv];
        if (disableIpc) {
            baseArgs.push("--no-ipc");
        } else {
            baseArgs.push("--ipc-path");
            baseArgs.push(this.ipcPath);
        }

        // Resolves when CodeChain initialization completed.
        return new Promise((resolve, reject) => {
            this.restarts++;
            this.process = { state: "initializing" };
            const child = spawn(
                `target/${useDebugBuild ? "debug" : "release"}/foundry`,
                [
                    ...baseArgs,
                    "--chain",
                    this.chain,
                    "--informer-interface",
                    "127.0.0.1",
                    "--informer-port",
                    this.informerPort.toString(),
                    "--db-path",
                    this.dbPath,
                    "--snapshot-path",
                    this.snapshotPath,
                    "--keys-path",
                    this.keysPath,
                    "--no-ws",
                    "--jsonrpc-port",
                    this.rpcPort.toString(),
                    "--port",
                    this.port.toString(),
                    "--instance-id",
                    this.id.toString()
                ],
                {
                    cwd: projectRoot,
                    env: {
                        RUN_ON_TEST: "1",
                        ...process.env,
                        ...this.env
                    }
                }
            );
            if (!disableLog) {
                const logStream = createWriteStream(this.logPath, {
                    flags: "a"
                });
                logStream.write(`Process restart #${this.restarts}\n`);
                child.stdout.pipe(logStream);
                child.stderr.pipe(logStream);
            }

            const readline = createReadline({ input: child.stderr });
            const self = this;
            function clearListeners() {
                child
                    .removeListener("error", onError)
                    .removeListener("close", onClose)
                    .removeListener("exit", onExit);
                readline.removeListener("line", onLine);
            }
            function reportErrorState(errorState: {
                message: string;
                source?: Error;
            }) {
                self.process = {
                    state: "error",
                    ...errorState
                };
                self.keepLogs();
            }
            function onError(e: Error) {
                clearListeners();
                reportErrorState({
                    message: "Error while spawning CodeChain",
                    source: e
                });
                reject(new ProcessStateError(self.id, self.process));
            }
            function onClose(code: number, signal: number) {
                clearListeners();
                reportErrorState({
                    message: `CodeChain unexpectedly closed on start: code ${code}, signal ${signal}`
                });
                return reject(new ProcessStateError(self.id, self.process));
            }
            function onExit(code: number, signal: number) {
                clearListeners();
                reportErrorState({
                    message: `CodeChain unexpectedly exited on start: code ${code}, signal ${signal}`
                });
                reject(new ProcessStateError(self.id, self.process));
            }
            function onLine(line: string) {
                if (line.includes("Initialization complete")) {
                    clearListeners();
                    self.process = {
                        state: "running",
                        process: child,
                        readline
                    };
                    child.on("close", (code, signal) => {
                        clearListeners();
                        reportErrorState({
                            message: `CodeChain unexpectedly closed while running: code ${code}, signal ${signal}`
                        });
                    });
                    child.on("exit", (code, signal) => {
                        clearListeners();
                        reportErrorState({
                            message: `CodeChain unexpectedly exited while running: code ${code}, signal ${signal}`
                        });
                    });
                    readline.on("line", (l: string) => {
                        if (!l.startsWith("stack backtrace:")) {
                            return;
                        }
                        console.error(
                            `CodeChain(${self.id}) unexpectedly dumped backtrace`
                        );
                    });
                    resolve();
                }
            }

            child
                .on("error", onError)
                .on("close", onClose)
                .on("exit", onExit);
            readline.on("line", onLine);
        });
    }

    public keepLogs() {
        if (!this._keepLogs) {
            this._keepLogs = true;
            console.log(`Keep log file: ${this._logPath}`);
        }
    }

    public async clean() {
        return new Promise((resolve, reject) => {
            if (this.process.state === "stopped") {
                return resolve();
            } else if (this.process.state !== "running") {
                return reject(new ProcessStateError(this.id, this.process));
            }
            const { process: child, readline } = this.process;
            this.process = { state: "stopping" };
            readline.removeAllListeners("line");
            child
                .removeAllListeners()
                .on("error", e => {
                    child.removeAllListeners();
                    this.process = {
                        state: "error",
                        message: "CodeChain unexpectedly exited on clean",
                        source: e
                    };
                    reject(new ProcessStateError(this.id, this.process));
                })
                .on("close", (code, signal) => {
                    child.removeAllListeners();
                    if (code !== 0) {
                        console.error(
                            `CodeChain(${this.id}) closed with code ${code}, ${signal}`
                        );
                    } else if (!this._keepLogs) {
                        unlinkSync(this.logPath);
                    }
                    this.process = { state: "stopped" };
                    resolve();
                })
                .on("exit", (code, signal) => {
                    child.removeAllListeners();
                    if (code !== 0) {
                        console.error(
                            `CodeChain(${this.id}) exited with code ${code}, ${signal}`
                        );
                    } else if (!this._keepLogs) {
                        unlinkSync(this.logPath);
                    }
                    this.process = { state: "stopped" };
                    resolve();
                });
            child.kill();
        });
    }

    public async connect(peer: CodeChain) {
        if (!this.process) {
            return Promise.reject(Error("process isn't available"));
        }
        await this.rpc.net.connect({ address: "127.0.0.1", port: peer.port });
        while (
            !(await this.rpc.net.isConnected({
                address: "127.0.0.1",
                port: peer.port
            }))
        ) {
            await wait(250);
        }
    }

    public async disconnect(peer: CodeChain) {
        if (!this.process) {
            return Promise.reject(Error("process isn't available"));
        }
        return this.rpc.net.disconnect({
            address: "127.0.0.1",
            port: peer.port
        });
    }

    public async waitPeers(n: number) {
        while (n > (await this.rpc.net.getPeerCount())) {
            await wait(500);
        }
        return;
    }

    public async waitBlockNumberSync(peer: CodeChain) {
        while (
            (await this.getBestBlockNumber()) !==
            (await peer.getBestBlockNumber())
        ) {
            await wait(500);
        }
    }

    public async waitBlockNumber(n: number) {
        while ((await this.getBestBlockNumber()) < n) {
            await wait(500);
        }
    }

    public async getBestBlockNumber() {
        return this.rpc.chain.getBestBlockNumber();
    }

    public async getBestBlockHash() {
        return new H256(
            await this.rpc.chain.getBlockHash({
                blockNumber: await this.getBestBlockNumber()
            })
        );
    }

    public async createaddress() {
        const keyStore = await this.testFramework.key.createLocalKeyStore(
            this.localKeyStorePath
        );
        return this.testFramework.key.createaddress({ keyStore });
    }

    public async pay(
        recipient: string | Address,
        quantity: U64 | string | number
    ): Promise<H256> {
        const tx = this.testFramework.core
            .createPayTransaction({
                recipient,
                quantity
            })
            .sign({
                secret: faucetSecret,
                seq: (await this.rpc.chain.getSeq({
                    address: faucetAddress.toString(),
                    blockNumber: null
                }))!,
                fee: 10
            });
        return new H256(
            await this.rpc.mempool.sendSignedTransaction({
                tx: tx.rlpBytes().toString("hex")
            })
        );
    }

    public async sendTransaction(
        tx: Transaction,
        params: {
            account: string | Address;
            fee?: number | string | U64;
            seq?: number;
        }
    ) {
        const keyStore = await this.testFramework.key.createLocalKeyStore(
            this.localKeyStorePath
        );
        const { account, fee = 10 } = params;
        const {
            seq = (await this.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: null
            }))!
        } = params;
        const signed = await this.testFramework.key.signTransaction(tx, {
            keyStore,
            account,
            fee,
            seq
        });
        return new H256(
            await this.rpc.mempool.sendSignedTransaction({
                tx: signed.rlpBytes().toString("hex")
            })
        );
    }

    public createPayTx(options: {
        seq: number;
        recipient?: Address | string;
        quantity?: number;
        secret?: any;
        fee?: number;
    }): SignedTransaction {
        const {
            seq,
            recipient = "nxcmkryvIAwv6UL9vpFDMj7SZNDjnnsyHKM8eL-nipJXbgDcnr0tc0",
            quantity = 0,
            secret = faucetSecret,
            fee = 10 + this.id
        } = options || {};
        return this.testFramework.core
            .createPayTransaction({
                recipient,
                quantity
            })
            .sign({
                secret,
                fee,
                seq
            });
    }

    public async sendPayTx(options?: {
        seq?: number;
        recipient?: Address | string;
        quantity?: number;
        secret?: any;
        fee?: number;
    }): Promise<SignedTransaction> {
        const {
            seq = (await this.rpc.chain.getSeq({
                address: faucetAddress.toString(),
                blockNumber: await this.getBestBlockNumber()
            })) || 0
        } = options || {};
        const tx = this.createPayTx({
            seq,
            ...options
        });
        await this.rpc.mempool.sendSignedTransaction({
            tx: tx.rlpBytes().toString("hex")
        });
        return tx;
    }

    // If one only sends certainly failing transactions, the miner would not generate any block.
    // So to clearly check the result failed, insert the failing transactions after succeessful ones.
    public async sendTransactionExpectedToFail(
        tx: Transaction,
        options: { account: string | Address }
    ): Promise<H256> {
        const { account } = options;
        await this.rpc.devel!.stopSealing();

        const blockNumber = await this.getBestBlockNumber();
        const signedDummyTxHash = (
            await this.sendPayTx({
                quantity: 1
            })
        ).hash();
        const targetTxHash = await this.sendTransaction(tx, { account });

        await this.rpc.devel!.startSealing();
        await this.waitBlockNumber(blockNumber + 1);

        expect(
            await this.rpc.chain.containsTransaction({
                transactionHash: `0x${targetTxHash.toString()}`
            })
        ).be.false;
        expect(
            await this.rpc.mempool.getErrorHint({
                transactionHash: targetTxHash.toString()
            })
        ).not.null;
        expect(
            await this.rpc.chain.getTransaction({
                transactionHash: `0x${targetTxHash.toString()}`
            })
        ).be.null;

        expect(
            await this.rpc.chain.containsTransaction({
                transactionHash: `0x${signedDummyTxHash.toString()})`
            })
        ).be.true;
        expect(
            await this.rpc.mempool.getErrorHint({
                transactionHash: signedDummyTxHash.toString()
            })
        ).null;
        expect(
            await this.rpc.chain.getTransaction({
                transactionHash: `0x${signedDummyTxHash.toString()}`
            })
        ).not.be.null;

        return targetTxHash;
    }

    public async sendSignedTransactionExpectedToFail(
        tx: SignedTransaction | (() => Promise<H256>),
        options: { error?: string } = {}
    ): Promise<H256> {
        await this.rpc.devel!.stopSealing();

        const blockNumber = await this.getBestBlockNumber();
        const signedDummyTxHash = (
            await this.sendPayTx({
                fee: 1000,
                quantity: 1
            })
        ).hash();

        const targetTxHash =
            tx instanceof SignedTransaction
                ? await this.rpc.mempool.sendSignedTransaction({
                      tx: tx.rlpBytes().toString("hex")
                  })
                : await tx();

        await this.rpc.devel!.startSealing();
        await this.waitBlockNumber(blockNumber + 1);

        expect(
            await this.rpc.chain.containsTransaction({
                transactionHash: `${targetTxHash.toString()}`
            })
        ).be.false;
        const hint = await this.rpc.mempool.getErrorHint({
            transactionHash: targetTxHash.toString()
        });
        expect(hint).not.null;
        if (options.error != null) {
            expect(hint).contains(options.error);
        }
        expect(
            await this.rpc.chain.getTransaction({
                transactionHash: `${targetTxHash.toString()}`
            })
        ).be.null;

        expect(
            await this.rpc.chain.containsTransaction({
                transactionHash: `0x${signedDummyTxHash.toString()}`
            })
        ).be.true;
        expect(
            await this.rpc.mempool.getErrorHint({
                transactionHash: `0x${signedDummyTxHash.toString()}`
            })
        ).null;
        expect(
            await this.rpc.chain.getTransaction({
                transactionHash: `0x${signedDummyTxHash.toString()}`
            })
        ).not.be.null;

        return new H256(targetTxHash.toString());
    }

    public async sendSignedTransactionWithRlpBytes(
        rlpBytes: Buffer
    ): Promise<H256> {
        const bytes = Array.from(rlpBytes)
            .map(byte =>
                byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)
            )
            .join("");
        return new H256(
            await this.rpc.mempool.sendSignedTransaction({ tx: `0x${bytes}` })
        );
    }

    public async waitForTx(
        hashlikes: H256 | Promise<H256> | (H256 | Promise<H256>)[],
        option?: { timeout?: number }
    ) {
        const { timeout = 10000 } = option || {};

        const hashes = await Promise.all(
            Array.isArray(hashlikes) ? hashlikes : [hashlikes]
        );

        const containsAll = async () => {
            const contains = await Promise.all(
                hashes.map(hash =>
                    this.rpc.chain.containsTransaction({
                        transactionHash: `0x${hash.toString()}`
                    })
                )
            );
            return contains.every(x => x);
        };
        const checkNoError = async () => {
            const errorHints = await Promise.all(
                hashes.map(hash =>
                    this.rpc.mempool.getErrorHint({
                        transactionHash: `0x${hash.toString()}`
                    })
                )
            );
            for (const errorHint of errorHints) {
                if (errorHint !== null && errorHint !== "") {
                    throw Error(`waitForTx: Error found: ${errorHint}`);
                }
            }
        };

        const start = Date.now();
        while (!(await containsAll())) {
            await checkNoError();

            await wait(500);
            if (Date.now() - start >= timeout) {
                throw Error("Timeout on waitForTx");
            }
        }
        await checkNoError();
    }

    public async waitForTermChange(target: number, timeout?: number) {
        const start = Date.now();
        while (true) {
            const termMetadata = (await stake.getTermMetadata(this.rpc))!;
            if (termMetadata.currentTermId >= target) {
                return termMetadata;
            }
            await wait(1000);
            if (timeout) {
                if (Date.now() - start > timeout * 1000) {
                    throw new Error(
                        `Term didn't changed to ${target} in ${timeout} s. It is ${termMetadata.currentTermId} now`
                    );
                }
            }
        }
    }
    public informerClient(): WebSocket {
        return new WebSocket(`ws://localhost:${this.informerPort}`);
    }
    private async initialize_port() {
        this._port = await getPort({ port: this.id + 3486 });
        this._rpcPort = await getPort({ port: this.id + 8081 });
        this._informerPort = await getPort({ port: this.id + 7070 });
    }
}
