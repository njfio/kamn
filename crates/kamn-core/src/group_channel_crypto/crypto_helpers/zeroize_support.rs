use super::super::SenderKeyDistributionRecord;
use std::collections::BTreeMap;
use zeroize::Zeroize;

pub(crate) fn zeroize_sender_key_history(
    sender_key_history: &mut BTreeMap<String, BTreeMap<u64, SenderKeyDistributionRecord>>,
) {
    for (mut sender_did, generations) in std::mem::take(sender_key_history) {
        sender_did.zeroize();
        for (_, mut record) in generations {
            zeroize_sender_key_distribution_record(&mut record);
        }
    }
}

pub(crate) fn zeroize_sender_key_distribution_record(record: &mut SenderKeyDistributionRecord) {
    record.channel_id.zeroize();
    record.sender_did.zeroize();
    record.sender_key_ref.zeroize();
    let allowlist = std::mem::take(&mut record.recipient_allowlist);
    for mut recipient in allowlist {
        recipient.zeroize();
    }
}

pub(crate) fn zeroize_u64_keyed_sender_history(sender_history: &mut BTreeMap<String, u64>) {
    for (mut sender_did, _) in std::mem::take(sender_history) {
        sender_did.zeroize();
    }
}
