//! An asynchronous, pipelined, PostgreSQL client.
//!
//! # Example
//!
//! ```no_run
//! use tokio_postgres::{NoTls, Error};
//!
//! # #[cfg(not(feature = "runtime"))] fn main() {}
//! # #[cfg(feature = "runtime")]
//! #[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
//! async fn main() -> Result<(), Error> {
//!     // Connect to the database.
//!     let (client, connection) =
//!         tokio_postgres::connect("host=localhost user=postgres", NoTls).await?;
//!
//!     // The connection object performs the actual communication with the database,
//!     // so spawn it off to run on its own.
//!     tokio::spawn(async move {
//!         if let Err(e) = connection.await {
//!             eprintln!("connection error: {}", e);
//!         }
//!     });
//!
//!     // Now we can execute a simple statement that just returns its parameter.
//!     let rows = client
//!         .query("SELECT $1::TEXT", &[&"hello world"])
//!         .await?;
//!
//!     // And then check that we got back the same string we sent over.
//!     let value: &str = rows[0].get(0);
//!     assert_eq!(value, "hello world");
//!
//!     Ok(())
//! }
//! ```
//!
//! # Behavior
//!
//! Calling a method like `Client::query` on its own does nothing. The associated request is not sent to the database
//! until the future returned by the method is first polled. Requests are executed in the order that they are first
//! polled, not in the order that their futures are created.
//!
//! # Pipelining
//!
//! The client supports *pipelined* requests. Pipelining can improve performance in use cases in which multiple,
//! independent queries need to be executed. In a traditional workflow, each query is sent to the server after the
//! previous query completes. In contrast, pipelining allows the client to send all of the queries to the server up
//! front, minimizing time spent by one side waiting for the other to finish sending data:
//!
//! ```not_rust
//!             Sequential                              Pipelined
//! | Client         | Server          |    | Client         | Server          |
//! |----------------|-----------------|    |----------------|-----------------|
//! | send query 1   |                 |    | send query 1   |                 |
//! |                | process query 1 |    | send query 2   | process query 1 |
//! | receive rows 1 |                 |    | send query 3   | process query 2 |
//! | send query 2   |                 |    | receive rows 1 | process query 3 |
//! |                | process query 2 |    | receive rows 2 |                 |
//! | receive rows 2 |                 |    | receive rows 3 |                 |
//! | send query 3   |                 |
//! |                | process query 3 |
//! | receive rows 3 |                 |
//! ```
//!
//! In both cases, the PostgreSQL server is executing the queries sequentially - pipelining just allows both sides of
//! the connection to work concurrently when possible.
//!
//! Pipelining happens automatically when futures are polled concurrently (for example, by using the futures `join`
//! combinator):
//!
//! ```rust
//! use futures_util::future;
//! use std::future::Future;
//! use tokio_postgres::{Client, Error, Statement};
//!
//! async fn pipelined_prepare(
//!     client: &Client,
//! ) -> Result<(Statement, Statement), Error>
//! {
//!     future::try_join(
//!         client.prepare("SELECT * FROM foo"),
//!         client.prepare("INSERT INTO bar (id, name) VALUES ($1, $2)")
//!     ).await
//! }
//! ```
//!
//! # Runtime
//!
//! The client works with arbitrary `AsyncRead + AsyncWrite` streams. Convenience APIs are provided to handle the
//! connection process, but these are gated by the `runtime` Cargo feature, which is enabled by default. If disabled,
//! all dependence on the tokio runtime is removed.
//!
//! # SSL/TLS support
//!
//! TLS support is implemented via external libraries. `Client::connect` and `Config::connect` take a TLS implementation
//! as an argument. The `NoTls` type in this crate can be used when TLS is not required. Otherwise, the
//! `postgres-openssl` and `postgres-native-tls` crates provide implementations backed by the `openssl` and `native-tls`
//! crates, respectively.
//!
//! # Features
//!
//! The following features can be enabled from `Cargo.toml`:
//!
//! | Feature | Description | Extra dependencies | Default |
//! | ------- | ----------- | ------------------ | ------- |
//! | `runtime` | Enable convenience API for the connection process based on the `tokio` crate. | [tokio](https://crates.io/crates/tokio) 1.0 with the features `net` and `time` | yes |
//! | `array-impls` | Enables `ToSql` and `FromSql` trait impls for arrays | - | no |
//! | `with-bit-vec-0_6` | Enable support for the `bit-vec` crate. | [bit-vec](https://crates.io/crates/bit-vec) 0.6 | no |
//! | `with-chrono-0_4` | Enable support for the `chrono` crate. | [chrono](https://crates.io/crates/chrono) 0.4 | no |
//! | `with-eui48-0_4` | Enable support for the 0.4 version of the `eui48` crate. | [eui48](https://crates.io/crates/eui48) 0.4 | no |
//! | `with-eui48-1` | Enable support for the 1.0 version of the `eui48` crate. | [eui48](https://crates.io/crates/eui48) 1.0 | no |
//! | `with-geo-types-0_6` | Enable support for the 0.6 version of the `geo-types` crate. | [geo-types](https://crates.io/crates/geo-types/0.6.0) 0.6 | no |
//! | `with-geo-types-0_7` | Enable support for the 0.7 version of the `geo-types` crate. | [geo-types](https://crates.io/crates/geo-types/0.7.0) 0.7 | no |
//! | `with-serde_json-1` | Enable support for the `serde_json` crate. | [serde_json](https://crates.io/crates/serde_json) 1.0 | no |
//! | `with-uuid-0_8` | Enable support for the `uuid` crate. | [uuid](https://crates.io/crates/uuid) 0.8 | no |
//! | `with-uuid-1` | Enable support for the `uuid` crate. | [uuid](https://crates.io/crates/uuid) 1.0 | no |
//! | `with-time-0_2` | Enable support for the 0.2 version of the `time` crate. | [time](https://crates.io/crates/time/0.2.0) 0.2 | no |
//! | `with-time-0_3` | Enable support for the 0.3 version of the `time` crate. | [time](https://crates.io/crates/time/0.3.0) 0.3 | no |
#![warn(rust_2018_idioms, clippy::all)]
use std::sync::Arc;

