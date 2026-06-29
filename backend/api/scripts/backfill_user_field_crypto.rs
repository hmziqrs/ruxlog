#!/usr/bin/env cargo --bin ruxlog-backfill-user-field-crypto

//! One-time operator backfill: re-encrypt every user row's `two_fa_secret` and
//! `google_id` at rest (CRYP-2FA-002 / CRYP-ENC-013 / CRYP-ENC-004).
//!
//! After the field-crypto fix the model layer encrypts these columns on every
//! write and transparently decrypts on read (a legacy plaintext value is
//! returned AS-IS, so a half-migrated DB keeps working — TOTP verify and OAuth
//! lookup still succeed). Rows written BEFORE the fix, however, still hold
//! PLAINTEXT, which means `find_by_google_id` cannot match them via the
//! encrypted lookup column. This helper loads each user, sets any
//! still-plaintext field back to its plaintext value, and re-saves —
//! `ActiveModelBehavior::before_save` re-encrypts under the current key.
//! Already-encrypted rows are detected and skipped, so the tool is idempotent
//! and safe to run repeatedly.
//!
//! Run once with the field key and DB credentials in the process env (run with
//! `--dry-run` first to preview):
//!
//! ```sh
//! FIELD_ENC_KEY=<32-byte key> cargo run --features full \
//!     --bin ruxlog-backfill-user-field-crypto -- --dry-run
//! ```
//!
//! See migration `m20260627_000052_alter_user_add_session_auth_secret_and_encrypt_fields`
//! and `docs/CRYPTO_AUDIT.md`.

use clap::Parser;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QuerySelect};

use ruxlog::db::sea_connect;
use ruxlog::db::sea_models::user::{self, looks_like_envelope};
use ruxlog::state;

#[derive(Parser, Debug)]
#[command(name = "ruxlog-backfill-user-field-crypto")]
#[command(about = "Re-encrypt legacy plaintext user.two_fa_secret / google_id at rest")]
struct Args {
    /// Report what would change without writing anything.
    #[arg(long)]
    dry_run: bool,

    /// Only process the first N users (smoke-test on a subset).
    #[arg(long)]
    limit: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    println!(
        "🔐 user-field-crypto backfill ({})",
        if args.dry_run {
            "DRY RUN — no writes"
        } else {
            "LIVE"
        }
    );

    // Reads FIELD_ENC_KEY (panics on prod+unset / wrong length — the same guard
    // as the server boot) and installs it into the process-wide slot the model
    // layer encrypts with. MUST run before any model write, or `before_save`
    // fails closed with KeyUnset.
    println!("   • installing FIELD_ENC_KEY …");
    let _ = state::load_field_enc_key();

    println!("   • connecting to database …");
    let db = sea_connect::try_connect(false).await?;

    let mut query = user::Entity::find();
    if let Some(n) = args.limit {
        query = query.limit(n);
    }
    let users = query.all(&db).await?;
    let total = users.len();
    println!("   • loaded {total} user row(s)");

    let mut would_update = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for (idx, model) in users.into_iter().enumerate() {
        let id = model.id;
        // A field needs re-encryption iff it holds a value that is NOT already a
        // field-crypto envelope (i.e. legacy plaintext). `None` and already-
        // encrypted envelopes are left untouched.
        let needs_two_fa = model
            .two_fa_secret
            .as_deref()
            .is_some_and(|v| !looks_like_envelope(v));
        let needs_google = model
            .google_id
            .as_deref()
            .is_some_and(|v| !looks_like_envelope(v));

        if !needs_two_fa && !needs_google {
            skipped += 1;
            continue;
        }
        would_update += 1;

        if args.dry_run {
            println!(
                "   [dry-run] would re-encrypt user id={id} \
                 (two_fa_secret={needs_two_fa}, google_id={needs_google})"
            );
            continue;
        }

        // Capture the plaintext BEFORE moving the model into an ActiveModel.
        let two_fa_plain = model.two_fa_secret.clone();
        let google_plain = model.google_id.clone();
        let mut active: user::ActiveModel = model.into();
        // `Set(plaintext)` makes `before_save` re-encrypt under the current key.
        // Fields left `Unchanged` (incl. `session_auth_secret`) are not written.
        if needs_two_fa {
            active.two_fa_secret = Set(two_fa_plain);
        }
        if needs_google {
            active.google_id = Set(google_plain);
        }

        match active.update(&db).await {
            Ok(_) => updated += 1,
            Err(err) => {
                errors += 1;
                eprintln!("   ⚠️  user id={id} failed: {err}");
            }
        }

        if (idx + 1) % 500 == 0 {
            println!("   … scanned {idx} row(s)");
        }
    }

    println!(
        "\n✅ done: scanned={total}, updated={updated}, would_update={would_update}, \
         skipped={skipped} (already encrypted / empty), errors={errors}"
    );
    if errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}
