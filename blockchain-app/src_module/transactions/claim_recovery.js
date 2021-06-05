const { BaseAsset } = require("lisk-sdk");
const { claimRecoverySchema } = require("../schemas");
const { CLAIM_RECOVERY_ASSET_ID } = require("../constant");

class ClaimRecoveryAsset extends BaseAsset {
    name = "claimRecovery";
    id = CLAIM_RECOVERY_ASSET_ID;
    schema = claimRecoverySchema;

    async apply({
        asset, 
        transaction, 
        stateStore,
        reducerHandler,
    }) {
        const rescuer = await stateStore.account.get(transaction.senderAddress);
        const lostAccount = await stateStore.account.get(asset.lostAccount);

        const currentHeight = stateStore.chain.lastBlockHeaders[0].height;
        const delayPeriod = lostAccount.srs.config.delayPeriod;
        const recoveryThreshold = lostAccount.srs.config.recoveryThreshold;
        const deposit = lostAccount.srs.config.deposit;

        // check if the delay period is passed to claim the recovery
        if (lostAccount.srs.status.vouchList.length < recoveryThreshold) {
            throw new Error(`Can not account unitil minimum threshold of ${lostAccount.srs.config.friends.length} friends have vouched`);
        }

        const minBalance = await reducerHandler.invoke('token:getMinimumRemainingBalance');
        // get the account balance of list account
        const lostAccountBalance = await reducerHandler.invoke('token:getBalance', {
            address: lostAccount.address,
        });

        await reducerHandler.invoke('token:debit', {
            address: lostAccount.address, 
            // get the deposit back from the lost account as well as your own deposit that was locked
            amount: lostAccountBalance - minBalance,
        });

        await reducerHandler.invoke('token:credit', {
            address: rescuer.address, 
            // get the deposit back from the lost account as well as your own deposit that was locked
            amount: BigInt(2) * deposit + lostAccountBalance - minBalance, 
        });

        await stateStore.account.set(rescuer.address, rescuer);
        // reset all recovery values in the lost account
        lostAccount.srs.config.friends = [];
        lostAccount.srs.config.delayPeriod = 0;
        lostAccount.srs.config.recoveryThreshold = 0;
        lostAccount.srs.config.deposit = BigInt('0');
        lostAccount.srs.status.active = false;
        lostAccount.srs.status.rescuer = Buffer.from('');
        lostAccount.srs.status.created = 0;
        lostAccount.srs.status.deposit = BigInt('0');
        lostAccount.srs.status.vouchList = [];
        await stateStore.account.set(lostAccount.address, lostAccount);
    }
}

module.exports = ClaimRecoveryAsset;

