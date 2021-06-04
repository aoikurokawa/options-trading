const { BaseAsset, transactions } = require("lisk-sdk");
const { CREATE_RECOVERY_ASSET_ID, BASE_RECOVERY_DEPOSIT, FRIEND_FACTOR_FEE } = require("../constant.js");
const { createRecoverySchema } = require('../schemas');

// extend base asset to implement your custom asset
class CreateRecoveryAsset extends BaseAsset {
    // define unique asset name and id
    name = "createRecovery";
    id = CREATE_RECOVERY_ASSET_ID;
    schema = createRecoverySchema;

    async apply({
        asset,
        transaction,
        stateStore,
    }) {
        const sender = await stateStore.account.get(transaction.senderAddress);
        if (sender.srs.config && sender.srs.config.friends.length != 0) {
            throw Error("Account already has recovery configuration");
        }
        const sameAccount = asset.friends.find(f => f === sender.address);
        if (sameAccount) {
            throw Error("You cannot add yourself to your own frined list.");
        }
        // add frineds to the list
        sender.srs.config.friends = [...asset.friends.sort()];
        // minimum number of friends required to vouch
        sender.srs.config.recoveryThreshold = asset.recoveryThreshold;
        // minimum number of blocks after recovery process when account will be recoverable
        sender.srs.config.delayPeriod = asset.delayPeriod;
        // set the deposit based on number of friends, 10 + friends.length * 2
        const deposit = BigInt(BASE_RECOVERY_DEPOSIT) + BigInt(transactions.convertLSKToBeddows((sender.srs.config.friends.length * FRIEND_FACTOR_FEE).toString()));
        sender.srs.config.deposit = deposit;
        // save the value in stateStore
        await stateStore.account.set(sender.address, sender);
    }
}

module.exports = CreateRecoveryAsset;

