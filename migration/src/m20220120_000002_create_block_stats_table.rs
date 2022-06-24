use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220120_000002_create_block_stats_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BlockStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BlockStats::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BlockStats::BlockHash).string().not_null())
                    .col(ColumnDef::new(BlockStats::Height).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::AvgFee).double().not_null())
                    .col(ColumnDef::new(BlockStats::AvgFeeRate).double().not_null())
                    .col(ColumnDef::new(BlockStats::AvgTxSize).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::Ins).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::MaxFee).double().not_null())
                    .col(ColumnDef::new(BlockStats::MaxFeeRate).double().not_null())
                    .col(ColumnDef::new(BlockStats::MaxTxSize).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::MedianFee).double().not_null())
                    .col(ColumnDef::new(BlockStats::MedianTime).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::MedianTxSize).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::Outs).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::Subsidy).double().not_null())
                    .col(ColumnDef::new(BlockStats::SwTotalSize).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::SwTotalWeight).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::SwTxs).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::Time).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::TotalOut).double().not_null())
                    .col(ColumnDef::new(BlockStats::TotalSize).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::TotalWeight).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::TotalFee).double().not_null())
                    .col(ColumnDef::new(BlockStats::Txs).big_integer().not_null())
                    .col(ColumnDef::new(BlockStats::UtxoIncrease).integer().not_null())
                    .col(ColumnDef::new(BlockStats::UtxoSizeInc).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BlockStats::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum BlockStats {
    Table,
    Id,
    BlockHash,
    Height,
    AvgFee,
    AvgFeeRate,
    AvgTxSize,
    Ins,
    MaxFee,
    MaxFeeRate,
    MaxTxSize,
    MedianFee,
    MedianTime,
    MedianTxSize,
    Outs,
    Subsidy,
    SwTotalSize,
    SwTotalWeight,
    SwTxs,
    Time,
    TotalOut,
    TotalSize,
    TotalWeight,
    TotalFee,
    Txs,
    UtxoIncrease,
    UtxoSizeInc,
}
