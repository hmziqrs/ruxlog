use sea_orm_migration::prelude::*;

/// `post_purchases` records a user's one-time purchase of a gated post, granting
/// permanent read access to that post's `content`. It is the entitlement row
/// consulted by the server-side paywall (`services/paywall`) for
/// `PostAccessType::Paid` posts. Rows are created by the **verified** billing
/// webhook from a server-bound checkout intent — never from client JSON — so a
/// forged or replayed webhook cannot grant access (see plan Phase 4d/4e).
///
/// Follows the `m20260512_000045_create_post_access_table` pattern.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostPurchases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostPurchases::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostPurchases::UserId).integer().not_null())
                    .col(ColumnDef::new(PostPurchases::PostId).integer().not_null())
                    // Optional link to the `payments` row that recorded the
                    // transaction. Nullable because the webhook may insert the
                    // purchase before/without a corresponding payment row.
                    .col(ColumnDef::new(PostPurchases::PaymentId).integer())
                    .col(
                        ColumnDef::new(PostPurchases::Provider)
                            .string_len(40)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostPurchases::AmountCents)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostPurchases::Currency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(PostPurchases::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_purchases_user_id")
                            .from(PostPurchases::Table, PostPurchases::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_purchases_post_id")
                            .from(PostPurchases::Table, PostPurchases::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_purchases_payment_id")
                            .from(PostPurchases::Table, PostPurchases::PaymentId)
                            .to(Payments::Table, Payments::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // A user only needs to buy a given post once — permanent access. The
        // unique constraint also makes the webhook grant idempotent: replaying a
        // verified purchase event upserts rather than duplicating.
        manager
            .create_index(
                Index::create()
                    .name("idx_post_purchases_user_post_unique")
                    .table(PostPurchases::Table)
                    .col(PostPurchases::UserId)
                    .col(PostPurchases::PostId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Lookup index for the paywall's "did this user buy this post" check.
        manager
            .create_index(
                Index::create()
                    .name("idx_post_purchases_post_id")
                    .table(PostPurchases::Table)
                    .col(PostPurchases::PostId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostPurchases::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostPurchases {
    Table,
    Id,
    UserId,
    PostId,
    PaymentId,
    Provider,
    AmountCents,
    Currency,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}

#[derive(Iden)]
enum Payments {
    Table,
    Id,
}
