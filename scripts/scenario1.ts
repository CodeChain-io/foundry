import * as assert from "assert";
import { AssetTransferTransaction, AssetMintTransaction, PaymentTransaction } from "../src/primitives/transaction/";
import { Parcel, H256, U256, H160 } from "../src/primitives";
import { SDK } from "../src";
import { privateKeyToAddress } from "../src/utils";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));
const networkId = 17;

const sdk = new SDK("http://localhost:8080");

sdk.getNonce(address).then(nonce => {
    if (!nonce) {
        throw new Error("No nonce");
    }
    console.log(`Current nonce: ${nonce.value}`);
}).then( () => {
    const t = new PaymentTransaction({
        nonce: new U256(1),
        address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
        value: new U256("0")
    });
    const p = new Parcel(new U256(0), new U256(10), t, networkId);
    return sendParcel(p, new H256("8cd5afa74438c814aebcdb430459947342f6e926e9fbe3453b61190ea1498a3c"));
}).then( () => {
    const t = new PaymentTransaction({
        nonce: new U256(3),
        address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
        value: new U256("0")
    });
    const p = new Parcel(new U256(2), new U256(10), t, networkId);
    return sendParcel(p, new H256("778812c44c66c5ef1764793ff9961e51f5a248f054369b688bcef7083809a93e"));
}).then( () => {
    const t = new PaymentTransaction({
        nonce: new U256(5),
        address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
        value: new U256("0")
    });
    const p = new Parcel(new U256(4), new U256(10), t, networkId);
    return sendParcel(p, new H256("1955d10b06de938d6ffee026d6220bbd7496b158a56b6d280dc2363a18e8068e"));
}).then( () => {
    const t = new PaymentTransaction({
        nonce: new U256(7),
        address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
        value: new U256("0")
    });
    const p = new Parcel(new U256(6), new U256(10), t, networkId);
    return sendParcel(p, new H256("63c6969fa92659da9afd54d976293365ed8093bc7a87d8c5282d59e55e2478da"));
}).then(printAssets).then( () => {
}).then( () => {
    const t = new AssetMintTransaction({
        metadata: "metadata of permissioned infinite asset",
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        amount: null,
        registrar: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911")
    });

    const p = new Parcel(new U256(8), new U256(10), t, networkId);
    return sendParcel(p, new H256("a507f0116053be0bfb57a067cd55ac5cfabbc4dd838b9bed1fa552fba531e104"));
}).then(printAssets).then( () => {
}).then( () => {
    const t = new AssetMintTransaction({
        metadata: "metadata of non-permissioned asset",
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        amount: 100,
        registrar: null
    });

    const p = new Parcel(new U256(9), new U256(10), t, networkId);
    return sendParcel(p, new H256("b25401a45611df40521a8a04a54be3b017acdce68ca134115a9b58d7571246e1"));
}).then(printAssets).then( () => {
}).then( () => {
    const inputs = [ {
        prevOut: {
            transactionHash: new H256("84c8d5d2328dc4ea6da1cdaddaf8cfa5ce6ba0373f724d91bdc6a69c6977183d"),
            index: 0,
            assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
            amount: 100
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }];
    const outputs = [{
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
        amount: 93
    }, {
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
        amount: 4
    }, {
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
        amount: 2
    }, {
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
        amount: 1
    }];
    const t = new AssetTransferTransaction(networkId, { inputs, outputs });

    const p = new Parcel(new U256(10), new U256(10), t, networkId);
    return sendParcel(p, new H256("be285304c6aa7f5df42835e91f8c08cfd50c0d9cfa59700e29946fafcbb3ab8c"));
}).then(printAssets).then( () => {
}).then( () => {
    const inputs = [{
        prevOut: {
            transactionHash: new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"),
            index: 1,
            assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
            amount: 4
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }, {
        prevOut: {
            transactionHash: new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"),
            index: 3,
            assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
            amount: 1
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }];
    const outputs = [{
        lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
        parameters: [],
        assetType: new H256("5300000000000000e97740b63d40849b152624cd468aef4c95838689b0fdb551"),
        amount: 5
    }];
    const t = new AssetTransferTransaction(networkId, {inputs, outputs});

    const p = new Parcel(new U256(11), new U256(10), t, networkId);
    return sendParcel(p, new H256("5d98a4e3d65999838859a3c28e643b2125e9a87fc6020a86c10461125aef7367"));
}).then(printAssets).then( () => {
    console.log("Succeed");
}).catch( err => {
    console.error(err);
});

function printAsset(hash: H256, index: number) {
    return sdk.getAsset(hash, index).then(asset => {
        if (asset) {
            console.log(asset);
        } else {
            console.log("No asset");
        }
    });
}
function printAssetScheme(hash: H256) {
    return sdk.getAssetScheme(hash).then(scheme => {
        if (scheme) {
            console.log(scheme);
        } else {
            console.log("No scheme");
        }
    });
}

function printAssets(): Promise<any> {
    console.log("=====================");
    return printAssetScheme(new H256("ca79dffe73be0b0ef8afc3eeef2c300b087e332486172c79096d0c42f47abc9c"))
        .then(() => printAssetScheme(new H256("84c8d5d2328dc4ea6da1cdaddaf8cfa5ce6ba0373f724d91bdc6a69c6977183d")))
        .then(() => printAsset(new H256("ca79dffe73be0b0ef8afc3eeef2c300b087e332486172c79096d0c42f47abc9c"), 0))
        .then(() => printAsset(new H256("84c8d5d2328dc4ea6da1cdaddaf8cfa5ce6ba0373f724d91bdc6a69c6977183d"), 0))
        .then(() => printAsset(new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"), 0))
        .then(() => printAsset(new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"), 1))
        .then(() => printAsset(new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"), 2))
        .then(() => printAsset(new H256("4b770ac940e476148754f903a6cb2448be89cbfa0d89bd398a9edb03e913ae01"), 3))
        .then(() => printAsset(new H256("236ecc7778acf5dad60fd2b0dd0fb9fefe5a2bf466e244d75de180f9687c1b82"), 0))
        .catch(err => {
            console.error(err);
        }).then(() => {
            console.log("=====================");
        });
}

function sendParcel(parcel: Parcel, parcel_hash: H256): Promise<any> {
    return sdk.sendSignedParcel(parcel.sign(secret)).then(hash => {
        assert(hash.isEqualTo(parcel_hash));
        return new Promise<H256>(resolver => {
            setTimeout(() => resolver(hash), 2000);
        });
    }).catch(err => {
        console.error(err);
        return parcel_hash;
    }).then(hash => {
        return sdk.getParcelInvoice(hash);
    }).then (invoice => {
        if (!invoice) {
            throw new Error("No invoice");
        }
        assert(invoice.toEncodeObject());
    });
}