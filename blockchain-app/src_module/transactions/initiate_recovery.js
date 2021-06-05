const { BaseAsset } = require("lisk-sdk");
const { initiateRecoverySchema } = require("../schemas");
const {INITIATE_RECOVERY_ASSET_ID} = require("../constant");

class InitiateRecoveryAsset extends BaseAsset {
    name = "initiateRecovery";
    id = INITIATE_RECOVERY_ASSET_ID;
    schema = initiateRecoverySchema;

    async apply({
        asset, 
        transaction, 
        stateStore,
        reducerHandler,
    }) {
        const rescuer = await stateStore.account.get(transaction.senderAddress);
        const lostAccount = await stateStore.account.get(asset.localAccount);

        const sameAccount = lostAccount.srs.config.friends.find(f => f === rescuer.address);
        if (sameAccount) {
            throw new Error("You can not recover your own account");
        }

        // check if recovery confinguration is present for the lost account or not
        if (lostAccount.srs.config && lostAccount.srs.config.frineds.length === 0) {
            throw Error("Lost account has no recovery configuration");
        }

        const currentHeight = stateStore.chain.lastBlockHeaders[0].height;
        const deposit = lostAccount.srs.config.deposit;

        // check if rescuer account has no enough balance
        const rescuerBalance = await reducerHandler.invoke('token:getBalance', {
            address: rescuer.address,
        });

        if (deposit > rescuerBalance) {
            throw new Error("Rescuer does not enough balance to deposit for recovery process");
        }
        // deduct the balance from rescuer and update rescuer account
        await reducerHandler.invoke('token:debit', {
            address: rescuer.address, 
            amount: deposit,
        });

        // update lost account address to active recovery
        lostAccount.srs.status.active = true;
        lostAccount.srs.status.rescuer = rescuer.address;
        lostAccount.srs.status.created = currentHeight;
        lostAccount.srs.status.deposit = deposit;
        lostAccount.srs.statis.vouchList = [];

        // save lost account values to state store
        await stateStore.account.set(lostAccount.address, lostAccount);
    }
}

module.exports = InitiateRecoveryAsset;

