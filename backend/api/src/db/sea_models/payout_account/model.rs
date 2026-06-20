use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use ruxlog_types::enums::PayoutAccountStatus;

use crate::utils::field_crypto;

/// Marker JSON object key for the encrypted envelope.
///
/// V-MED-11 (CWE-312): `metadata` is a JSONB blob holding provider payout
/// configuration (bank routing, wallet addresses, account holder names). Stored
/// plaintext it leaks in full on a DB dump. The model layer now stores an
/// envelope of the form `{"enc": "<base64(AES-256-GCM(nonce||ct||tag))>"}` and
/// keeps the real JSON out of the database. The `enc` key both flags the column
/// as encrypted and lets the decrypt path tell an envelope from a legacy
/// plaintext row (see `decrypt_metadata`).
const ENC_KEY: &str = "enc";

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payout_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub provider: String,
    pub provider_account_id: String,
    pub status: PayoutAccountStatus,
    /// At-rest this holds the encrypted envelope `{"enc": "<b64 ciphertext>"}`,
    /// NOT the real payout metadata. The decrypted plaintext JSON is surfaced
    /// only through [`Model::decrypted_metadata`] / [`decrypt_metadata`]; the
    /// `metadata` field on a freshly-loaded `Model` is whatever came off the
    /// wire (the envelope), so callers MUST go through the decrypt helpers to
    /// read it. Writes are encrypted automatically by `ActiveModelBehavior`.
    pub metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
        to = "super::super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

// ──────────────────────────────────────────────────────────────────────────
// V-MED-11: model-layer encryption (no caller can forget)
// ──────────────────────────────────────────────────────────────────────────
//
// The encrypt happens in `ActiveModelBehavior::before_save` — every insert and
// update that touches `metadata` is wrapped, so no service/handler layer can
// persist plaintext by accident. The decrypt is a read-side helper because
// SeaORM's `DeriveEntityModel` generates `Model::from_query_result` and the row
// reader has no per-field hook; callers therefore call
// `Model::decrypted_metadata` (or the free `decrypt_metadata`) on a loaded row.
// The envelope shape (`{"enc": ...}`) is how the decrypt path distinguishes a
// real encrypted row from a legacy plaintext row written before this fix.

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Encrypt `metadata` to the envelope shape before it hits the DB. Runs for
    /// both `Insert` and `Update`. A missing key is a deployment bug — we
    /// surface it as a `DbErr` rather than silently persisting plaintext
    /// (which would re-introduce the CWE-312 exposure).
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // Only encrypt when a value is actually being written. `Unchanged`
        // (loaded-but-not-modified) is skipped so an unrelated UPDATE of another
        // column does not re-encrypt — and so a row that is already an encrypted
        // envelope on disk is not treated as plaintext and double-wrapped.
        // `Set(Some(json))` is the plaintext write path we must wrap.
        match self.metadata.clone() {
            sea_orm::ActiveValue::Set(Some(plaintext_json)) => {
                let envelope = encrypt_metadata(&plaintext_json).map_err(|err| {
                    DbErr::Custom(format!(
                        "payout_account.metadata encryption failed: {err}"
                    ))
                })?;
                self.metadata = sea_orm::ActiveValue::Set(Some(envelope));
            }
            // `Set(None)` clears metadata — nothing to encrypt, persist as NULL.
            // `Unchanged` / `NotSet` — leave the active value alone.
            _ => {}
        }
        Ok(self)
    }
}

/// Encrypt a plaintext metadata JSON value into the `{"enc": "<b64>"}` envelope.
/// `None` stays `None` (a missing payout config is not a secret to protect).
/// Failures (key unset, AES error) propagate so the caller never stores
/// plaintext on a crypto failure.
///
/// Idempotent: if the value already matches the envelope shape it is returned
/// unchanged, so re-saving a row whose `metadata` was loaded as the stored
/// envelope does not double-encrypt (which would make it undecryptable).
pub fn encrypt_metadata(plaintext: &Json) -> Result<Json, field_crypto::FieldCryptoError> {
    // Already an encrypted envelope → leave it alone (idempotent re-save).
    if is_encrypted_envelope(plaintext) {
        return Ok(plaintext.clone());
    }
    // Compact JSON string of the real metadata — what gets encrypted.
    let plaintext_str = serde_json::to_string(plaintext).map_err(|_| {
        field_crypto::FieldCryptoError::Encrypt
    })?;
    let ciphertext = field_crypto::encrypt(&plaintext_str)?;
    Ok(serde_json::json!({ ENC_KEY: ciphertext }))
}

