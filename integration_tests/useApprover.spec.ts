import { SDK } from "../src";
import { AssetTransferAddress, MintAsset } from "../src/core/classes";

import {
    ACCOUNT_ADDRESS,
    ACCOUNT_SECRET,
    CODECHAIN_NETWORK_ID,
    SERVER_URL
} from "./helper";

const sdk = new SDK({
    server: SERVER_URL,
    keyStoreType: "memory",
    networkId: CODECHAIN_NETWORK_ID
});
const masterSecret = ACCOUNT_SECRET;
const masterAddress = ACCOUNT_ADDRESS;

const otherSecret =
    "0000000000000000000000000000000000000000000000000000000000000001";
const otherAccountId = SDK.util.getAccountIdFromPrivate(otherSecret);
const otherAddress = sdk.core.classes.PlatformAddress.fromAccountId(
    otherAccountId,
    { networkId: CODECHAIN_NETWORK_ID }
);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

test("checkApproverValidation", async () => {
    await setRegularKey();
    await sendCCCToOther();

    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const bobAddress = "tcaqyqckq0zgdxgpck6tjdg4qmp52p2vx3qaexqnegylk";

    const mintTx = await mintAssetUsingMaster(aliceAddress);
    await transferAssetUsingOther(mintTx, aliceAddress, bobAddress);
    await transferAssetUsingRegular(mintTx, aliceAddress, bobAddress);
});

async function setRegularKey() {
    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    const p = sdk.core.createSetRegularKeyTransaction({
        key: regularPublic
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        p.sign({
            secret: masterSecret,
            seq,
            fee: 10
        })
    );

    await sdk.rpc.chain.getInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function sendCCCToOther() {
    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    const p = sdk.core.createPayTransaction({
        recipient: otherAddress,
        quantity: 100
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        p.sign({
            secret: regularSecret,
            seq,
            fee: 10
        })
    );

    await sdk.rpc.chain.getInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function mintAssetUsingMaster(
    aliceAddress: AssetTransferAddress
): Promise<MintAsset> {
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        }),
        supply: 10000,
        approver: masterAddress
    });

    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    await sdk.rpc.chain.sendSignedTransaction(
        mintTx.sign({
            secret: masterSecret,
            seq,
            fee: 10
        })
    );

    const mintTxInvoices = await sdk.rpc.chain.getInvoicesByTracker(
        mintTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(mintTxInvoices.length).toBe(1);
    expect(mintTxInvoices[0].success).toBe(true);
    return mintTx;
}

async function transferAssetUsingRegular(
    mintTx: MintAsset,
    aliceAddress: AssetTransferAddress,
    bobAddress: string
) {
    const asset = mintTx.getMintedAsset();
    const transferTx = sdk.core
        .createTransferAssetTransaction()
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                quantity: 3000,
                assetType: asset.assetType,
                shardId: asset.shardId
            },
            {
                recipient: aliceAddress,
                quantity: 7000,
                assetType: asset.assetType,
                shardId: asset.shardId
            }
        );
    await sdk.key.signTransactionInput(transferTx, 0);

    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    await sdk.rpc.chain.sendSignedTransaction(
        transferTx.sign({
            secret: regularSecret,
            seq,
            fee: 10
        })
    );

    const transferTxInvoices = await sdk.rpc.chain.getInvoicesByTracker(
        transferTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxInvoices.length).toBe(2);
    expect(transferTxInvoices[1].success).toBe(true);
}
async function transferAssetUsingOther(
    mintTx: MintAsset,
    aliceAddress: AssetTransferAddress,
    bobAddress: string
) {
    const asset = mintTx.getMintedAsset();

    const transferTx = sdk.core
        .createTransferAssetTransaction({
            burns: [],
            inputs: [],
            outputs: []
        })
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                quantity: 3000,
                assetType: asset.assetType,
                shardId: asset.shardId
            },
            {
                recipient: aliceAddress,
                quantity: 7000,
                assetType: asset.assetType,
                shardId: asset.shardId
            }
        );
    await sdk.key.signTransactionInput(transferTx, 0);

    const seq = await sdk.rpc.chain.getSeq(otherAddress);
    await sdk.rpc.chain.sendSignedTransaction(
        transferTx.sign({
            secret: otherSecret,
            seq,
            fee: 10
        })
    );

    const transferTxInvoices = await sdk.rpc.chain.getInvoicesByTracker(
        transferTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxInvoices.length).toBe(1);
    expect(transferTxInvoices[0].success).toBe(false);
}
