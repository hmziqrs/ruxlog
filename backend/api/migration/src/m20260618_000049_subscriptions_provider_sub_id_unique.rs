use sea_orm_migration::prelude::*;

/// Makes a provider's subscription id globally unique within that provider, so
/// a replayed or concurrent billing webhook can never insert two `subscriptions`
/// rows for the same external subscription (audit F#1 / CWE-294 idempotency).
///
/// The existing index `idx_subscriptions_provider_sub_id` (migration
/// `m20260512_000038`) is a plain (non-unique) composite on
/// `(provider, provider_subscription_id)`. It speeds lookups but does not
/// *prevent* duplicates, so a race between two verified webhook deliveries — or
/// a provider that redelivers an event — could grant the same subscription
/// twice. The webhook grant path is now duplicate-tolerant (it treats a unique
/// violation as success), and this unique index is the hard backstop.
///
/// On PostgreSQL, `NULL` values are considered distinct, so multiple rows with
/// `provider_subscription_id IS NULL` (legacy/manual subscriptions with no
/// external id) coexist — only genuinely equal, non-null `(provider,
/// provider_subscription_id)` pairs collide, which is exactly the duplicate we
/// want to forbid. No partial `WHERE` clause is required.
///
/// If duplicate non-null rows already exist when this runs, `CREATE UNIQUE
/// INDEX` will fail; in that case deduplicate before retrying (keep the newest
/// `status = Active` row per `(provider, provider_subscription_id)`).
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the non-unique lookup index first so the new unique index can
        // take its place without a redundant second index on the same columns.
        if manager
            .has_index("subscriptions", "idx_subscriptions_provider_sub_id")
            .await?
        {
            manager
                .drop_index(
                    Index::drop()
                        .name("idx_subscriptions_provider_sub_id")
                        .table(Subscriptions::Table)
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_provider_sub_id_unique")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::Provider)
                    .col(Subscriptions::ProviderSubscriptionId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_subscriptions_provider_sub_id_unique")
                    .table(Subscriptions::Table)
                    .to_owned(),
            )
            .await?;

        // Restore the original non-unique lookup index.
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_provider_sub_id")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::Provider)
                    .col(Subscriptions::ProviderSubscriptionId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Subscriptions {
    Table,
    Provider,
    ProviderSubscriptionId,
}
