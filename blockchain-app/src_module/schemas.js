const createRecoverySchema = {
    $id: 'srs/recovery/create',
    type: 'object',
    required: ['friends', 'recoveryThreshold', 'delayPeriod'],
    properties: {
        friends: {
            type: 'array',
            fieldNumber: 1,
            items: {
                dataType: 'bytes',
            },
        },
        recoveryThreshold: {
            dataType: 'uint32',
            fieldNumber: 2,
        },
        delayPeriod: {
            dataType: 'uint32',
            fieldNumber: 3,
        },
    },
};

module.exports = { createRecoverySchema };
