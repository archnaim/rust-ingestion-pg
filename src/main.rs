mod database;

use database::models::DataModel;
use dotenv::dotenv;
use tokio_postgres::types::Type;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let db_source_pool = database::source_db::create_pool().await?;

    // Make a simple query to return the given parameter
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&db_source_pool)
        .await?;

    println!("{}", row.0);

    db_source_pool.close().await;

    let mut db_sink_client = database::sink_db::create_conn().await;

    // Now we can execute a simple statement that just returns its parameter.
    let rows =
        database::sink_db::query(&db_sink_client, "SELECT $1::TEXT, $2::TEXT", &[&"hello", &"world"]).await?;
    
    let rows1 =
        database::sink_db::query(&db_sink_client, "SELECT 1::TEXT", &[]).await?;


    let value1:&str = rows[0].get(0);
    let value2:&str = rows[0].get(1);
    let value = format!("{} {}", &value1, &value2);
    println!("{}",&value);
    assert_eq!(value, "hello world");
    let value3:&str = rows1[0].get(0);
    println!("{}", &value3);
    db_sink_client.is_closed();


    let copy_statement = "COPY data (id, name) FROM STDIN BINARY";

    let col_types = vec![Type::OID, Type::VARCHAR];
    let data = vec![DataModel{id: 0, name:String::from("test")},DataModel{id: 1, name:String::from("test1")}];
    
    let num_data =database::sink_db::import_data(&mut db_sink_client, &copy_statement, &data, &col_types).await?;
    print!("num data: {}", num_data);

    Ok(())
}
