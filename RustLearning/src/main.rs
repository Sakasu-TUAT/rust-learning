use dotenv::dotenv;
use std::env;
use std::io::{self, BufRead};
use tokio_postgres::{NoTls, Error};
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // .env ファイルを読み込み
    dotenv().ok();

    // 環境変数から接続情報を取得
    let db_host = env::var("DB_HOST").expect("DB_HOST not set");
    let db_port = env::var("DB_PORT").expect("DB_PORT not set");
    let db_user = env::var("DB_USER").expect("DB_USER not set");
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD not set");
    let db_name = env::var("DB_NAME").expect("DB_NAME not set");

    // PostgreSQLの接続情報を設定
    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        db_host, db_port, db_user, db_password, db_name
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;

    // 接続タスクをスポーンして実行
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // テーブル作成のクエリを実行
    client
        .batch_execute(
            "
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL
            )
            ",
        )
        .await?;

    // 標準入力からキー入力を受け取るためのストリームを作成
    let stdin = io::stdin();
    let mut input_stream = stdin.lock().lines();

    println!("Press 'a' to add a record, 'd' to delete all records, 's' to sort and display records, or any other key to exit.");

    // キー入力を監視し、対応する操作を実行します
    while let Some(Ok(input)) = input_stream.next() {
        match input.as_str() {
            "a" => {
                // レコードの追加
                let mut rng = rand::thread_rng();
                let id: i32 = rng.gen();

                client
                    .execute(
                        "INSERT INTO users (id, name) VALUES ($1, $2)",
                        &[&id, &"John Doe"],
                    )
                    .await?;
                println!("Record added.");
            }
            "d" => {
                // レコードの削除
                client.execute("DELETE FROM users", &[]).await?;
                println!("All records deleted.");
            }
            "s" => {
                // レコードのソートと表示
                let rows = client
                    .query("SELECT id, name FROM users ORDER BY id", &[])
                    .await?;
                for row in rows {
                    let id: i32 = row.get(0);
                    let name: &str = row.get(1);
                    println!("ID: {}, Name: {}", id, name);
                }
            }
            _ => {
                break;
            }
        }
    }

    Ok(())
}
