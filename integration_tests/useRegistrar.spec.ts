import { SDK } from "../";

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
    otherAccountId
);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

test("checkRegistrarValidation", async () => {
    await setRegularKey();
    await sendCCCToOther();

    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const bobAddress =
        "tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze";

    const mintTx = await mintAssetUsingMaster(aliceAddress, bobAddress);
    await transferAssetUsingOther(mintTx, aliceAddress, bobAddress);
    await transferAssetUsingRegular(mintTx, aliceAddress, bobAddress);
});

async function setRegularKey() {
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    const p = sdk.core.createSetRegularKeyParcel({
        key: regularPublic
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: masterSecret,
            nonce,
            fee: 10
        })
    );

    await sdk.rpc.chain.getParcelInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function sendCCCToOther() {
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    const p = sdk.core.createPaymentParcel({
        recipient: otherAddress,
        amount: 100
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: regularSecret,
            nonce,
            fee: 10
        })
    );

    await sdk.rpc.chain.getParcelInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function mintAssetUsingMaster(aliceAddress, bobAddress) {
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        worldId: 0,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        }),
        amount: 10000,
        registrar: masterAddress
    });

    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const p = sdk.core.createAssetTransactionGroupParcel({
        transactions: [mintTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: masterSecret,
            nonce,
            fee: 10
        })
    );

    const mintTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        mintTx.hash(),
        {
            timeout: 5 * 60 * 1000
        }
    );

    expect(mintTxInvoice.success).toBe(true);
    return mintTx;
}

async function transferAssetUsingRegular(mintTx, aliceAddress, bobAddress) {
    const asset = mintTx.getMintedAsset();
    const transferTx = sdk.core
        .createAssetTransferTransaction()
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                amount: 3000,
                assetType: asset.assetType
            },
            {
                recipient: aliceAddress,
                amount: 7000,
                assetType: asset.assetType
            }
        );
    sdk.key.signTransactionInput(transferTx, 0);

    const p = sdk.core.createAssetTransactionGroupParcel({
        transactions: [transferTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: regularSecret,
            nonce,
            fee: 10
        })
    );

    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        transferTx.hash(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxInvoice.success).toBe(true);
}
async function transferAssetUsingOther(mintTx, aliceAddress, bobAddress) {
    const asset = mintTx.getMintedAsset();

    const transferTx = sdk.core
        .createAssetTransferTransaction({
            burns: [],
            inputs: [],
            outputs: [],
            nonce: 1 // use nonce here because "transferAssetUsingRegular" will send the same transaction
        })
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                amount: 3000,
                assetType: asset.assetType
            },
            {
                recipient: aliceAddress,
                amount: 7000,
                assetType: asset.assetType
            }
        );
    sdk.key.signTransactionInput(transferTx, 0);

    const p = sdk.core.createAssetTransactionGroupParcel({
        transactions: [transferTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(otherAddress);
    await sdk.rpc.chain.sendSignedParcel(
        p.sign({
            secret: otherSecret,
            nonce,
            fee: 10
        })
    );

    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(
        transferTx.hash(),
        {
            timeout: 5 * 60 * 1000
        }
    );
    expect(transferTxInvoice.success).toBe(false);
}
