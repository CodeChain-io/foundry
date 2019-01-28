import { SDK } from "../";
import { Asset } from "../lib/core/Asset";
import { AssetScheme } from "../lib/core/AssetScheme";
import {
    H256,
    Invoice,
    MintAsset,
    PlatformAddress,
    SignedTransaction,
    Transaction,
    TransferAsset,
    U256,
    U64
} from "../lib/core/classes";
import {
    generatePrivateKey,
    getAccountIdFromPrivate,
    signEcdsa
} from "../lib/utils";

import {
    ACCOUNT_ADDRESS,
    ACCOUNT_SECRET,
    CODECHAIN_NETWORK_ID,
    SERVER_URL
} from "./helper";

// FIXME:
const ERROR = {
    VERIFICATION_FAILED: {
        code: -32030,
        data: expect.anything(),
        message: expect.anything()
    },
    ALREADY_IMPORTED: {
        code: -32031,
        data: expect.anything(),
        message: expect.anything()
    },
    NOT_ENOUGH_BALANCE: {
        code: -32032,
        data: expect.anything(),
        message: expect.anything()
    },
    TOO_LOW_FEE: {
        code: -32033,
        data: expect.anything(),
        message: expect.anything()
    },
    TOO_CHEAP_TO_REPLACE: {
        code: -32034,
        data: expect.anything(),
        message: expect.anything()
    },
    INVALID_SEQ: {
        code: -32035,
        data: expect.anything(),
        message: expect.anything()
    },
    INVALID_NETWORK_ID: {
        code: -32036,
        data: expect.anything(),
        message: expect.anything()
    },
    // FIXME:
    KEY_ERROR: {
        code: -32041,
        data: expect.anything(),
        message: expect.anything()
    },
    // FIXME:
    ALREADY_EXISTS: {
        code: -32042,
        data: expect.anything(),
        message: expect.anything()
    },
    // FIXME:
    WRONG_PASSWORD: {
        code: -32043,
        data: expect.anything(),
        message: expect.anything()
    },
    // FIXME:
    NO_SUCH_ACCOUNT: {
        code: -32044,
        data: expect.anything(),
        message: expect.anything()
    },
    INVALID_PARAMS: {
        code: -32602,
        message: expect.anything()
    }
};

