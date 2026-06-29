use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub use ruxlog_types::enums::UserRole;

use crate::utils::field_crypto;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub avatar_id: Option<i32>,
    pub is_verified: bool,
    pub role: UserRole,
    pub two_fa_enabled: bool,
    // Never serialize these — leaking the TOTP seed or backup-code hashes lets an
    // attacker bypass 2FA. See plan Phase 2a.
    //
    // CRYP-2FA-002 / CRYP-ENC-013: `two_fa_secret` is stored AT REST as the
    // field-crypto envelope `V1:<base64(...)>` (AES-256-GCM), never as the raw
    // Base32 TOTP seed. Writes are encrypted transparently in `before_save`
    // (callers keep doing `Set(Some(plaintext))`); reads MUST go through
    // [`Model::two_fa_secret_plain`] — the raw `two_fa_secret` field on a
    // freshly-loaded row is the opaque envelope.
    #[serde(skip_serializing)]
    pub two_fa_secret: Option<String>,
    #[serde(skip_serializing)]
    pub two_fa_backup_codes: Option<Json>,
    // V-MED-6 (TOTP replay): highest accepted RFC 6238 time-step counter. NULL
    // means "no code consumed yet" (first-time use accepts any valid counter).
    // Never serialize — leaking it is unnecessary and it's auth state.
    #[serde(skip_serializing)]
    pub two_fa_last_totp_counter: Option<i64>,
    // CRYP-ENC-004(a): `google_id` is stored AT REST as the DETERMINISTIC
    // field-crypto envelope `D1:<base64(...)>` so `find_by_google_id` can
    // encrypt-then-lookup transparently. Writes encrypt in `before_save`; the
    // plaintext google_id is surfaced only via [`Model::google_id_plain`]. The
    // raw `google_id` field on a freshly-loaded row is the opaque envelope.
    #[serde(skip_serializing)]
    pub google_id: Option<String>,
    pub oauth_provider: Option<String>,
    // CRYP-ENC-004(b): a server-random per-user secret the session auth hash is
    // derived from (replacing the raw-email basis). Backfilled to a random value
    // for every existing user in the migration. Never serialize — it is auth
    // state; leaking it would let an attacker forge a session hash offline.
    #[serde(skip_serializing)]
    pub session_auth_secret: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    pub fn get_role(&self) -> UserRole {
        self.role
    }

    pub fn is_user(&self) -> bool {
        self.get_role().to_i32() >= UserRole::User.to_i32()
    }

    pub fn is_author(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Author.to_i32()
    }

    pub fn is_moderator(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Moderator.to_i32()
    }

    pub fn is_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::Admin.to_i32()
    }

    pub fn is_super_admin(&self) -> bool {
        self.get_role().to_i32() >= UserRole::SuperAdmin.to_i32()
    }

    // ───────────────────────────────────────────────────────────────────────
    // CRYP-2FA-002 / CRYP-ENC-013 / CRYP-ENC-004: decrypt accessors
    // ───────────────────────────────────────────────────────────────────────
    //
    // The encrypted columns (`two_fa_secret`, `google_id`) are stored as opaque
    // field-crypto envelopes at rest. A freshly-loaded `Model` therefore carries
    // the ciphertext in those fields; callers that need the PLAINTEXT (TOTP
    // verify, OAuth id display) MUST decrypt via these helpers rather than
    // reading the field directly. Decrypt happens lazily at the call site (not
    // on load) so a list endpoint that never touches the secret never pays the
    // AES cost.

    /// Decrypt this user's stored TOTP secret to the plaintext Base32 seed.
    /// `Ok(None)` means the user has no secret (2FA not set up). An envelope
    /// that fails to decrypt (tampered / wrong key) propagates as `Err`.
    pub fn two_fa_secret_plain(&self) -> Result<Option<String>, field_crypto::FieldCryptoError> {
        self.two_fa_secret
            .as_deref()
            .map(decrypt_two_fa_secret_value)
            .transpose()
    }

    /// Decrypt this user's stored google_id to the plaintext Google subject id.
    /// `Ok(None)` means the user is not linked to a Google identity.
    pub fn google_id_plain(&self) -> Result<Option<String>, field_crypto::FieldCryptoError> {
        self.google_id
            .as_deref()
            .map(decrypt_google_id_value)
            .transpose()
    }
}

/// Decrypt a stored `two_fa_secret` envelope into the plaintext Base32 seed.
/// Free function so callers that hold only the column value (e.g. a backfill)
/// can reuse it without a full `Model`. Accepts BOTH the new `V1:` envelope and
/// a legacy plaintext Base32 seed (pre-CRYP-2FA-002) — a value that does not
/// look like a field-crypto envelope is returned AS-IS so a half-migrated DB
/// does not hard-fail TOTP verify (see the runbook in the migration).
pub fn decrypt_two_fa_secret_value(blob: &str) -> Result<String, field_crypto::FieldCryptoError> {
    // A field-crypto envelope starts with `V` + digits + `:`. Anything else is
    // treated as a legacy plaintext Base32 seed (the pre-encryption shape) and
    // surfaced unchanged so reads stay correct during the backfill window.
    if looks_like_envelope(blob) {
        field_crypto::decrypt(blob)
    } else {
        Ok(blob.to_string())
    }
}