/// True iff `value` is the `{"enc": "<string>"}` single-key envelope produced by
/// `encrypt_metadata`. Used to keep `encrypt_metadata` idempotent and to let the
/// decrypt path distinguish real encrypted rows from legacy plaintext.
pub fn is_encrypted_envelope(value: &Json) -> bool {
    value
        .as_object()
        .map(|obj| obj.len() == 1 && obj.get(ENC_KEY).map(|v| v.is_string()).unwrap_or(false))
        .unwrap_or(false)
}

/// Decrypt a metadata JSON value loaded from the DB back to the plaintext JSON.
///
/// Handles three column states:
///   * envelope `{"enc": "<b64>"}` → decrypt and parse (the post-fix shape);
///   * a JSON object WITHOUT the `enc` key → treated as a legacy plaintext row
///     (written before V-MED-11) and returned as-is so a half-migrated DB does
///     not hard-fail reads (see runbook note in the migration); log a warning;
///   * `None` → `Ok(None)`.
///
/// A malformed envelope (e.g. `enc` present but not decryptable) fails closed
/// — it returns `Err` rather than returning the opaque blob, so a tampered row
/// is never silently surfaced as if it were valid metadata.
pub fn decrypt_metadata(
    stored: &Option<Json>,
) -> Result<Option<Json>, field_crypto::FieldCryptoError> {
    let Some(value) = stored else {
        return Ok(None);
    };

    // Real encrypted envelope → decrypt.
    if is_encrypted_envelope(value) {
        let ciphertext = value
            .get(ENC_KEY)
            .and_then(|v| v.as_str())
            .expect("is_encrypted_envelope guarantees a string `enc` value");
        let plaintext_str = field_crypto::decrypt(ciphertext)?;
        let plaintext_json: Json =
            serde_json::from_str(&plaintext_str).map_err(|_| field_crypto::FieldCryptoError::Decode)?;
        return Ok(Some(plaintext_json));
    }

    // Legacy plaintext row — see the runbook note in m20260620_000051. Surface
    // it unchanged (do NOT lose data the operator still needs to backfill), but
    // flag it so the migration gap is visible.
    tracing::warn!(
        "payout_account.metadata row is plaintext (pre-V-MED-11) or malformed; \
         returning as-is without decryption. Backfill before treating as safe."
    );
    Ok(Some(value.clone()))
}

impl Model {
    /// Decrypt this loaded row's `metadata` into the plaintext JSON. Convenience
    /// over [`decrypt_metadata`] for the common read path:
    /// `let meta = account.decrypted_metadata()?;`
    pub fn decrypted_metadata(&self) -> Result<Option<Json>, field_crypto::FieldCryptoError> {
        decrypt_metadata(&self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Install a deterministic test key into the global slot for this test
    /// thread. The slot is process-wide (`OnceLock`), so the first test to run
    /// wins; all assertions here use the explicit-keyed API which is
    /// independent of the global, so order does not matter.
    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(0x11);
        }
        k
    }

    #[test]
    fn metadata_round_trips_through_envelope() {
        let key = test_key();
        field_crypto::set_key(&key).ok(); // best-effort global install
        let plaintext = serde_json::json!({
            "iban": "DE89370400440532013000",
            "holder": "Jane Doe",
            "bank_code": "37040044"
        });
        let envelope = encrypt_metadata(&plaintext).expect("encrypt");
        // The stored shape MUST be the `{"enc": ...}` envelope, never the
        // plaintext keys.
        assert!(
            envelope.get(ENC_KEY).and_then(|v| v.as_str()).is_some(),
            "envelope must wrap ciphertext under `{ENC_KEY}`"
        );
        assert!(
            !envelope.to_string().contains("iban"),
            "plaintext field name leaked into ciphertext envelope"
        );
        let recovered = decrypt_metadata(&Some(envelope)).expect("decrypt");
        assert_eq!(recovered, Some(plaintext));
    }