describe("rpc", () => {
    let sdk: SDK;
    const { Block } = SDK.Core.classes;
    const invalidHash =
        "0x0000000000000000000000000000000000000000000000000000000000000000";
    let signerSecret: string;
    let signerAccount: string;
    let signerAddress: string;

    beforeAll(async () => {
        sdk = new SDK({
            server: SERVER_URL,
            keyStoreType: "memory",
            networkId: CODECHAIN_NETWORK_ID
        });

        signerSecret = sdk.util.generatePrivateKey();
        signerAccount = sdk.util.getAccountIdFromPrivate(signerSecret);
        signerAddress = sdk.core.classes.PlatformAddress.fromAccountId(
            signerAccount,
            { networkId: "tc" }
        ).toString();
        const seq = await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS);
        const pay = sdk.core
            .createPayTransaction({
                recipient: signerAddress,
                quantity: 1_000_000_000
            })
            .sign({ secret: ACCOUNT_SECRET, seq, fee: 10 });
        await sdk.rpc.chain.sendSignedTransaction(pay);
    });

    test("PlatformAddress", () => {
        expect(
            sdk.core.classes.PlatformAddress.fromAccountId(signerAccount, {
                networkId: CODECHAIN_NETWORK_ID
            }).value
        ).toEqual(signerAddress);
    });

    describe("node", () => {
        test("ping", async () => {
            expect(await sdk.rpc.node.ping()).toBe("pong");
        });

        test("getNodeVersion", async () => {
            // FIXME: regex for semver
            expect(typeof (await sdk.rpc.node.getNodeVersion())).toBe("string");
        });
    });

    test("getBestBlockNumber", async () => {
        expect(typeof (await sdk.rpc.chain.getBestBlockNumber())).toBe(
            "number"
        );
    });

    describe("chain", () => {
        test("getBlockHash", async () => {
            expect(await sdk.rpc.chain.getBlockHash(0)).toEqual(
                expect.any(H256)
            );
            expect(await sdk.rpc.chain.getBlockHash(9999999999)).toEqual(null);
        });

        test("getBlock - by number", async () => {
            expect(await sdk.rpc.chain.getBlock(0)).toEqual(expect.any(Block));
            expect(await sdk.rpc.chain.getBlock(9999999999)).toEqual(null);
        });

        test("getBlock - by hash", async () => {
            const hash = await sdk.rpc.chain.getBlockHash(0);
            if (hash == null) {
                throw Error("Cannot get block hash");
            }
            expect(await sdk.rpc.chain.getBlock(hash)).toEqual(
                expect.any(Block)
            );
            expect(await sdk.rpc.chain.getBlock(hash.value)).toEqual(
                expect.any(Block)
            );

            expect(await sdk.rpc.chain.getBlock(invalidHash)).toEqual(null);
        });

        describe("with account", () => {
            test("PlatformAddress", () => {
                expect(
                    sdk.core.classes.PlatformAddress.fromAccountId(
                        signerAccount,
                        {
                            networkId: "tc"
                        }
                    ).value
                ).toEqual(signerAddress);
            });

            test("getBalance", async () => {
                expect(await sdk.rpc.chain.getBalance(signerAddress)).toEqual(
                    expect.any(U64)
                );
            });

            test("getSeq", async () => {
                expect(typeof (await sdk.rpc.chain.getSeq(signerAddress))).toBe(
                    "number"
                );
            });

            describe("has regular key", () => {
                let signerSecretForRK: string;
                let signerAccountForRK: string;
                let signerAddressForRK: string;
                let regularKey: string;

                beforeAll(async () => {
                    const regularSecret = sdk.util.generatePrivateKey();
                    regularKey = sdk.util.getPublicFromPrivate(regularSecret);
                    signerSecretForRK = sdk.util.generatePrivateKey();
                    signerAccountForRK = sdk.util.getAccountIdFromPrivate(
                        signerSecretForRK
                    );
                    signerAddressForRK = sdk.core.classes.PlatformAddress.fromAccountId(
                        signerAccountForRK,
                        { networkId: "tc" }
                    ).toString();
                    const seq = await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS);
                    const pay = sdk.core
                        .createPayTransaction({
                            recipient: signerAddressForRK,
                            quantity: 1_000_000
                        })
                        .sign({ secret: ACCOUNT_SECRET, seq, fee: 10 });
                    await sdk.rpc.chain.sendSignedTransaction(pay);
                    const transaction = sdk.core
                        .createSetRegularKeyTransaction({
                            key: regularKey
                        })
                        .sign({
                            secret: signerSecretForRK,
                            seq: await sdk.rpc.chain.getSeq(signerAddressForRK),
                            fee: 10
                        });
                    await sdk.rpc.chain.sendSignedTransaction(transaction);
                });

                test("getRegularKey", async () => {
                    const regularKeyOfAddress = await sdk.rpc.chain.getRegularKey(
                        signerAddressForRK
                    );
                    if (regularKeyOfAddress == null) {
                        throw Error("Cannot get a regular key");
                    }
                    expect(regularKeyOfAddress.value).toEqual(regularKey);
                });

                test("getRegularKeyOwner", async () => {
                    const keyOwner = await sdk.rpc.chain.getRegularKeyOwner(
                        regularKey
                    );
                    if (keyOwner == null) {
                        throw Error("Cannot get a key owner");
                    }
                    expect(keyOwner.toString()).toEqual(signerAddressForRK);
                });
            });
        });

        describe("sendSignedTransaction", () => {
            let seq: number;
            let tx: Transaction;
            beforeEach(async () => {
                tx = sdk.core.createPayTransaction({
                    recipient: signerAddress,
                    quantity: 10
                });
                seq = await sdk.rpc.chain.getSeq(signerAddress);
            });

            test("Ok", async done => {
                sdk.rpc.chain
                    .sendSignedTransaction(
                        tx.sign({ secret: signerSecret, fee: 10, seq })
                    )
                    .then(() => done())
                    .catch(e => done.fail(e));
            });

            test("VerificationFailed", done => {
                const signed = tx.sign({ secret: signerSecret, fee: 10, seq });
                signed.r = new U256(0);
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.VERIFICATION_FAILED);
                        done();
                    });
            });

            test("AlreadyImported", done => {
                const signed = tx.sign({ secret: signerSecret, fee: 10, seq });
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => {
                        sdk.rpc.chain
                            .sendSignedTransaction(signed)
                            .then(() => done.fail())
                            .catch(e => {
                                expect(e).toEqual(ERROR.ALREADY_IMPORTED);
                                done();
                            });
                    })
                    .catch(done.fail);
            });

            test("NotEnoughBalance", async done => {
                const signed = tx.sign({
                    secret: signerSecret,
                    fee: new U64("0xffffffffffffffff"),
                    seq
                });
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.NOT_ENOUGH_BALANCE);
                        done();
                    });
            });

            test("TooLowFee", done => {
                const signed = tx.sign({ secret: signerSecret, fee: 9, seq });
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.TOO_LOW_FEE);
                        done();
                    });
            });

            test.skip("TooCheapToReplace", done => {
                done.fail("Not implemented");
            });

            test("InvalidSeq", done => {
                const signed = tx.sign({
                    secret: signerSecret,
                    fee: 12321,
                    seq: seq - 1
                });
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.INVALID_SEQ);
                        done();
                    });
            });

            test("InvalidNetworkId", done => {
                (tx as any)._networkId = "zz";
                const signed = tx.sign({ secret: signerSecret, fee: 10, seq });
                sdk.rpc.chain
                    .sendSignedTransaction(signed)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.INVALID_NETWORK_ID);
                        done();
                    });
            });
        });

        describe("with tx hash", () => {
            let txHash: H256;

            beforeAll(async () => {
                const tx = sdk.core.createPayTransaction({
                    recipient: signerAddress,
                    quantity: 10
                });
                const signed = tx.sign({
                    secret: signerSecret,
                    fee: 10,
                    seq: await sdk.rpc.chain.getSeq(signerAddress)
                });
                txHash = await sdk.rpc.chain.sendSignedTransaction(signed);
            });

            test("getTransaction", async () => {
                expect(await sdk.rpc.chain.getTransaction(txHash)).toEqual(
                    expect.any(SignedTransaction)
                );
            });

            test("getInvoice", async () => {
                expect(await sdk.rpc.chain.getInvoice(txHash)).toEqual(
                    expect.any(Invoice)
                );
                expect(await sdk.rpc.chain.getInvoice(invalidHash)).toBe(null);
            });
        });

        describe.skip("with pending transactions", () => {
            test("getPendingTransactions", async () => {
                const pendingTransactions = await sdk.rpc.chain.getPendingTransactions();
                expect(pendingTransactions[0]).toEqual(
                    expect.any(SignedTransaction)
                );
            });
        });

        describe("with asset mint transaction", () => {
            let mintTransaction: MintAsset;
            const shardId = 0;

            beforeAll(async () => {
                mintTransaction = sdk.core
                    .createAssetScheme({
                        shardId,
                        metadata: "metadata",
                        supply: 10,
                        approver: undefined
                    })
                    .createMintTransaction({
                        recipient: await sdk.key.createAssetTransferAddress()
                    });
                await sdk.rpc.chain.sendSignedTransaction(
                    mintTransaction.sign({
                        secret: signerSecret,
                        seq: await sdk.rpc.chain.getSeq(signerAddress),
                        fee: 10
                    })
                );
            });

            test("getTransactionByTracker", async () => {
                expect(
                    ((await sdk.rpc.chain.getTransactionByTracker(
                        mintTransaction.tracker()
                    )) as any).unsigned.actionToJSON()
                ).toEqual((mintTransaction as any).actionToJSON());
            });

            test("getInvoicesByTracker", async () => {
                expect(
                    await sdk.rpc.chain.getInvoicesByTracker(
                        mintTransaction.tracker()
                    )
                ).toEqual(
                    expect.arrayContaining([
                        Invoice.fromJSON({ success: true })
                    ])
                );
            });

            test("getAssetSchemeByTracker", async () => {
                expect(
                    await sdk.rpc.chain.getAssetSchemeByTracker(
                        mintTransaction.tracker(),
                        shardId
                    )
                ).toEqual(expect.any(AssetScheme));
                expect(
                    await sdk.rpc.chain.getAssetSchemeByTracker(
                        invalidHash,
                        shardId
                    )
                ).toBe(null);
            });

            test("getAsset", async () => {
                expect(
                    await sdk.rpc.chain.getAsset(
                        mintTransaction.tracker(),
                        0,
                        shardId
                    )
                ).toEqual(expect.any(Asset));
                expect(
                    await sdk.rpc.chain.getAsset(
                        mintTransaction.tracker(),
                        1,
                        shardId
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.getAsset(invalidHash, 0, shardId)
                ).toBe(null);
            });

            test("isAssetSpent", async () => {
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        0,
                        shardId
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        1,
                        shardId
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.isAssetSpent(invalidHash, 0, shardId)
                ).toBe(null);
            });
        });

        describe("with mint and transfer transaction", () => {
            let mintTransaction: MintAsset;
            let transferTransaction: TransferAsset;
            let blockNumber: number;
            const shardId = 0;
            const wrongShardId = 1;

            beforeAll(async () => {
                mintTransaction = sdk.core
                    .createAssetScheme({
                        shardId,
                        metadata: "metadata",
                        supply: 10,
                        approver: undefined
                    })
                    .createMintTransaction({
                        recipient: await sdk.key.createAssetTransferAddress()
                    });
                const mintedAsset = mintTransaction.getMintedAsset();
                transferTransaction = sdk.core
                    .createTransferAssetTransaction()
                    .addInputs(mintedAsset)
                    .addOutputs({
                        recipient: await sdk.key.createAssetTransferAddress(),
                        quantity: 10,
                        assetType: mintedAsset.assetType,
                        shardId
                    });
                await sdk.key.signTransactionInput(transferTransaction, 0);
                const seq = await sdk.rpc.chain.getSeq(signerAddress);
                await sdk.rpc.chain.sendSignedTransaction(
                    mintTransaction.sign({
                        secret: signerSecret,
                        seq,
                        fee: 10
                    })
                );
                await sdk.rpc.chain.sendSignedTransaction(
                    transferTransaction.sign({
                        secret: signerSecret,
                        seq: seq + 1,
                        fee: 10
                    })
                );
                blockNumber = await sdk.rpc.chain.getBestBlockNumber();
            });

            test("isAssetSpent", async () => {
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        0,
                        shardId
                    )
                ).toBe(true);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        0,
                        shardId,
                        blockNumber - 2
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        0,
                        shardId,
                        blockNumber
                    )
                ).toBe(true);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.tracker(),
                        0,
                        wrongShardId
                    )
                ).toBe(null);

                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.tracker(),
                        0,
                        shardId
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.tracker(),
                        0,
                        shardId,
                        blockNumber - 2
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.tracker(),
                        0,
                        shardId,
                        blockNumber
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.tracker(),
                        0,
                        wrongShardId
                    )
                ).toBe(null);
            });
        });
    });

    describe("account", () => {
        const noSuchAccount = "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhhn9p3";

        test("getList", async () => {
            await expect(sdk.rpc.account.getList()).resolves.toEqual(
                expect.anything()
            );
        });

        test("create", async () => {
            expect(await sdk.rpc.account.create()).toEqual(expect.anything());
            expect(await sdk.rpc.account.create("my-password")).toEqual(
                expect.anything()
            );
        });

        describe("importRaw", () => {
            let randomSecret: string;
            beforeEach(() => {
                randomSecret = sdk.util.generatePrivateKey();
            });

            test("Ok", async () => {
                const account = getAccountIdFromPrivate(randomSecret);
                const address = PlatformAddress.fromAccountId(account, {
                    networkId: CODECHAIN_NETWORK_ID
                });
                // FIXME: Check that address not exists
                expect(await sdk.rpc.account.importRaw(randomSecret)).toEqual(
                    address.toString()
                );
            });

            test("KeyError", done => {
                const invalidSecret =
                    "0000000000000000000000000000000000000000000000000000000000000000";
                sdk.rpc.account
                    .importRaw(invalidSecret)
                    .then(done.fail)
                    .catch(e => {
                        expect(e).toEqual(ERROR.KEY_ERROR);
                        done();
                    });
            });

            test("AlreadyExists", async done => {
                sdk.rpc.account.importRaw(randomSecret).then(() => {
                    sdk.rpc.account
                        .importRaw(randomSecret)
                        .then(() => done.fail())
                        .catch(e => {
                            expect(e).toEqual(ERROR.ALREADY_EXISTS);
                            done();
                        });
                });
            });
        });

        describe("sign", () => {
            const message =
                "0000000000000000000000000000000000000000000000000000000000000000";
            let address: string;
            let secret: string;
            beforeAll(async () => {
                secret = generatePrivateKey();
                address = await sdk.rpc.account.importRaw(
                    secret,
                    "my-password"
                );
            });

            test("Ok", async () => {
                const { r, s, v } = signEcdsa(message, secret);
                const signature = await sdk.rpc.account.sign(
                    message,
                    address,
                    "my-password"
                );
                expect(signature).toContain(r);
                expect(signature).toContain(s);
                expect(signature).toContain(v);
            });

            test("WrongPassword", async done => {
                sdk.rpc.account
                    .sign(message, address, "wrong-password")
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.WRONG_PASSWORD);
                        done();
                    });
            });

            test("NoSuchAccount", async done => {
                sdk.rpc.account
                    .sign(message, noSuchAccount, "my-password")
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.NO_SUCH_ACCOUNT);
                        done();
                    });
            });
        });

        describe("unlock", () => {
            let address: string;
            beforeEach(async () => {
                address = await sdk.rpc.account.create("123");
            });

            test("Ok", async () => {
                await sdk.rpc.account.unlock(address, "123");
                await sdk.rpc.account.unlock(address, "123", 0);
                await sdk.rpc.account.unlock(address, "123", 300);
            });

            test("WrongPassword", async done => {
                sdk.rpc.account
                    .unlock(address, "456")
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.WRONG_PASSWORD);
                        done();
                    });
            });

            test("NoSuchAccount", async done => {
                sdk.rpc.account
                    .unlock(noSuchAccount)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.NO_SUCH_ACCOUNT);
                        done();
                    });
            });
        });

        describe("changePassword", () => {
            let address: string;
            beforeEach(async () => {
                address = await sdk.rpc.account.create("123");
            });

            test("Ok", async () => {
                await sdk.rpc.account.changePassword(address, "123", "456");
                await sdk.rpc.account.changePassword(address, "456", "");
                await sdk.rpc.account.changePassword(address, "", "123");

                const addressWithNoPassphrase = await sdk.rpc.account.create();
                await sdk.rpc.account.changePassword(
                    addressWithNoPassphrase,
                    "",
                    "123"
                );
            });

            test("WrongPassword", async done => {
                sdk.rpc.account
                    .changePassword(address, "456", "123")
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.WRONG_PASSWORD);
                        done();
                    });
            });

            test("NoSuchAccount", async done => {
                sdk.rpc.account
                    .changePassword(noSuchAccount, "123", "456")
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.NO_SUCH_ACCOUNT);
                        done();
                    });
            });
        });
    });
});
