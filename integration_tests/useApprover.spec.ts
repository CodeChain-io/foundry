import {
    PlatformAddress,
    PlatformAddressValue
} from "codechain-primitives/lib";
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
let masterSecret: string;
let masterAddress: PlatformAddress;

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
    masterSecret = sdk.util.generatePrivateKey();
    const account = sdk.util.getAccountIdFromPrivate(masterSecret);
    masterAddress = sdk.core.classes.PlatformAddress.fromAccountId(account, {
        networkId: "tc"
    });
    await sendCCCToOther(ACCOUNT_ADDRESS, masterAddress, ACCOUNT_SECRET, 1_000);

    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const mintTx = await mintAssetUsingMaster(aliceAddress);
    await setRegularKey();
    await sendCCCToOther(masterAddress, otherAddress, regularSecret, 100);

    const bobAddress = "tcaqyqckq0zgdxgpck6tjdg4qmp52p2vx3qaexqnegylk";

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

    await sdk.rpc.chain.getTransactionResult(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function sendCCCToOther(
    address: PlatformAddressValue,
    recipient: PlatformAddress,
    secret: string,
    quantity: number
) {
    const seq = await sdk.rpc.chain.getSeq(address);
    const p = sdk.core.createPayTransaction({
        recipient,
        quantity
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        p.sign({
            secret,
            seq,
            fee: 10
        })
    );

    const result = await sdk.rpc.chain.getTransactionResult(hash, {
        timeout: 5 * 60 * 1000
    });
    expect(result).toBeTruthy();
    expect(result!).toBe(true);
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

    const mintTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        mintTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(mintTxResults.length).toBe(1);
    expect(mintTxResults[0]).toBe(true);
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

    const transferTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        transferTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxResults.length).toBe(2);
    expect(transferTxResults[1]).toBe(true);
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

    const transferTxResults = await sdk.rpc.chain.getTransactionResultsByTracker(
        transferTx.tracker(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxResults.length).toBe(1);
    expect(transferTxResults[0]).toBe(false);
}