    #[test]
    fn none_metadata_round_trips_to_none() {
        let recovered = decrypt_metadata(&None).expect("none should decrypt to none");
        assert!(recovered.is_none());
    }

    #[test]
    fn tampered_envelope_fails_closed() {
        field_crypto::set_key(&test_key()).ok();
        let plaintext = serde_json::json!({"wallet": "0xabc"});
        let mut envelope = encrypt_metadata(&plaintext).expect("encrypt");
        // Corrupt the ciphertext string inside the envelope.
        let ct = envelope
            .get(ENC_KEY)
            .and_then(|v| v.as_str())
            .expect("envelope has enc")
            .to_string();
        // Corrupt one base64 character of the envelope's ciphertext: stays valid
        // base64 + UTF-8 (so decrypt reaches the AEAD step) but changes the
        // decoded ciphertext so the GCM auth tag no longer matches → fail closed.
        // (XORing a raw byte with 0xff would instead yield invalid UTF-8 and
        // panic on the String round-trip before decrypting.)
        let mut chars: Vec<char> = ct.chars().collect();
        let mid = chars.len() / 2;
        chars[mid] = if chars[mid] == 'A' { 'B' } else { 'A' };
        envelope[ENC_KEY] = serde_json::Value::String(chars.into_iter().collect());
        let err = decrypt_metadata(&Some(envelope)).expect_err("tampered must fail closed");
        assert!(matches!(
            err,
            field_crypto::FieldCryptoError::Decrypt | field_crypto::FieldCryptoError::Decode
        ));
    }

    #[test]
    fn legacy_plaintext_row_is_returned_unencrypted_with_warning() {
        // A pre-V-MED-11 row stored its metadata as a plain object (no `enc`
        // key). The decrypt path must not panic or drop it — it returns the
        // value as-is so operators can still read/backfill it.
        let legacy = serde_json::json!({"iban": "DE00", "holder": "Old"});
        let recovered =
            decrypt_metadata(&Some(legacy.clone())).expect("legacy row must not hard-fail");
        assert_eq!(recovered, Some(legacy));
    }

    #[test]
    fn different_plaintexts_produce_different_envelopes() {
        field_crypto::set_key(&test_key()).ok();
        let a = encrypt_metadata(&serde_json::json!({"x": 1})).expect("encrypt");
        let b = encrypt_metadata(&serde_json::json!({"x": 1})).expect("encrypt");
        // Fresh nonce per call → distinct ciphertexts for identical plaintext.
        assert_ne!(
            a.get(ENC_KEY).and_then(|v| v.as_str()),
            b.get(ENC_KEY).and_then(|v| v.as_str()),
            "nonce reuse produced identical ciphertexts"
        );
    }

    #[test]
    fn encrypt_metadata_is_idempotent_on_envelopes() {
        // Re-encrypting an already-encrypted envelope MUST NOT double-wrap it —
        // a double-wrapped envelope would be undecryptable (decrypt yields the
        // inner envelope, not the real metadata).
        field_crypto::set_key(&test_key()).ok();
        let plaintext = serde_json::json!({"wallet": "0xdeadbeef"});
        let envelope = encrypt_metadata(&plaintext).expect("encrypt");
        let re_encrypted = encrypt_metadata(&envelope).expect("idempotent re-encrypt");
        assert_eq!(
            envelope, re_encrypted,
            "encrypt_metadata must pass an existing envelope through unchanged"
        );
        // And the single-wrapped envelope still round-trips to the original.
        let recovered = decrypt_metadata(&Some(envelope)).expect("decrypt");
        assert_eq!(recovered, Some(plaintext));
    }

    #[test]
    fn is_encrypted_envelope_detector() {
        assert!(is_encrypted_envelope(&serde_json::json!({"enc": "YWJj"})));
        // Not an envelope: extra keys.
        assert!(!is_encrypted_envelope(&serde_json::json!({"enc": "YWJj", "extra": 1})));
        // Not an envelope: single key but wrong name (legacy plaintext).
        assert!(!is_encrypted_envelope(&serde_json::json!({"iban": "DE00"})));
        // Not an envelope: enc present but not a string.
        assert!(!is_encrypted_envelope(&serde_json::json!({"enc": 42})));
        // Not an envelope: not even an object.
        assert!(!is_encrypted_envelope(&serde_json::json!("plain string")));
    }
}
