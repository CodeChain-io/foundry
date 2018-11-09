"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const H512_1 = require("../H512");
const Transaction_1 = require("../transaction/Transaction");
const U256_1 = require("../U256");
const AssetTransaction_1 = require("./AssetTransaction");
const CreateShard_1 = require("./CreateShard");
const Payment_1 = require("./Payment");
const SetReulgarKey_1 = require("./SetReulgarKey");
const SetShardOwners_1 = require("./SetShardOwners");
const SetShardUsers_1 = require("./SetShardUsers");
function getActionFromJSON(json) {
    const { action } = json;
    switch (action) {
        case "assetTransaction":
            const { transaction } = json;
            return new AssetTransaction_1.AssetTransaction({
                transaction: Transaction_1.getTransactionFromJSON(transaction)
            });
        case "payment":
            const { receiver, amount } = json;
            return new Payment_1.Payment(codechain_primitives_1.PlatformAddress.ensure(receiver), new U256_1.U256(amount));
        case "setRegularKey":
            const { key } = json;
            return new SetReulgarKey_1.SetRegularKey(new H512_1.H512(key));
        case "createShard":
            return new CreateShard_1.CreateShard();
        case "setShardOwners": {
            const { shardId, owners } = json;
            return new SetShardOwners_1.SetShardOwners({
                shardId,
                owners: owners.map(codechain_primitives_1.PlatformAddress.ensure)
            });
        }
        case "setShardUsers": {
            const { shardId, users } = json;
            return new SetShardUsers_1.SetShardUsers({
                shardId,
                users: users.map(codechain_primitives_1.PlatformAddress.ensure)
            });
        }
        default:
            throw Error(`Unexpected parcel action: ${action}`);
    }
}
exports.getActionFromJSON = getActionFromJSON;
