import { SDK } from "../";
import { H256, SignedParcel, Invoice, AssetMintTransaction, Asset, AssetScheme } from "../lib/core/classes";

describe("rpc", () => {
    let sdk: SDK;
    const { Block, H256, H512 , U256 } = SDK.Core.classes;
    const invalidHash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const signerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const signerAccount = "0xa6594b7196808d161b6fb137e781abbc251385d9";

    beforeAll(async () => {
        sdk = new SDK({ server: "http://localhost:8080" });
    });

    test("ping", async () => {
        expect(await sdk.rpc.node.ping()).toBe("pong");
    });

    test("getNodeVersion", async () => {
        // FIXME: regex for semver
        expect(typeof await sdk.rpc.node.getNodeVersion()).toBe("string");
    });

    test("getBestBlockNumber", async () => {
        expect(typeof await sdk.rpc.chain.getBestBlockNumber()).toBe("number");
    });

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

    describe("with parcel hash", () => {
        let parcelHash: H256;

        beforeAll(async () => {
            const parcel = sdk.core.createPaymentParcel({
                recipient: signerAccount,
                amount: 10,
                fee: 10,
                nonce: await sdk.rpc.chain.getNonce(signerAccount),
            });
            const signedParcel = parcel.sign(signerSecret);
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

        beforeAll(async () => {
            mintTransaction = sdk.core.createAssetScheme({
                metadata: "metadata",
                amount: 10,
                registrar: null
            }).mint(await sdk.key.createPubKeyAddress());
            const parcel = sdk.core.createChangeShardStateParcel({
                transactions: [mintTransaction],
                nonce: await sdk.rpc.chain.getNonce(signerAccount),
                fee: 10
            });
            await sdk.rpc.chain.sendSignedParcel(parcel.sign(signerSecret));
        });

        test("getTransactionInvoice", async () => {
            expect(await sdk.rpc.chain.getTransactionInvoice(mintTransaction.hash())).toEqual(expect.any(Invoice));
        });

        test("getAssetScheme", async () => {
            expect(await sdk.rpc.chain.getAssetScheme(mintTransaction.hash())).toEqual(expect.any(AssetScheme));
            expect(await sdk.rpc.chain.getAssetScheme(invalidHash)).toBe(null);
        });

        test("getAsset", async () => {
            expect(await sdk.rpc.chain.getAsset(mintTransaction.hash(), 0)).toEqual(expect.any(Asset));
            expect(await sdk.rpc.chain.getAsset(mintTransaction.hash(), 1)).toBe(null);
            expect(await sdk.rpc.chain.getAsset(invalidHash, 0)).toBe(null);
        });
    });
});
