use sea_orm_migration::prelude::*;

/// V-MED-11: payout_accounts.metadata field-level encryption at rest.
///
/// This migration is a **no-op schema change**. The `metadata` column stays
/// JSONB; from V-MED-11 onward the application stores an envelope
/// `{"enc": "<base64(AES-256-GCM nonce||ct||tag)>"}` instead of plaintext JSON
/// (see `src/utils/field_crypto.rs` and
/// `src/db/sea_models/payout_account/model.rs`). No type change is needed, so
/// there is nothing to `ALTER` here.
///
/// ── RUNBOOK: existing plaintext rows ──────────────────────────────────────
///
/// Pre-encryption rows hold their real metadata as a plain JSON object
/// (no `enc` key). After V-MED-11:
///
///   * Reads: `decrypt_metadata` detects the legacy shape and returns it
///     **as-is** (with a warning) rather than failing — so a half-migrated DB
///     does not break reads. But the row is still plaintext at rest.
///   * Writes: any `UPDATE` that touches `metadata` re-encrypts it, so the row
///     is protected the next time it is saved.
///
/// To backfill (encrypt every existing row in place) without going through the
/// ORM, run the bundled helper once, with `FIELD_ENC_KEY` exported in the
/// application process environment:
///
/// ```sh
/// # From backend/api:
/// FIELD_ENC_KEY=<32-byte key> cargo run --features full --bin ruxlog-backfill-payout-metadata
/// ```
///
/// (If no such helper binary is wired yet, the equivalent is: load each row
/// via `Entity::find()`, read `metadata`, and `ActiveModel::set(metadata).update()`
/// — the model's `before_save` re-encrypts. See `payout_account/model.rs`.)
///
/// If backfill is skipped intentionally (e.g. no production rows exist yet),
/// this migration leaves the table untouched and the system boots and reads
/// correctly in both encrypted and legacy-plaintext states.
///
/// If existing plaintext rows are acceptable to lose (dev/test data), a hard
/// clobber is also valid:
/// ```sql
/// UPDATE payout_accounts SET metadata = NULL WHERE metadata IS NOT NULL;
/// ```
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Intentionally no DDL. The column type is unchanged; only the
        // application-level interpretation of its contents changes (envelope vs
        // plaintext). See the runbook above for backfill.
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No schema was altered, so there is nothing to revert. Encrypted rows
        // are NOT auto-decrypted on `down`; run the backfill helper in reverse
        // if you need plaintext.
        Ok(())
    }
}
