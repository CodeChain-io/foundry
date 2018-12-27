import { SDK } from "..";
import { AssetTransaction } from "../src/core/parcel/AssetTransaction";

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

test("AssetMintTransaction fromJSONToParcel", async () => {
    const metadata = "";
    const lockScriptHash = new H160("0000000000000000000000000000000000000000");
    const amount = 100;
    const approver = null;
    const { parcelHash } = await mintAsset({
        metadata,
        lockScriptHash,
        amount,
        approver
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);
    if (parcel == null) {
        throw Error("Cannot get the parcel");
    }

    const action = parcel.unsigned.action();
    if (action !== "assetTransaction") {
        throw Error("Invalid action");
    }

    expect(
        ((parcel.unsigned as any) as AssetTransaction).transaction()
    ).toMatchObject({
        type: expect.stringMatching("assetMint"),
        metadata: expect.anything(),
        output: {
            lockScriptHash: expect.any(H160),
            // FIXME: Buffer[]
            parameters: expect.anything(),
            // FIXME: Change it to U64
            amount: expect.anything()
        },
        // FIXME: null or H160
        approver: null
    });
});

test("AssetTransferTransaction fromJSONToParcel", async () => {
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
    const firstAsset = await sdk.rpc.chain.getAsset(mintTx.hash(), 0);
    if (firstAsset == null) {
        throw Error("Cannot get the first asset");
    }
    const transferTx = await sdk.core.createAssetTransferTransaction();
    transferTx.addInputs(firstAsset);
    transferTx.addOutputs({
        assetType: firstAsset.assetType,
        recipient: addressB,
        amount: 100
    });
    await sdk.key.signTransactionInput(transferTx, 0);
    const { parcelHash } = await sendTransaction({
        transaction: transferTx
    });
    const parcel = await sdk.rpc.chain.getParcel(parcelHash);
    if (parcel == null) {
        throw Error("Cannot get a parcel");
    }
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

    if (parcel.unsigned.action() !== "assetTransaction") {
        throw Error("Invalid action");
    }

    expect(
        ((parcel.unsigned as any) as AssetTransaction).transaction()
    ).toMatchObject({
        type: expect.stringMatching("assetTransfer"),
        burns: [],
        inputs: expect.arrayContaining([expectedInput]),
        outputs: expect.anything(),
        networkId: expect.anything()
    });
});