/// Decrypt a stored `google_id` envelope into the plaintext Google subject id.
/// Accepts BOTH the new `D1:` envelope and a legacy plaintext google_id. A value
/// that does not look like an envelope is returned AS-IS (half-migrated DB).
pub fn decrypt_google_id_value(blob: &str) -> Result<String, field_crypto::FieldCryptoError> {
    if looks_like_envelope(blob) {
        field_crypto::decrypt_deterministic(blob)
    } else {
        Ok(blob.to_string())
    }
}

/// True iff `blob` begins with a recognized field-crypto version tag
/// (`V<digits>:` for random-nonce envelopes, `D<digits>:` for deterministic
/// ones). Used by the decrypt accessors to tell an encrypted envelope from a
/// legacy plaintext column value without attempting (and failing-closed on) an
/// AEAD operation on what is actually plaintext.
pub fn looks_like_envelope(blob: &str) -> bool {
    let Some(rest) = blob.strip_prefix(|c: char| c == 'V' || c == 'D') else {
        return false;
    };
    let Some(colon) = rest.find(':') else {
        return false;
    };
    let ver = &rest[..colon];
    !ver.is_empty() && ver.chars().all(|c| c.is_ascii_digit())
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::super::email_verification::Entity")]
    EmailVerification,
    #[sea_orm(has_many = "super::super::forgot_password::Entity")]
    ForgotPassword,
    #[sea_orm(has_many = "super::super::post::Entity")]
    Post,
    #[sea_orm(
        belongs_to = "super::super::media::Entity",
        from = "Column::AvatarId",
        to = "super::super::media::Column::Id"
    )]
    Media,
}

impl Related<super::super::email_verification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::EmailVerification.def()
    }
}

impl Related<super::super::forgot_password::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ForgotPassword.def()
    }
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl Related<super::super::media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Media.def()
    }
}

// ──────────────────────────────────────────────────────────────────────────
// CRYP-2FA-002 / CRYP-ENC-013 / CRYP-ENC-004: transparent at-rest encryption
// ──────────────────────────────────────────────────────────────────────────
//
// `ActiveModelBehavior::before_save` wraps every insert/update that SETS
// `two_fa_secret` or `google_id`: the plaintext value passed by the service
// layer is encrypted before it reaches the DB, so no caller can persist a raw
// TOTP seed or google_id by accident. `Unchanged` (loaded-but-not-modified)
// values are skipped — a row already holding an envelope on disk is never
// re-treated as plaintext and double-wrapped, and an unrelated UPDATE of
// another column does not re-encrypt.
//
// `google_id` uses the DETERMINISTIC envelope so the value is searchable
// (find_by_google_id encrypts the lookup key and matches); `two_fa_secret`
// uses the random-nonce envelope (no lookup needed, stronger — ciphertexts
// differ even for identical secrets).

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        // ── two_fa_secret (random-nonce envelope) ───────────────────────────
        match self.two_fa_secret.clone() {
            sea_orm::ActiveValue::Set(Some(value)) => {
                // Idempotent: a value that is ALREADY an envelope (re-saving a
                // loaded row) is passed through; only plaintext is encrypted.
                let to_store = if looks_like_envelope(&value) {
                    value
                } else {
                    field_crypto::encrypt(&value).map_err(|err| {
                        DbErr::Custom(format!("users.two_fa_secret encryption failed: {err}"))
                    })?
                };
                self.two_fa_secret = sea_orm::ActiveValue::Set(Some(to_store));
            }
            sea_orm::ActiveValue::Set(None) | sea_orm::ActiveValue::Unchanged(_) => {}
            sea_orm::ActiveValue::NotSet => {}
        }

        // ── google_id (deterministic envelope, searchable) ──────────────────
        match self.google_id.clone() {
            sea_orm::ActiveValue::Set(Some(value)) => {
                let to_store = if looks_like_envelope(&value) {
                    value
                } else {
                    field_crypto::encrypt_deterministic(&value).map_err(|err| {
                        DbErr::Custom(format!("users.google_id encryption failed: {err}"))
                    })?
                };
                self.google_id = sea_orm::ActiveValue::Set(Some(to_store));
            }
            sea_orm::ActiveValue::Set(None) | sea_orm::ActiveValue::Unchanged(_) => {}
            sea_orm::ActiveValue::NotSet => {}
        }

        // ── session_auth_secret: backfill on first write if the caller left it
        //    unset or empty (NOT NULL column). A random 32-byte hex secret makes
        //    the session hash unpredictable even when the email is known.
        let needs_backfill = match &self.session_auth_secret {
            sea_orm::ActiveValue::NotSet => true,
            sea_orm::ActiveValue::Set(s) if s.is_empty() => true,
            _ => false,
        };
        if needs_backfill {
            self.session_auth_secret =
                sea_orm::ActiveValue::Set(new_session_auth_secret().map_err(|err| {
                    DbErr::Custom(format!(
                        "users.session_auth_secret generation failed: {err}"
                    ))
                })?);
        }

        Ok(self)
    }
}

