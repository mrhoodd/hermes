use abscissa_core::clap::Parser;
use abscissa_core::{Command, Runnable};

use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc_relayer::chain::counterparty::unreceived_acknowledgements;
use ibc_relayer::chain::handle::BaseChainHandle;

use crate::cli_utils::spawn_chain_counterparty;
use crate::conclude::Output;
use crate::error::Error;
use crate::prelude::*;

/// This command does the following:
/// 1. queries the chain to get its counterparty, channel and port identifiers (needed in 2)
/// 2. queries the chain for all packet commitments/ sequences for a given port and channel
/// 3. queries the counterparty chain for the unacknowledged sequences out of the list obtained in 2.
#[derive(Clone, Command, Debug, Parser, PartialEq)]
pub struct QueryUnreceivedAcknowledgementCmd {
    #[clap(
        long = "chain",
        required = true,
        value_name = "CHAIN_ID",
        help = "Identifier of the chain to query the unreceived acknowledgments"
    )]
    chain_id: ChainId,

    #[clap(
        long = "port",
        required = true,
        value_name = "PORT_ID",
        help = "Port identifier"
    )]
    port_id: PortId,

    #[clap(
        long = "channel",
        visible_alias = "chan",
        required = true,
        value_name = "CHANNEL_ID",
        help = "Channel identifier"
    )]
    channel_id: ChannelId,
}

impl QueryUnreceivedAcknowledgementCmd {
    fn execute(&self) -> Result<Vec<Sequence>, Error> {
        let config = app_config();
        debug!("Options: {:?}", self);

        let (chains, chan_conn_cli) = spawn_chain_counterparty::<BaseChainHandle>(
            &config,
            &self.chain_id,
            &self.port_id,
            &self.channel_id,
        )?;

        debug!(
            "fetched from source chain {} the following channel {:?}",
            self.chain_id, chan_conn_cli.channel,
        );

        unreceived_acknowledgements(&chains.src, &chains.dst, &(&chan_conn_cli.channel).into())
            .map(|(sns, _)| sns)
            .map_err(Error::supervisor)
    }
}

impl Runnable for QueryUnreceivedAcknowledgementCmd {
    fn run(&self) {
        match self.execute() {
            Ok(seqs) => Output::success(seqs).exit(),
            Err(e) => Output::error(format!("{}", e)).exit(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::QueryUnreceivedAcknowledgementCmd;

    use std::str::FromStr;

    use abscissa_core::clap::Parser;
    use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};

    #[test]
    fn test_query_packet_unreceived_acks() {
        assert_eq!(
            QueryUnreceivedAcknowledgementCmd {
                chain_id: ChainId::from_string("chain_id"),
                port_id: PortId::from_str("port_id").unwrap(),
                channel_id: ChannelId::from_str("channel-07").unwrap()
            },
            QueryUnreceivedAcknowledgementCmd::parse_from(&[
                "test",
                "--chain",
                "chain_id",
                "--port",
                "port_id",
                "--channel",
                "channel-07"
            ])
        )
    }

    #[test]
    fn test_query_packet_unreceived_acks_chan_alias() {
        assert_eq!(
            QueryUnreceivedAcknowledgementCmd {
                chain_id: ChainId::from_string("chain_id"),
                port_id: PortId::from_str("port_id").unwrap(),
                channel_id: ChannelId::from_str("channel-07").unwrap()
            },
            QueryUnreceivedAcknowledgementCmd::parse_from(&[
                "test",
                "--chain",
                "chain_id",
                "--port",
                "port_id",
                "--chan",
                "channel-07"
            ])
        )
    }

    #[test]
    fn test_query_packet_unreceived_acks_no_chan() {
        assert!(QueryUnreceivedAcknowledgementCmd::try_parse_from(&[
            "test", "--chain", "chain_id", "--port", "port_id"
        ])
        .is_err())
    }

    #[test]
    fn test_query_packet_unreceived_acks_no_port() {
        assert!(QueryUnreceivedAcknowledgementCmd::try_parse_from(&[
            "test",
            "--chain",
            "chain_id",
            "--channel",
            "channel-07"
        ])
        .is_err())
    }

    #[test]
    fn test_query_packet_unreceived_acks_no_chain() {
        assert!(QueryUnreceivedAcknowledgementCmd::try_parse_from(&[
            "test",
            "--port",
            "port_id",
            "--channel",
            "channel-07"
        ])
        .is_err())
    }
}
