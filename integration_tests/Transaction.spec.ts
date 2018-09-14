import { SDK } from "..";

import {
    CODECHAIN_NETWORK_ID,
    mintAsset,
    sendTransactions,
    SERVER_URL
} from "./helper";

const sdk = new SDK({
    server: SERVER_URL,
    keyStoreType: "memory",
    networkId: CODECHAIN_NETWORK_ID
});
const { H256, AssetTransactionGroup } = SDK.Core.classes;

test("AssetMintTransaction fromJSON", async () => {
    const metadata = "";
    const lockScriptHash = new H256(
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    const amount = 100;
    const registrar = null;
    const { parcelHash } = await mintAsset({
        metadata,
        lockScriptHash,
        amount,
        registrar
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);

    if (!(parcel.unsigned.action instanceof AssetTransactionGroup)) {
        throw Error("Invalid action");
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetMint"),
        metadata: expect.anything(),
        output: {
            lockScriptHash: expect.any(H256),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U256
            amount: expect.anything()
        },
        // FIXME: null or H160
        registrar: null,
        nonce: expect.anything()
    });
});

test("AssetTransferTransaction fromJSON", async () => {
    const addressA = await sdk.key.createAssetTransferAddress();
    const addressB = await sdk.key.createAssetTransferAddress();
    const mintTx = sdk.core
        .createAssetScheme({
            shardId: 0,
            worldId: 0,
            metadata: "metadata of non-permissioned asset",
            amount: 100,
            registrar: null
        })
        .createMintTransaction({ recipient: addressA });
    await sendTransactions({ transactions: [mintTx] });
    const firstAsset = await sdk.rpc.chain.getAsset(mintTx.hash(), 0);
    const transferTx = await sdk.core.createAssetTransferTransaction();
    transferTx.addInputs(firstAsset);
    transferTx.addOutputs({
        assetType: firstAsset.assetType,
        recipient: addressB,
        amount: 100
    });
    sdk.key.signTransactionInput(transferTx, 0);
    const { parcelHash } = await sendTransactions({
        transactions: [transferTx]
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);
    // FIXME: Remove anythings when *Data fields are flattened
    const expectedInput = expect.objectContaining({
        prevOut: expect.objectContaining({
            transactionHash: expect.any(H256),
            index: expect.anything(),
            assetType: expect.any(H256),
            amount: expect.anything()
        }),
        lockScript: expect.anything(),
        unlockScript: expect.anything()
    });
    const expectedOutput = expect.objectContaining({
        lockScriptHash: expect.anything(),
        parameters: [],
        assetType: expect.anything(),
        amount: expect.anything()
    });

    if (!(parcel.unsigned.action instanceof AssetTransactionGroup)) {
        throw Error("Invalid action");
    }

    expect(parcel.unsigned.action.transactions[0]).toMatchObject({
        type: expect.stringMatching("assetTransfer"),
        burns: [],
        inputs: expect.arrayContaining([expectedInput]),
        outputs: expect.anything(),
        networkId: expect.anything(),
        nonce: expect.anything()
    });
});
