import { SDK, H256, AssetMintTransaction, MemoryKeyStore, PubkeyAssetAgent, ChangeShardState } from "../";
import { mintAsset, sendTransactions } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("AssetMintTransaction fromJSON", async () => {
    const metadata = "";
    const lockScriptHash = new H256("0000000000000000000000000000000000000000000000000000000000000000");
    const amount = 100;
    const registrar = null;
    const { parcelHash } = await mintAsset({ metadata, lockScriptHash, amount, registrar });
    const parcel = await sdk.getParcel(parcelHash);

    if (!(parcel.unsigned.action instanceof ChangeShardState)) {
        throw "Invalid action";
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetMint"),
        data: expect.objectContaining({
            metadata: expect.anything(),
            lockScriptHash: expect.any(H256),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U256
            amount: expect.anything(),
            // FIXME: null or H160
            registrar: null,
            nonce: expect.anything()
        })
    });
});

test("AssetTransferTransaction fromJSON", async () => {
    const pubkeyAssetAgent = sdk.getAssetAgent();

    const mintTx = sdk.createAssetScheme({
        metadata: "metadata of non-permissioned asset",
        amount: 100,
        registrar: null,
    }).mint(await pubkeyAssetAgent.createAddress());
    await sendTransactions({ transactions: [mintTx] });
    const firstAsset = await sdk.getAsset(mintTx.hash(), 0);

    const transferTx = await firstAsset.transfer(pubkeyAssetAgent, [{
        address: await pubkeyAssetAgent.createAddress(),
        amount: 100
    }]);
    const { parcelHash } = await sendTransactions({ transactions: [transferTx] });
    const parcel = await sdk.getParcel(parcelHash);
    // FIXME: Remove anythings when *Data fields are flattened
    const expectedInput = expect.objectContaining({
        prevOut: expect.objectContaining({
            data: expect.objectContaining({
                transactionHash: expect.any(H256),
                index: expect.anything(),
                assetType: expect.any(H256),
                amount: expect.anything(),
            })
        }),
        lockScript: expect.anything(),
        unlockScript: expect.anything(),
    });
    const expectedOutput = expect.objectContaining({
        lockScriptHash: expect.anything(),
        parameters: [],
        assetType: expect.anything(),
        amount: expect.anything(),
    });

    if (!(parcel.unsigned.action instanceof ChangeShardState)) {
        throw "Invalid action";
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetTransfer"),
        burns: [],
        inputs: expect.arrayContaining([expectedInput]),
        outputs: expect.anything(),
        networkId: expect.anything(),
        nonce: expect.anything(),
    });
});
