import { SDK } from "../";
import { H256, SignedParcel, Invoice, AssetMintTransaction, Asset, AssetScheme, AssetTransferTransaction } from "../lib/core/classes";
import { PlatformAddress } from "../lib/key/classes";
import { getAccountIdFromPrivate, generatePrivateKey, signEcdsa } from "../lib/utils";

describe("rpc", () => {
    let sdk: SDK;
    const { Block, H256, H512 , U256 } = SDK.Core.classes;
    const invalidHash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const signerAccount = "0xa6594b7196808d161b6fb137e781abbc251385d9";
    const signerAddress = "cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7";

    beforeAll(async () => {
        sdk = new SDK({ server: "http://localhost:8080" });
    });

    describe("node", () => {
        test("ping", async () => {
            expect(await sdk.rpc.node.ping()).toBe("pong");
        });

        test("getNodeVersion", async () => {
            // FIXME: regex for semver
            expect(typeof await sdk.rpc.node.getNodeVersion()).toBe("string");
        });
    });

    test("getBestBlockNumber", async () => {
        expect(typeof await sdk.rpc.chain.getBestBlockNumber()).toBe("number");
    });

    describe("chain", () => {
        test("getBlockHash", async () => {
            expect(await sdk.rpc.chain.getBlockHash(0)).toEqual(expect.any(H256));
            expect(await sdk.rpc.chain.getBlockHash(9999999999)).toEqual(null);
        });

        test("getBlock - by number", async () => {
            expect(await sdk.rpc.chain.getBlock(0)).toEqual(expect.any(Block));
            expect(await sdk.rpc.chain.getBlock(9999999999)).toEqual(null);
        });

        test("getBlock - by hash", async () => {
            const hash = await sdk.rpc.chain.getBlockHash(0);
            expect(await sdk.rpc.chain.getBlock(hash)).toEqual(expect.any(Block));
            expect(await sdk.rpc.chain.getBlock(hash.value)).toEqual(expect.any(Block));

            expect(await sdk.rpc.chain.getBlock(invalidHash)).toEqual(null);
        });

        describe("with account", () => {
            const account = "0xa6594b7196808d161b6fb137e781abbc251385d9";

            test("getBalance", async () => {
                expect(await sdk.rpc.chain.getBalance(account)).toEqual(expect.any(U256));
            });

            test("getNonce", async () => {
                expect(await sdk.rpc.chain.getNonce(account)).toEqual(expect.any(U256));
            });

            // FIXME: setRegularKey action isn't implemented.
            test.skip("getRegularKey", async () => {
                expect(await sdk.rpc.chain.getRegularKey(account)).toEqual(expect.any(H512));
            });
        });

        // FIXME: with address here.

        describe("with parcel hash", () => {
            let parcelHash: H256;

            beforeAll(async () => {
                const parcel = sdk.core.createPaymentParcel({
                    recipient: signerAccount,
                    amount: 10,
                });
                const signedParcel = parcel.sign({
                    secret: signerSecret,
                    fee: 10,
                    nonce: await sdk.rpc.chain.getNonce(signerAccount),
                });
                parcelHash = await sdk.rpc.chain.sendSignedParcel(signedParcel);
            });

            test("getParcel", async () => {
                expect(await sdk.rpc.chain.getParcel(parcelHash)).toEqual(expect.any(SignedParcel));
            });

            test("getParcelInvoice", async () => {
                expect(await sdk.rpc.chain.getParcelInvoice(parcelHash)).toEqual(expect.any(Invoice));
                expect(await sdk.rpc.chain.getParcelInvoice(invalidHash)).toBe(null);
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
            const worldId = 0;

            beforeAll(async () => {
                const keyStore = await sdk.key.createMemoryKeyStore();
                const p2pkh = await sdk.key.createP2PKH({ keyStore });
                mintTransaction = sdk.core.createAssetScheme({
                    shardId,
                    worldId,
                    metadata: "metadata",
                    amount: 10,
                    registrar: null
                }).createMintTransaction({ recipient: await p2pkh.createAddress() });
                const parcel = sdk.core.createChangeShardStateParcel({
                    transactions: [mintTransaction],
                });
                await sdk.rpc.chain.sendSignedParcel(parcel.sign({
                    secret: signerSecret,
                    nonce: await sdk.rpc.chain.getNonce(signerAccount),
                    fee: 10
                }));
            });

            test("getTransaction", async () => {
                expect(await sdk.rpc.chain.getTransaction(mintTransaction.hash())).toEqual(mintTransaction);
            });

            test("getTransactionInvoice", async () => {
                expect(await sdk.rpc.chain.getTransactionInvoice(mintTransaction.hash())).toEqual(expect.any(Invoice));
            });

            test("getAssetScheme", async () => {
                const shardId = 0;
                const worldId = 0;
                expect(await sdk.rpc.chain.getAssetSchemeByHash(mintTransaction.hash(), shardId, worldId)).toEqual(expect.any(AssetScheme));
                expect(await sdk.rpc.chain.getAssetSchemeByHash(invalidHash, shardId, worldId)).toBe(null);
            });

            test("getAsset", async () => {
                expect(await sdk.rpc.chain.getAsset(mintTransaction.hash(), 0)).toEqual(expect.any(Asset));
                expect(await sdk.rpc.chain.getAsset(mintTransaction.hash(), 1)).toBe(null);
                expect(await sdk.rpc.chain.getAsset(invalidHash, 0)).toBe(null);
            });

            test("isAssetSpent", async () => {
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 0, shardId)).toBe(false);
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 1, shardId)).toBe(null);
                expect(await sdk.rpc.chain.isAssetSpent(invalidHash, 0, shardId)).toBe(null);
            });
        });

        describe("with mint and transfer transaction", () => {
            let mintTransaction: AssetMintTransaction;
            let transferTransaction: AssetTransferTransaction;
            let blockNumber: number;
            const shardId = 0;
            const worldId = 0;
            const wrongShardId = 1;

            beforeAll(async () => {
                const keyStore = await sdk.key.createMemoryKeyStore();
                const p2pkh = await sdk.key.createP2PKH({ keyStore });
                mintTransaction = sdk.core.createAssetScheme({
                    shardId,
                    worldId,
                    metadata: "metadata",
                    amount: 10,
                    registrar: null
                }).createMintTransaction({ recipient: await p2pkh.createAddress() });
                const mintedAsset = mintTransaction.getMintedAsset();
                transferTransaction = sdk.core.createAssetTransferTransaction()
                    .addInputs(mintedAsset)
                    .addOutputs({
                        recipient: await p2pkh.createAddress(),
                        amount: 10,
                        assetType: mintedAsset.assetType,
                    });
                await transferTransaction.sign(0, { signer: p2pkh });
                const parcel = sdk.core.createChangeShardStateParcel({ transactions: [mintTransaction, transferTransaction] });
                await sdk.rpc.chain.sendSignedParcel(parcel.sign({
                    secret: signerSecret,
                    nonce: await sdk.rpc.chain.getNonce(signerAccount),
                    fee: 10
                }));
                blockNumber = await sdk.rpc.chain.getBestBlockNumber();
            });

            test("isAssetSpent", async () => {
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 0, shardId)).toBe(true);
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 0, shardId, blockNumber - 1)).toBe(null);
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 0, shardId, blockNumber)).toBe(true);
                expect(await sdk.rpc.chain.isAssetSpent(mintTransaction.hash(), 0, wrongShardId)).toBe(null);

                expect(await sdk.rpc.chain.isAssetSpent(transferTransaction.hash(), 0, shardId)).toBe(false);
                expect(await sdk.rpc.chain.isAssetSpent(transferTransaction.hash(), 0, shardId, blockNumber - 1)).toBe(null);
                expect(await sdk.rpc.chain.isAssetSpent(transferTransaction.hash(), 0, shardId, blockNumber)).toBe(false);
                expect(await sdk.rpc.chain.isAssetSpent(transferTransaction.hash(), 0, wrongShardId)).toBe(null);
            });
        });
    });

    describe("account", () => {
        test("getList", async () => {
            expect(async () => {
                await sdk.rpc.account.getList();
            }).not.toThrow();
        });

        test("create", async () => {
            expect(await sdk.rpc.account.create()).toEqual(expect.anything());
            expect(await sdk.rpc.account.create("my-password")).toEqual(expect.anything());
        });

        test("importRaw", async () => {
            const secret = "a2b39d4aefecdb17f84ed4cf629e7c8817691cc4f444ac7522902b8fb4b7bd53";
            const account = getAccountIdFromPrivate(secret);
            const address = PlatformAddress.fromAccountId(account);
            expect(address.toString()).toEqual("cccqz3z4e3x6f5j80wexg0xfr0qsrqcuyzf7g4y0je6");
            expect(await sdk.rpc.account.importRaw(secret)).toEqual(address.toString());
        });

        test("remove", async () => {
            const account = await sdk.rpc.account.create("123");
            expect(async () => {
                await sdk.rpc.account.remove(account, "123");
                expect(await sdk.rpc.account.getList()).not.toContain(account);
            }).not.toThrow();
        });

        test("sign", async () => {
            const secret = generatePrivateKey();
            const account = await sdk.rpc.account.importRaw(secret, "my-password");

            const message = "0000000000000000000000000000000000000000000000000000000000000000";
            const { r, s, v } = signEcdsa(message, secret);
            // FIXME:
            const sig = await sdk.rpc.account.sign(message, account, "my-password");
            expect(sig).toContain(r);
            expect(sig).toContain(s);
            expect(sig).toContain(v);
        });
    });
});
