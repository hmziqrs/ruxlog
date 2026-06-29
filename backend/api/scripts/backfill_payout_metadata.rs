#!/usr/bin/env cargo --bin ruxlog-backfill-payout-metadata

//! One-time operator backfill: re-encrypt every `payout_accounts.metadata` row
//! at rest (V-MED-11 / CWE-312).
//!
//! After V-MED-11 the model layer stores `metadata` as the encrypted envelope
//! `{"enc": "<base64(AES-256-GCM(nonce||ct||tag))>"}` on every write, and
//! `decrypt_metadata` transparently recovers the plaintext on read (a legacy
//! plaintext row is returned AS-IS with a warning, so a half-migrated DB does
//! not break reads). Rows written BEFORE the fix still hold their metadata as
//! PLAINTEXT JSON, so a database dump leaks every account's payout details.
//! This helper loads each account, sets any still-plaintext metadata back to
//! its plaintext value, and re-saves — `ActiveModelBehavior::before_save`
//! re-encrypts via `encrypt_metadata`. Rows that are already the `{"enc":...}`
//! envelope (or NULL) are skipped, so the tool is idempotent and safe to repeat.
//!
//! Run once with the field key and DB credentials in the process env (run with
//! `--dry-run` first to preview):
//!
//! ```sh
//! FIELD_ENC_KEY=<32-byte key> cargo run --features full \
//!     --bin ruxlog-backfill-payout-metadata -- --dry-run
//! ```
//!
//! See migration `m20260620_000051_payout_account_metadata_encryption_runbook`
//! and `docs/CRYPTO_AUDIT.md` (V-MED-11).

use clap::Parser;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, QuerySelect};

use ruxlog::db::sea_connect;
use ruxlog::db::sea_models::payout_account::{self, is_encrypted_envelope};
use ruxlog::state;

#[derive(Parser, Debug)]
#[command(name = "ruxlog-backfill-payout-metadata")]
#[command(about = "Re-encrypt legacy plaintext payout_accounts.metadata at rest")]
struct Args {
    /// Report what would change without writing anything.
    #[arg(long)]
    dry_run: bool,

    /// Only process the first N payout accounts (smoke-test on a subset).
    #[arg(long)]
    limit: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    println!(
        "🔐 payout-metadata backfill ({})",
        if args.dry_run {
            "DRY RUN — no writes"
        } else {
            "LIVE"
        }
    );

    // Reads FIELD_ENC_KEY (panics on prod+unset / wrong length — the same guard
    // as the server boot) and installs it into the process-wide slot the model
    // layer encrypts with. MUST run before any model write.
    println!("   • installing FIELD_ENC_KEY …");
    let _ = state::load_field_enc_key();

    println!("   • connecting to database …");
    let db = sea_connect::try_connect(false).await?;

    let mut query = payout_account::Entity::find();
    if let Some(n) = args.limit {
        query = query.limit(n);
    }
    let accounts = query.all(&db).await?;
    let total = accounts.len();
    println!("   • loaded {total} payout_account row(s)");

    let mut would_update = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for (idx, model) in accounts.into_iter().enumerate() {
        let id = model.id;
        // A row needs backfill iff it carries metadata that is NOT already the
        // `{"enc": ...}` envelope — i.e. legacy plaintext (or any non-envelope
        // shape). NULL metadata is skipped (no secret to protect).
        let needs = model
            .metadata
            .as_ref()
            .is_some_and(|m| !is_encrypted_envelope(m));

        if !needs {
            skipped += 1;
            continue;
        }
        would_update += 1;

        if args.dry_run {
            println!("   [dry-run] would re-encrypt payout_account id={id}");
            continue;
        }

        let metadata_plain = model.metadata.clone();
        let mut active: payout_account::ActiveModel = model.into();
        // `Set(plaintext)` makes `before_save` re-encrypt via `encrypt_metadata`.
        active.metadata = Set(metadata_plain);

        match active.update(&db).await {
            Ok(_) => updated += 1,
            Err(err) => {
                errors += 1;
                eprintln!("   ⚠️  payout_account id={id} failed: {err}");
            }
        }

        if (idx + 1) % 500 == 0 {
            println!("   … scanned {idx} row(s)");
        }
    }

    println!(
        "\n✅ done: scanned={total}, updated={updated}, would_update={would_update}, \
         skipped={skipped} (already encrypted / null), errors={errors}"
    );
    if errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}
