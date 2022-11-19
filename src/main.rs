pub mod config;
pub mod db;
pub mod fetcher;
pub mod rpc;
pub mod utils;

use dotenv::dotenv;
use log::LevelFilter;

use crate::config::IndexerConfig;
use crate::db::IndexerDB;

use crate::fetcher::fetch_blocks_range_workers;
use crate::rpc::IndexerRPC;

use simple_logger::SimpleLogger;

#[tokio::main(worker_threads = 2)]

async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    dotenv().ok();

    log::info!("Starting EVM Indexer");

    let config = IndexerConfig::new();

    let db = IndexerDB::new(&config.db_url, &config.db_name, config.initial_block)
        .await
        .expect("Unable to connect to the database");
    let rpc = IndexerRPC::new(&config.rpc_ws_url, &config.rpc_http_url)
        .await
        .expect("Unable to connect to the rpc url");

    // Get the last synced block and compare with the RPC
    let last_synced_block: i64 = db.last_synced_block().await.unwrap();
    let last_chain_block: i64 = rpc.last_block().await.unwrap();

    log::info!("==> Main: Last DB Synced Block: {}", last_synced_block);
    log::info!("==> Main: Last Chain Block: {}", last_chain_block);

    tokio::join!(
        rpc.subscribe_heads(&db),
        fetch_blocks_range_workers(
            &rpc,
            &db,
            last_synced_block + 1,
            last_chain_block,
            &config.batch_size,
            &config.workers
        ),
    );
}
