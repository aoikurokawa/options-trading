const { BaseAsset } = require("lisk-sdk");
const { vouchRecoverySchema } = require("../schemas");
const { VOUCH_RECOVERY_ASSET_ID } = require("../constant");

class VouchRecoveryAsset extends BaseAsset {
    name = "vouchRecovery";
    id = VOUCH_RECOVERY_ASSET_ID;
    schema = vouchRecoverySchema;

    async apply({
        asset, 
        transaction, 
        stateStore,
    }) {
        const sender = await stateStore.account.get(transaction.senderAddress);
        const lostAccount = await stateStore.account.get(asset.lostAccount);
        const rescuer = await stateStore.account.get(asset.rescuer);

        // make sure rescuer and lost account match according to config settings
        if (!lostAccount.srs.status.rescuer.equals(rescuer.address)) {
            throw new Error(`Rescuer address is incorrect for the recovery of ${lostAccount.address.toString('hex')}`);
        }

        const found = lostAccount.srs.config.friends.find(f => f.equals(sender.address));
        // make sure friend is present in the configuration
        if (!found) {
            throw new Error('The sender is not part of friends who can vouch for rescuer for recovery process');
        }

        const foundSignature = lostAccount.srs.status.vouchList.find(f => f.equals(sender.address));
        // make sure the friend has not already voted
        if (foundSignature) {
            throw new Error('The sender has already vouched for the rescuer for recovery process');
        }

        // push signature to vouch list
        lostAccount.srs.status.vouchList.push(sender.address);
        await stateStore.account.set(lostAccount.address, lostAccount);
    }
}

module.exports = VouchRecoveryAsset;