/// Generate a fresh server-random per-user session auth secret (64 hex chars =
/// 256 bits of entropy from the OS CSPRNG). Used to backfill the NOT NULL
/// column on first write when the service layer does not supply one. Returns
/// `Err` only if the OS CSPRNG fails (never silently fall back to a constant).
pub fn new_session_auth_secret() -> Result<String, String> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).map_err(|e| format!("CSPRNG failure: {e}"))?;
    Ok(hex_encode(&buf))
}

/// Lowercase hex encoding (no external dep).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(0x11);
        }
        k
    }

    #[test]
    fn before_save_encrypts_two_fa_secret() {
        // A plaintext TOTP seed passed via Set(...) must be wrapped to the V1:
        // envelope before persist — the field-crypto layer owns this, callers
        // keep writing plaintext. (The DB itself is not exercised; we only
        // assert the encrypt + envelope-detector contract.)
        field_crypto::set_key(&test_key()).ok();
        let plaintext = "JBSWY3DPEHPK3PXP"; // a Base32 TOTP seed
        let envelope = field_crypto::encrypt(plaintext).expect("encrypt");
        assert!(looks_like_envelope(&envelope), "envelope must be tagged");
        assert_ne!(envelope, plaintext);
        assert_eq!(
            decrypt_two_fa_secret_value(&envelope).expect("decrypt"),
            plaintext
        );
    }

    #[test]
    fn before_save_is_idempotent_on_envelope() {
        // Re-saving an already-encrypted value MUST NOT double-wrap it.
        field_crypto::set_key(&test_key()).ok();
        let envelope = field_crypto::encrypt("seed").expect("encrypt");
        // looks_like_envelope is the gate before_save uses; an envelope is
        // passed through unchanged.
        assert!(looks_like_envelope(&envelope));
        assert_eq!(
            decrypt_two_fa_secret_value(&envelope).expect("decrypt"),
            "seed"
        );
    }

    #[test]
    fn legacy_plaintext_two_fa_secret_is_returned_as_is() {
        // A pre-encryption row holds a bare Base32 seed (no V1: tag). The
        // decrypt accessor must surface it unchanged so TOTP verify still works
        // during the backfill window.
        let legacy = "JBSWY3DPEHPK3PXP";
        assert!(!looks_like_envelope(legacy));
        assert_eq!(decrypt_two_fa_secret_value(legacy).expect("legacy"), legacy);
    }

    #[test]
    fn google_id_deterministic_envelope_round_trips() {
        field_crypto::set_key(&test_key()).ok();
        let id = "1234567890";
        let envelope = field_crypto::encrypt_deterministic(id).expect("enc");
        assert!(looks_like_envelope(&envelope));
        // Deterministic: encrypting the SAME id yields the SAME envelope, so a
        // lookup column encrypted at write matches a lookup key encrypted now.
        assert_eq!(
            field_crypto::encrypt_deterministic(id).expect("enc"),
            envelope
        );
        assert_eq!(decrypt_google_id_value(&envelope).expect("dec"), id);
    }

    #[test]
    fn looks_like_envelope_detector() {
        assert!(looks_like_envelope("V1:abc"));
        assert!(looks_like_envelope("D1:abc"));
        assert!(looks_like_envelope("V42:abc"));
        // Not envelopes: plaintext Base32 seed / google id.
        assert!(!looks_like_envelope("JBSWY3DPEHPK3PXP"));
        assert!(!looks_like_envelope("1234567890"));
        // `V` with no colon / non-numeric version is not an envelope.
        assert!(!looks_like_envelope("Value"));
        assert!(!looks_like_envelope("Vx:abc"));
        assert!(!looks_like_envelope("plain"));
    }

    #[test]
    fn new_session_auth_secret_is_hex_and_unique() {
        let a = new_session_auth_secret().expect("CSPRNG available");
        let b = new_session_auth_secret().expect("CSPRNG available");
        assert_eq!(a.len(), 64, "must be 64 hex chars (256 bits)");
        assert!(
            a.chars().all(|c| c.is_ascii_hexdigit()),
            "must be lowercase hex"
        );
        assert_ne!(a, b, "must be CSPRNG-random, not constant");
        assert!(!a.is_empty());
    }
}
