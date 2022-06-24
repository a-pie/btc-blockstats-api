mod flash;

use axum::{
    extract::{Extension, Form, Path, Query},
    http::StatusCode,
    response::Html,
    routing::{get, get_service, post},
    Router, Server,
};
use entity::post;
use flash::{get_flash_cookie, post_response, PostResponse};
use migration::{Migrator, MigratorTrait};
use post::Entity as Post;
use sea_orm::{prelude::*, Database, QueryOrder, Set, ConnectionTrait};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{env, net::SocketAddr};
use tera::Tera;
use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::services::ServeDir;

use entity::blockstats;
use blockstats::Entity as BlockStats;

use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::json::GetBlockStatsResult;

use sea_orm::DbBackend;
use sea_orm::Statement;

extern crate jsonrpc;
// todo() get from env
const INDEX_FROM: u64 = 630000;
const INTERVAL: u64 = 1000;
const RETRY_ATTEMPTS: u8 = 10;
const USERNAME: &str = "mynode";
const PASSWORD: &str = "3uEi4nZGOmMonHdLzQWPSypD";
const URL: &str = "http://mynode.local:8332";
const DATABASE_URL: &str = "postgres://postgres:pac123@localhost/axum_example8";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);




    let conn2 = Database::connect(DATABASE_URL)
        .await
        .expect("Database connection failed");  
    Migrator::up(&conn2, None).await.unwrap();
    let max_height_in_db = get_db_tip(&conn2).await.unwrap();
    let temp = u64::try_from(max_height_in_db).unwrap();
    let r = create_stat(&conn2, temp).await;





    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"))
        .expect("Tera initialization failed");
    // let state = AppState { templates, conn };

    let app = Router::new()
        .route("/", get(list_posts).post(create_post))
        .nest(
            "/static",
            get_service(ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static"
            )))
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(Extension(conn))
                .layer(Extension(templates)),
        );

    let addr = SocketAddr::from_str(&server_url).unwrap();
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[derive(Deserialize)]
struct Params {
    page: Option<usize>,
    posts_per_page: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

async fn list_posts(
    Extension(ref templates): Extension<Tera>,
    Extension(ref conn): Extension<DatabaseConnection>,
    Query(params): Query<Params>,
    cookies: Cookies,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);
    let paginator = Post::find()
        .order_by_asc(post::Column::Id)
        .paginate(conn, posts_per_page);
    let num_pages = paginator.num_pages().await.ok().unwrap();
    let posts = paginator
        .fetch_page(page - 1)
        .await
        .expect("could not retrieve posts");

    let mut ctx = tera::Context::new();
    ctx.insert("posts", &posts);
    ctx.insert("page", &page);
    ctx.insert("posts_per_page", &posts_per_page);
    ctx.insert("num_pages", &num_pages);

    if let Some(value) = get_flash_cookie::<FlashData>(&cookies) {
        ctx.insert("flash", &value);
    }

    let body = templates
        .render("index.html.tera", &ctx)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error"))?;

    Ok(Html(body))
}


async fn create_post(
    Extension(ref conn): Extension<DatabaseConnection>,
    form: Form<post::Model>,
    mut cookies: Cookies,
) -> Result<PostResponse, (StatusCode, &'static str)> {
    let model = form.0;

    post::ActiveModel {
        title: Set(model.title.to_owned()),
        text: Set(model.text.to_owned()),
        ..Default::default()
    }
    .save(conn)
    .await
    .expect("could not insert post");

    let data = FlashData {
        kind: "success".to_owned(),
        message: "Post succcessfully added".to_owned(),
    };

    Ok(post_response(&mut cookies, data))
}