pub use crate::cancel_token::CancelToken;
pub use crate::client::Client;
pub use crate::config::Config;
pub use crate::connection::Connection;
pub use crate::copy_in::CopyInSink;
pub use crate::copy_out::CopyOutStream;
use crate::error::DbError;
pub use crate::error::Error;
pub use crate::generic_client::GenericClient;
pub use crate::generic_result::GenericResult;
pub use crate::portal::Portal;
pub use crate::query::{ResultStream, RowStream};
pub use crate::row::{Row, SimpleQueryRow};
pub use crate::simple_query::SimpleQueryStream;
#[cfg(feature = "runtime")]
pub use crate::socket::Socket;
pub use crate::statement::{Column, Statement};
#[cfg(feature = "runtime")]
use crate::tls::MakeTlsConnect;
pub use crate::tls::NoTls;
pub use crate::to_statement::ToStatement;
pub use crate::transaction::Transaction;
pub use crate::transaction_builder::{IsolationLevel, TransactionBuilder};
use crate::types::ToSql;
use bytes::Bytes;
pub use postgres_protocol::message::backend::{Message, OwnedField};

pub mod binary_copy;
mod bind;
#[cfg(feature = "runtime")]
mod cancel_query;
mod cancel_query_raw;
mod cancel_token;
pub mod client;
pub mod codec;
pub mod config;
#[cfg(feature = "runtime")]
mod connect;
mod connect_raw;
#[cfg(feature = "runtime")]
mod connect_socket;
mod connect_tls;
pub mod connection;
mod copy_in;
mod copy_out;
pub mod error;
mod generic_client;
pub mod generic_result;
#[cfg(not(target_arch = "wasm32"))]
mod keepalive;
mod maybe_tls_stream;
mod portal;
mod prepare;
mod query;
pub mod row;
pub mod simple_query;
#[cfg(feature = "runtime")]
mod socket;
mod statement;
pub mod tls;
mod to_statement;
mod transaction;
mod transaction_builder;
pub mod types;

/// This crate was originally written to use binary result format for pretty much everything, and
/// it's still the case in most parts of the code that we hardcode to use binary. There are some
/// small use cases we've hacked in where you can specify different result formats, though.
/// Since Postgres lets you specify different result formats for different columns, we generally
/// accept an IntoIterator<Item = i16> since each result format is specified as an i16 value of 0
/// for text or 1 for binary. However, if you only specify a single format, Postgres interprets
/// that as "use this format for everything", hence Some(1) being the default here:
pub(crate) const DEFAULT_RESULT_FORMATS: Option<i16> = Some(1);

/// A convenience function which parses a connection string and connects to the database.
///
/// See the documentation for [`Config`] for details on the connection string format.
///
/// Requires the `runtime` Cargo feature (enabled by default).
///
/// [`Config`]: config/struct.Config.html
#[cfg(feature = "runtime")]
pub async fn connect<T>(
    config: &str,
    tls: T,
) -> Result<(Client, Connection<Socket, T::Stream>), Error>
where
    T: MakeTlsConnect<Socket>,
{
    let config = config.parse::<Config>()?;
    config.connect(tls).await
}

/// An asynchronous notification.
#[derive(Clone, Debug)]
pub struct Notification {
    process_id: i32,
    channel: String,
    payload: String,
}

impl Notification {
    /// The process ID of the notifying backend process.
    pub fn process_id(&self) -> i32 {
        self.process_id
    }

    /// The name of the channel that the notify has been raised on.
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// The "payload" string passed from the notifying process.
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// An asynchronous message from the server.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AsyncMessage {
    /// A notice.
    ///
    /// Notices use the same format as errors, but aren't "errors" per-se.
    Notice(DbError),
    /// A notification.
    ///
    /// Connections can subscribe to notifications with the `LISTEN` command.
    Notification(Notification),
}

/// Message returned by the `SimpleQuery` stream.
#[non_exhaustive]
#[derive(Debug)]
pub enum SimpleQueryMessage {
    /// A row of data.
    Row(SimpleQueryRow),
    /// A statement in the query has completed.
    ///
    /// The full tag, along with the rows modified or selected are returned.
    CommandComplete(CommandCompleteContents),
}

#[derive(Eq, PartialEq, Debug)]
pub struct CommandCompleteContents {
    /// Fields, if relevant for the CommandCompleteContents.
    pub fields: Option<Arc<[OwnedField]>>,
    /// The number of rows modified or selected.
    pub rows: u64,
    /// The full tag in bytes of the CommandComplete body.
    pub tag: Bytes,
}

fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
