// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Runs the relay: `bindle-relay [--db relay.db]`. It listens on the port in
//! `PORT` (8080 when unset), as most hosting services expect.

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db = args
        .windows(2)
        .find(|w| w[0] == "--db")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "relay.db".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let conn = rusqlite::Connection::open(&db).map_err(std::io::Error::other)?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("Bindle relay on port {port}, keeping sealed data in {db}");
    bindle_relay::serve(listener, conn).await
}
