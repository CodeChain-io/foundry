import { SDK, AssetMintTransaction, H256, Parcel, H160, U256, PaymentTransaction, H512, SetRegularKeyTransaction, AssetTransferTransaction, AssetTransferInput, AssetOutPoint, AssetTransferOutput, privateKeyToAddress } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));

export const mintAsset = async ({ metadata, amount, lockScriptHash, parameters, registrar }) => {
    const assetMintTransaction = new AssetMintTransaction({
        nonce: 1,
        metadata,
        lockScriptHash,
        parameters,
        amount,
        registrar,
    });
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;

    const parcel = Parcel.transactions(nonce, fee, networkId, assetMintTransaction);
    const parcelHash = await sdk.sendSignedParcel(parcel.sign(secret));

    return {
        parcelHash,
        assetMintTransaction
    };
};

export const transferAsset = async ({ mintTx }) => {
    const networkId = 17;
    const assetTransferTransaction = new AssetTransferTransaction(networkId, {
        inputs: [new AssetTransferInput({
            prevOut: new AssetOutPoint({
                transactionHash: mintTx.hash(),
                index: 0,
                assetType: mintTx.getAssetSchemeAddress(),
                amount: 100
            }),
            lockScript: Buffer.from([0x2, 0x1]),
            unlockScript: Buffer.from([])
        })],
        outputs: [new AssetTransferOutput({
            lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
            parameters: [],
            assetType: mintTx.getAssetSchemeAddress(),
            amount: 100
        })]
    });
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);

    const parcel = Parcel.transactions(nonce, fee, networkId, mintTx, assetTransferTransaction);
    const parcelHash = await sdk.sendSignedParcel(parcel.sign(secret));

    return {
        parcelHash,
        assetTransferTransaction
    };
};

export const payment = async () => {
    const nonce = await sdk.getNonce(address);
    const fee = new U256(10);
    const networkId = 17;
    const p = Parcel.payment(nonce, fee, networkId, address, new U256(0)).sign(secret);
    return await sdk.sendSignedParcel(p);
};

export const paymentTwice = async () => {
    const fee = new U256(10);
    const networkId = 17;
    const value = new U256(0);

    const nonce1 = await sdk.getNonce(address);
    const nonce2 = nonce1.increase();

    const p1 = Parcel.payment(nonce1, fee, networkId, address, value).sign(secret);
    await sdk.sendSignedParcel(p1);

    const p2 = Parcel.payment(nonce2, fee, networkId, address, value).sign(secret);
    await sdk.sendSignedParcel(p2);
};

export const setRegularKey = async () => {
    const nonce = await sdk.getNonce(address);
    const key = new H512("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
    const fee = new U256(10);
    const networkId = 17;
    const p = Parcel.setRegularKey(nonce, fee, networkId, key);
    return await sdk.sendSignedParcel(p.sign(secret));
};
