import { SDK } from "../";
import {
    Asset,
    AssetMintTransaction,
    AssetScheme,
    AssetTransferTransaction,
    H256,
    Invoice,
    Parcel,
    PlatformAddress,
    SignedParcel
} from "../lib/core/classes";
import {
    generatePrivateKey,
    getAccountIdFromPrivate,
    signEcdsa
} from "../lib/utils";
import { U256 } from "../src/core/U256";

import {
    ACCOUNT_ADDRESS,
    ACCOUNT_ID,
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
    const signerSecret = ACCOUNT_SECRET;
    const signerAccount = ACCOUNT_ID;
    const signerAddress = ACCOUNT_ADDRESS;

    beforeAll(async () => {
        sdk = new SDK({
            server: SERVER_URL,
            keyStoreType: "memory",
            networkId: CODECHAIN_NETWORK_ID
        });
    });

    test("PlatformAddress", () => {
        expect(
            sdk.core.classes.PlatformAddress.fromAccountId(signerAccount).value
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
            const account = "0x6fe64ffa3a46c074226457c90ccb32dc06ccced1";
            const address = "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";

            test("PlatformAddress", () => {
                expect(
                    sdk.core.classes.PlatformAddress.fromAccountId(account)
                        .value
                ).toEqual(address);
            });

            test("getBalance", async () => {
                expect(await sdk.rpc.chain.getBalance(address)).toEqual(
                    expect.any(U256)
                );
            });

            test("getSeq", async () => {
                expect(await sdk.rpc.chain.getSeq(address)).toEqual(
                    expect.any(U256)
                );
            });

            describe("has regular key", () => {
                const regularKey =
                    "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b";

                beforeAll(async () => {
                    const parcel = sdk.core
                        .createSetRegularKeyParcel({
                            key: regularKey
                        })
                        .sign({
                            secret: signerSecret,
                            seq: await sdk.rpc.chain.getSeq(signerAddress),
                            fee: 10
                        });
                    await sdk.rpc.chain.sendSignedParcel(parcel);
                });

                test("getRegularKey", async () => {
                    const regularKeyOfAddress = await sdk.rpc.chain.getRegularKey(
                        address
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
                    expect(keyOwner.toString()).toEqual(signerAddress);
                });
            });
        });

        describe("sendSignedParcel", () => {
            const secret = signerSecret;
            let seq: U256;
            let parcel: Parcel;
            beforeEach(async () => {
                parcel = sdk.core.createPaymentParcel({
                    recipient: signerAddress,
                    amount: 10
                });
                seq = await sdk.rpc.chain.getSeq(signerAddress);
            });

            test("Ok", async done => {
                sdk.rpc.chain
                    .sendSignedParcel(parcel.sign({ secret, fee: 10, seq }))
                    .then(() => done())
                    .catch(e => done.fail(e));
            });

            test("VerificationFailed", done => {
                const signedParcel = parcel.sign({ secret, fee: 10, seq });
                signedParcel.r = new U256(0);
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.VERIFICATION_FAILED);
                        done();
                    });
            });

            test("AlreadyImported", done => {
                const signedParcel = parcel.sign({ secret, fee: 10, seq });
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
                    .then(() => {
                        sdk.rpc.chain
                            .sendSignedParcel(signedParcel)
                            .then(() => done.fail())
                            .catch(e => {
                                expect(e).toEqual(ERROR.ALREADY_IMPORTED);
                                done();
                            });
                    })
                    .catch(done.fail);
            });

            test("NotEnoughBalance", async done => {
                const signedParcel = parcel.sign({
                    secret,
                    fee: new U256(
                        "0xffffffffffffffffffffffffffffffffffffffffffffffffff"
                    ),
                    seq
                });
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.NOT_ENOUGH_BALANCE);
                        done();
                    });
            });

            test("TooLowFee", done => {
                const signedParcel = parcel.sign({ secret, fee: 9, seq });
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
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
                const signedParcel = parcel.sign({
                    secret,
                    fee: 12321,
                    seq: new U256(seq.value.minus(1))
                });
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.INVALID_SEQ);
                        done();
                    });
            });

            test("InvalidNetworkId", done => {
                (parcel as any).networkId = "zz";
                const signedParcel = parcel.sign({ secret, fee: 10, seq });
                sdk.rpc.chain
                    .sendSignedParcel(signedParcel)
                    .then(() => done.fail())
                    .catch(e => {
                        expect(e).toEqual(ERROR.INVALID_NETWORK_ID);
                        done();
                    });
            });
        });

        describe("with parcel hash", () => {
            let parcelHash: H256;

            beforeAll(async () => {
                const parcel = sdk.core.createPaymentParcel({
                    recipient: signerAddress,
                    amount: 10
                });
                const signedParcel = parcel.sign({
                    secret: signerSecret,
                    fee: 10,
                    seq: await sdk.rpc.chain.getSeq(signerAddress)
                });
                parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
            });

            test("getParcel", async () => {
                expect(await sdk.rpc.chain.getParcel(parcelHash)).toEqual(
                    expect.any(SignedParcel)
                );
            });

            test("getParcelInvoice", async () => {
                expect(
                    await sdk.rpc.chain.getParcelInvoice(parcelHash)
                ).toEqual(expect.any(Invoice));
                expect(await sdk.rpc.chain.getParcelInvoice(invalidHash)).toBe(
                    null
                );
            });
        });

        describe.skip("with pending parcels", () => {
            test("getPendingParcels", async () => {
                const pendingParcels = await sdk.rpc.chain.getPendingParcels();
                expect(pendingParcels[0]).toEqual(expect.any(SignedParcel));
            });
        });

        describe("with asset mint transaction", () => {
            let mintTransaction: AssetMintTransaction;
            const shardId = 0;

            beforeAll(async () => {
                mintTransaction = sdk.core
                    .createAssetScheme({
                        shardId,
                        metadata: "metadata",
                        amount: 10,
                        registrar: undefined
                    })
                    .createMintTransaction({
                        recipient: await sdk.key.createAssetTransferAddress()
                    });
                const parcel = sdk.core.createAssetTransactionParcel({
                    transaction: mintTransaction
                });
                await sdk.rpc.chain.sendSignedParcel(
                    parcel.sign({
                        secret: signerSecret,
                        seq: await sdk.rpc.chain.getSeq(signerAddress),
                        fee: 10
                    })
                );
            });

            test("getTransaction", async () => {
                expect(
                    await sdk.rpc.chain.getTransaction(mintTransaction.hash())
                ).toEqual(mintTransaction);
            });

            test("getTransactionInvoice", async () => {
                expect(
                    await sdk.rpc.chain.getTransactionInvoice(
                        mintTransaction.hash()
                    )
                ).toEqual(expect.any(Invoice));
            });

            test("getAssetScheme", async () => {
                const shardId = 0;
                expect(
                    await sdk.rpc.chain.getAssetSchemeByHash(
                        mintTransaction.hash(),
                        shardId
                    )
                ).toEqual(expect.any(AssetScheme));
                expect(
                    await sdk.rpc.chain.getAssetSchemeByHash(
                        invalidHash,
                        shardId
                    )
                ).toBe(null);
            });

            test("getAsset", async () => {
                expect(
                    await sdk.rpc.chain.getAsset(mintTransaction.hash(), 0)
                ).toEqual(expect.any(Asset));
                expect(
                    await sdk.rpc.chain.getAsset(mintTransaction.hash(), 1)
                ).toBe(null);
                expect(await sdk.rpc.chain.getAsset(invalidHash, 0)).toBe(null);
            });

            test("isAssetSpent", async () => {
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
                        0,
                        shardId
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
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
            let mintTransaction: AssetMintTransaction;
            let transferTransaction: AssetTransferTransaction;
            let blockNumber: number;
            const shardId = 0;
            const wrongShardId = 1;

            beforeAll(async () => {
                mintTransaction = sdk.core
                    .createAssetScheme({
                        shardId,
                        metadata: "metadata",
                        amount: 10,
                        registrar: undefined
                    })
                    .createMintTransaction({
                        recipient: await sdk.key.createAssetTransferAddress()
                    });
                const mintedAsset = mintTransaction.getMintedAsset();
                transferTransaction = sdk.core
                    .createAssetTransferTransaction()
                    .addInputs(mintedAsset)
                    .addOutputs({
                        recipient: await sdk.key.createAssetTransferAddress(),
                        amount: 10,
                        assetType: mintedAsset.assetType
                    });
                await sdk.key.signTransactionInput(transferTransaction, 0);
                const mintParcel = sdk.core.createAssetTransactionParcel({
                    transaction: mintTransaction
                });
                const seq = await sdk.rpc.chain.getSeq(signerAddress);
                await sdk.rpc.chain.sendSignedParcel(
                    mintParcel.sign({
                        secret: signerSecret,
                        seq,
                        fee: 10
                    })
                );
                const transferParcel = sdk.core.createAssetTransactionParcel({
                    transaction: transferTransaction
                });
                await sdk.rpc.chain.sendSignedParcel(
                    transferParcel.sign({
                        secret: signerSecret,
                        seq: seq.increase(),
                        fee: 10
                    })
                );
                blockNumber = await sdk.rpc.chain.getBestBlockNumber();
            });

            test("isAssetSpent", async () => {
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
                        0,
                        shardId
                    )
                ).toBe(true);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
                        0,
                        shardId,
                        blockNumber - 2
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
                        0,
                        shardId,
                        blockNumber
                    )
                ).toBe(true);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        mintTransaction.hash(),
                        0,
                        wrongShardId
                    )
                ).toBe(null);

                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.hash(),
                        0,
                        shardId
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.hash(),
                        0,
                        shardId,
                        blockNumber - 2
                    )
                ).toBe(null);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.hash(),
                        0,
                        shardId,
                        blockNumber
                    )
                ).toBe(false);
                expect(
                    await sdk.rpc.chain.isAssetSpent(
                        transferTransaction.hash(),
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
    });
});
