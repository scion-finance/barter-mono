use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Deserializer, Serialize};

/// Todo:
pub mod socket;
pub mod public;
pub mod util;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub kind: InstrumentKind,
    pub base: Symbol,
    pub quote: Symbol,
}

impl Instrument {
    pub fn new<S>(base: S, quote: S, kind: InstrumentKind) -> Self
    where
        S: Into<Symbol>
    {
        Self {
            kind,
            base: base.into(),
            quote: quote.into(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum InstrumentKind {
    Spot,
    Future,
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Symbol(pub String);

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        String::deserialize(deserializer).map(Symbol::new)
    }
}

impl<S> From<S> for Symbol where S: Into<String> {
    fn from(input: S) -> Self {
        Self(input.into().to_lowercase())
    }
}

impl Symbol {
    pub fn new<S>(input: S) -> Self where S: Into<Symbol> {
        input.into()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use futures::StreamExt;
    use crate::public::MarketStream;
    use crate::public::binance::futures::{BinanceFuturesItem, BinanceFuturesStream};
    use crate::public::explore::{Exchange, StreamBuilder, StreamConfig, StreamKind};
    use crate::public::model::{MarketEvent, Subscription};
    use super::*;

    // Todo: Find a way to remove SocketItem... it surely isn't needed.

    // Todo: Is it possible to pair the Socket & ProtocolParser generics eg/ 'SocketParser'

    // Todo: Maybe OutputIter will become an Option<OutputIter>?

    // Todo: Add proper error enum for BinanceMessage in Barter-Data
    //     '--> eg/ enum BinanceMessage { Error, BinancePayload }

    // Todo: Do I want to keep the name trait Exchange? Do I like the generic ExTransformer, etc.

    async fn run<S, OutputIter>(subscriptions: &[Subscription])
    where
        S: MarketStream<OutputIter>,
        OutputIter: IntoIterator<Item = MarketEvent>,
        <<OutputIter as IntoIterator>::IntoIter as Iterator>::Item: Debug,
    {
        let mut stream = S::init(subscriptions)
            .await
            .expect("failed to init stream");

        while let Some(result) = stream.next().await {
            match result {
                Ok(market_data) => {
                    market_data
                        .into_iter()
                        .for_each(|event| {
                            println!("{:?}", event);
                        })
                }
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            }
        }
    }

    #[tokio::test]
    async fn it_works() {
        let subscriptions = [
            Subscription::Trades(Instrument::new(
                "btc", "usdt", InstrumentKind::Future)
            ),
            Subscription::Trades(Instrument::new(
                "eth", "usdt", InstrumentKind::Future)
            ),
        ];

        run::<BinanceFuturesStream, BinanceFuturesItem>(&subscriptions).await;
    }

    #[tokio::test]
    async fn stream_builder_works() {
        let streams = vec![
            StreamConfig {
                instrument: Instrument::new("btc", "usdt", InstrumentKind::Future),
                kind: StreamKind::Trade
            }
        ];


        let mut binance_rx = StreamBuilder::new()
            .add(Exchange::BinanceFutures, streams)
            .build()
            .await.unwrap()
            .remove(&Exchange::BinanceFutures).unwrap();

        while let Some(event) = binance_rx.recv().await {
            println!("{:?}", event)
        }
    }
}