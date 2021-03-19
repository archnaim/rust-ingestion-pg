use crate::database::models::DataModel;
use futures::pin_mut;
use std::{ env, usize};
use tokio_postgres::{
    binary_copy::BinaryCopyInWriter,
    types::{ToSql, Type},
    Client, Error, NoTls, Row,
};
use std::convert::TryInto;

pub async fn create_conn() -> Client {
    let res = tokio_postgres::connect(
        &env::var("SINK_DB").expect("Env Variable SINK_DB is missing "),
        NoTls,
    )
    .await;
    let (client, connection) = res.expect("Cannot connect to sink database (postgresql)");
    tokio::spawn(async move {
        if let Err(e) = &connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    client
}

pub async fn query(
    db_client: &Client,
    query: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<Row>, Error> {
    db_client.query(query, params).await
}

pub async fn import_data(
    db_client: &mut Client,
    copy_statement: &str,
    data: &Vec<DataModel>,
    col_types: &Vec<Type>,
) -> Result<usize, Error> {
    let tx = db_client.transaction().await?;
    let sink = tx.copy_in(copy_statement).await?;
    let writer = BinaryCopyInWriter::new(sink, &col_types);

    pin_mut!(writer);
    let mut row: Vec<&'_ (dyn ToSql + Sync)> = Vec::new();
    for m in data {
        row.clear();
        row.push(&m.id);
        row.push(&m.name);
        // row.extend(m.metrics.iter().map(|x| x as &(dyn ToSql + Sync)));
        writer.as_mut().write(&row).await?;
    }

    let num_written:usize = writer.finish().await?.try_into().unwrap();
    tx.commit().await?;
    Ok(num_written)
}
