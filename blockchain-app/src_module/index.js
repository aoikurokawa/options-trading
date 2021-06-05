const { BaseModule, codec } = require("lisk-sdk");
const CreateRecoveryAsset = require("./transactions/create_recovery");
const ClaimRecoveryAsset = require("./transactions/claim_recovery");
const InitiateRecoveryAsset = require("./transactions/initiate_recovery");
const VouchRecoveryAsset = require("./transactions/vouch_recovery");
const CloseRecoveryAsset = require("./transactions/close_recovery");
const RemoveRecoveryAsset = require("./transactions/remove_recovery");
const { 
    SRSAccountSchema,
    createRecoverySchema,
    initiateRecoverySchema,
    claimRecoverySchema, 
} = require("./schemas");
const {
    CREATE_RECOVERY_ASSET_ID, 
    CLAIM_RECOVERY_ASSET_ID, 
    INITIATE_RECOVERY_ASSET_ID, 
    REMOVE_RECOVERY_ASSET_ID,
} = require("../constant");


// extend from the base module to implement a custom module
class SRSModule extends BaseModule {
    name = 'srs';
    id = 1026;
    accountSchema = SRSAccountSchema;

    transactionAssets = [
        new CreateRecoveryAsset(),
        new ClaimRecoveryAsset(),
        new InitiateRecoveryAsset(),
        new VouchRecoveryAsset(),
        new CloseRecoveryAsset(),
        new RemoveRecoveryAsset(),
    ];

    events = ['configCreated', 'configRemoved', 'recoeryInitiated'];

    async afterTransactionApply({transaction, stateStore, reducerHandler}) {
        if (transaction.moduleID === this.id && transaction.assetID === CREATE_RECOVERY_ASSET_ID) {
            let createRecoveryAsset = codec.decode(
                createRecoverySchema, 
                transaction.asset
            );
            const friends = createRecoveryAsset.friends.map(bufferFriend => bufferFriend.toString('hex'));
            this._channel.publish('srs:configCreated', {
                address: transaction._senderAddress.toString('hex'), 
                friends: friends,
                recoveryThreshold: createRecoveryAsset.recoveryThreshold, 
                delayPeriod: createRecoveryAsset.delayPeriod,
            });

        } else if (transaction.moduleID === this.id && transaction.assetID === REMOVE_RECOVERY_ASSET_ID) {
            this._channel.publish('srs:configRemoved', {
                address: transaction._senderAddress.toString('hex'),
            });

        } else if (transaction.moduleID === this.id && transaction.assetID === CLAIM_RECOVERY_ASSET_ID) {
            let claimRecoveryAsset = codec.decode(
                claimRecoverySchema, 
                transaction.asset
            );
            this._channel.publish('srs:configRemoved', {
                address: claimRecoveryAsset.lostAccount.toString('hex')
            });
        } else if (transaction.moduleID === this.id && transaction.assetID === INITIATE_RECOVERY_ASSET_ID) {
            const initiateRecoveryAsset = codec.decode(
                initiateRecoverySchema, 
                transaction.asset
            );
            this._channel.publish('srs:recoveryInitiated', {
                address: transaction._senderAddress.toString('hex'), 
                asset: initiateRecoveryAsset
            });
        }
    };
}

module.exports = SRSModule;