async fn create_stat(conn: &DatabaseConnection, index_from_height: u64) -> Result<(), &'static str> {
    let rpc = Client::new(
        &URL,
        Auth::UserPass(USERNAME.to_string(), PASSWORD.to_string()),
    )
    .unwrap();
    let rpc = RetryClient { client: rpc };
    
    let bestblockcount = rpc.get_block_count().unwrap();
    println!("bestblockcount: {}", bestblockcount);

    if (bestblockcount == index_from_height)
    {
        println!("no new blocks");
        return Ok(());
    } else {
        //TODO test this from genesis block to see if first height is 0 or 1
        for i in index_from_height+1..bestblockcount {
            println!("block_stats at height: {} out of: {}", i, bestblockcount);
            let blockstats = rpc.get_block_stats(i).unwrap();
                blockstats::ActiveModel {
                    block_hash: Set(blockstats.block_hash.to_string().to_owned()),
                    height: Set(blockstats.height.to_owned() as i64),
                    avg_fee: Set(blockstats.avg_fee.as_btc().to_owned()),
                    avg_fee_rate: Set(blockstats.avg_fee_rate.as_btc().to_owned()),
                    avg_tx_size: Set(blockstats.avg_tx_size.to_owned() as i64),
                    ins: Set(blockstats.ins.to_owned() as i64),
                    max_fee: Set(blockstats.max_fee.as_btc().to_owned()),
                    max_fee_rate: Set(blockstats.max_fee_rate.as_btc().to_owned()),
                    max_tx_size: Set(blockstats.max_tx_size.to_owned() as i64),
                    median_fee: Set(blockstats.median_fee.as_btc().to_owned()),
                    median_time: Set(blockstats.median_time.to_owned() as i64),
                    median_tx_size: Set(blockstats.median_tx_size.to_owned() as i64),
                    outs: Set(blockstats.outs.to_owned() as i64),
                    subsidy: Set(blockstats.subsidy.as_btc().to_owned()),
                    sw_total_size: Set(blockstats.sw_total_size.to_owned() as i64),
                    sw_total_weight: Set(blockstats.sw_total_weight.to_owned() as i64),
                    sw_txs: Set(blockstats.sw_txs.to_owned() as i64),
                    time:  Set(blockstats.time.to_owned() as i64),
                    total_out: Set(blockstats.total_out.as_btc().to_owned()),
                    total_size: Set(blockstats.total_size.to_owned() as i64),
                    total_weight: Set(blockstats.total_weight.to_owned() as i64),
                    total_fee: Set(blockstats.total_fee.as_btc().to_owned()),
                    txs: Set(blockstats.txs.to_owned() as i64),
                    utxo_increase: Set(blockstats.utxo_increase.to_owned()),
                    utxo_size_inc: Set(blockstats.utxo_size_inc.to_owned()),
                    ..Default::default()
                }
                .save(conn)
                .await
                .expect("could not insert blockstats");
        }
    }


    Ok(())
}

async fn get_db_tip(db: &DatabaseConnection) -> Result<i64, sea_orm::DbErr> {

   /* //Working Examples:
   let block : blockstats::ActiveModel = BlockStats::find()
        .one(db)
        .await
        .unwrap()
        .unwrap()
        .into();
    println!("block: {:?}", block);*/

    /*let block2 : Vec<blockstats::Model> = BlockStats::find()
        .filter(blockstats::Column::BlockHash.eq("000000000000000000071bd6722a25e18d718b84e7173af76304ac1fb13cfb0e"))
        .all(db)
        .await?;
    println!("block2: {:?}", block2);*/


    let query_res_vec = db
    .query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT max(height) FROM block_stats;".to_owned(),
    ))
    .await?;

    let query_res = query_res_vec.unwrap();
    let height: i64 = query_res.try_get("", "max").unwrap_or(0);
    println!("Database Height Tip: {:?}", height);

    Ok(height)
}


pub struct RetryClient {
    client: Client,
}
impl bitcoincore_rpc::RpcApi for RetryClient {
    fn call<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        cmd: &str,
        args: &[serde_json::Value],
    ) -> bitcoincore_rpc::Result<T> {
        for _ in 0..RETRY_ATTEMPTS {
            match self.client.call(cmd, args) {
                Ok(ret) => return Ok(ret),
                Err(bitcoincore_rpc::Error::JsonRpc(jsonrpc::error::Error::Rpc(ref rpcerr)))
                    if rpcerr.code == -28 =>
                {
                    ::std::thread::sleep(::std::time::Duration::from_millis(INTERVAL));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        self.client.call(cmd, args)
    }
}