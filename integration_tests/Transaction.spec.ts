import { SDK } from "..";

import {
    CODECHAIN_NETWORK_ID,
    mintAsset,
    sendTransaction,
    SERVER_URL
} from "./helper";

const sdk = new SDK({
    server: SERVER_URL,
    keyStoreType: "memory",
    networkId: CODECHAIN_NETWORK_ID
});
const { H160, H256 } = SDK.Core.classes;

test("AssetMintTransaction fromJSONToTransaction", async () => {
    const metadata = "";
    const lockScriptHash = new H160("0000000000000000000000000000000000000000");
    const amount = 100;
    const approver = null;
    const { hash } = await mintAsset({
        metadata,
        lockScriptHash,
        amount,
        approver
    });
    const tx = await sdk.rpc.chain.getTransaction(hash);
    if (tx == null) {
        throw Error("Cannot get the tx");
    }

    expect(tx.unsigned.type()).toEqual("mintAsset");

    expect(tx.unsigned.toJSON().action).toMatchObject({
        metadata: expect.anything(),
        output: {
            lockScriptHash: expect.anything(),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U64
            amount: expect.anything()
        },
        // FIXME: null or H160
        approver: null
    });
});

test("AssetTransferTransaction fromJSONToTransaction", async () => {
    const addressA = await sdk.key.createAssetTransferAddress();
    const addressB = await sdk.key.createAssetTransferAddress();
    const mintTx = sdk.core
        .createAssetScheme({
            shardId: 0,
            metadata: "metadata of non-permissioned asset",
            amount: 100,
            approver: undefined
        })
        .createMintTransaction({ recipient: addressA });
    await sendTransaction({ transaction: mintTx });
    const firstAsset = await sdk.rpc.chain.getAsset(mintTx.id(), 0);
    if (firstAsset == null) {
        throw Error("Cannot get the first asset");
    }
    const transferTx = await sdk.core.createTransferAssetTransaction();
    transferTx.addInputs(firstAsset);
    transferTx.addOutputs({
        assetType: firstAsset.assetType,
        recipient: addressB,
        amount: 100
    });
    await sdk.key.signTransactionInput(transferTx, 0);
    const { hash } = await sendTransaction({
        transaction: transferTx
    });
    const tx = await sdk.rpc.chain.getTransaction(hash);
    if (tx == null) {
        throw Error("Cannot get a tx");
    }
    // FIXME: Remove anythings when *Data fields are flattened
    const expectedInput = expect.objectContaining({
        prevOut: expect.objectContaining({
            transactionId: expect.any(H256),
            index: expect.anything(),
            assetType: expect.any(H256),
            amount: expect.anything()
        }),
        lockScript: expect.anything(),
        unlockScript: expect.anything()
    });

    expect(tx.unsigned.type()).toEqual("transferAsset");
    expect((tx.unsigned as any)._transaction).toMatchObject({
        burns: [],
        inputs: expect.arrayContaining([expectedInput]),
        outputs: expect.anything(),
        networkId: expect.anything()
    });
});
