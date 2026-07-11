use super::*;
use solana_sdk::{
    hash::Hash, message::compiled_instruction::CompiledInstruction, pubkey::Pubkey,
    signature::Keypair, transaction::Transaction,
};
use solana_system_transaction as system_transaction;

const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

pub(super) fn build_live_settlement_transaction(
    client: &RpcClient,
    config: &LiveSolanaSettlementConfig,
    keypair: &Keypair,
    escrow_id: &str,
) -> Result<Transaction, String> {
    let blockhash = client.get_latest_blockhash().map_err(|error| {
        format!("live solana settlement latest blockhash lookup failed: {error}")
    })?;
    build_escrow_bound_transaction(
        keypair,
        &config.recipient_pubkey,
        config.lamports,
        blockhash,
        escrow_id,
    )
}

pub(super) fn build_escrow_bound_transaction(
    payer: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
    blockhash: Hash,
    escrow_id: &str,
) -> Result<Transaction, String> {
    let mut transaction = system_transaction::transfer(payer, recipient, lamports, blockhash);
    append_escrow_memo(&mut transaction, escrow_id)?;
    transaction.sign(&[payer], blockhash);
    Ok(transaction)
}

fn append_escrow_memo(transaction: &mut Transaction, escrow_id: &str) -> Result<(), String> {
    let memo_program = Pubkey::from_str(MEMO_PROGRAM_ID)
        .map_err(|error| format!("settlement memo program invalid: {error}"))?;
    let index = u8::try_from(transaction.message.account_keys.len())
        .map_err(|_| "settlement memo account index overflow".to_owned())?;
    transaction.message.account_keys.push(memo_program);
    transaction.message.header.num_readonly_unsigned_accounts += 1;
    transaction.message.instructions.push(CompiledInstruction {
        program_id_index: index,
        accounts: Vec::new(),
        data: format!("kamn-escrow:{escrow_id}").into_bytes(),
    });
    Ok(())
}

pub(super) fn validate_persisted_transaction(
    json: &str,
    expected_signature: &str,
) -> Result<(), String> {
    let transaction: Transaction = serde_json::from_str(json)
        .map_err(|error| format!("settlement transaction integrity decode failed: {error}"))?;
    let signature = transaction.signatures.first().map(ToString::to_string);
    if signature.as_deref() != Some(expected_signature) || transaction.verify().is_err() {
        return Err("settlement transaction integrity verification failed".to_owned());
    }
    Ok(())
}

pub(super) fn validate_prepared_transaction(
    transaction: &Transaction,
    prepared: &PreparedLiveSettlement,
    escrow_id: &str,
) -> Result<(), String> {
    validate_persisted_transaction(
        prepared.signed_transaction_json.as_str(),
        prepared.expected_signature.as_str(),
    )?;
    validate_transfer_instruction(transaction, prepared)?;
    validate_escrow_memo(transaction, escrow_id)
}

fn validate_transfer_instruction(
    transaction: &Transaction,
    prepared: &PreparedLiveSettlement,
) -> Result<(), String> {
    let instruction = transaction
        .message
        .instructions
        .first()
        .ok_or_else(|| "settlement transfer instruction missing".to_owned())?;
    let recipient_index = *instruction
        .accounts
        .get(1)
        .ok_or_else(|| "settlement transfer recipient missing".to_owned())?
        as usize;
    let recipient = transaction.message.account_keys.get(recipient_index);
    let amount = instruction
        .data
        .get(4..12)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u64::from_le_bytes);
    if recipient.map(ToString::to_string).as_deref() == Some(prepared.recipient_pubkey.as_str())
        && amount == Some(prepared.amount_lamports)
    {
        return Ok(());
    }
    Err("settlement transaction agreement verification failed".to_owned())
}

fn validate_escrow_memo(transaction: &Transaction, escrow_id: &str) -> Result<(), String> {
    let expected = format!("kamn-escrow:{escrow_id}");
    let memo_program = Pubkey::from_str(MEMO_PROGRAM_ID)
        .map_err(|error| format!("settlement memo program invalid: {error}"))?;
    if transaction.message.instructions.iter().any(|instruction| {
        transaction
            .message
            .account_keys
            .get(instruction.program_id_index as usize)
            == Some(&memo_program)
            && instruction.data == expected.as_bytes()
    }) {
        return Ok(());
    }
    Err("settlement transaction escrow binding verification failed".to_owned())
}
