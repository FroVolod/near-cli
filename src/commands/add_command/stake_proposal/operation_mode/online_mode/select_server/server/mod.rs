use dialoguer::Input;

/// предустановленный RPC-сервер
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliServer {
    #[clap(subcommand)]
    pub send_from: Option<CliSendFrom>,
}

/// данные для custom server
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCustomServer {
    #[clap(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[clap(subcommand)]
    send_from: Option<CliSendFrom>,
}

#[derive(Debug)]
pub struct Server {
    pub connection_config: Option<crate::common::ConnectionConfig>,
    pub send_from: SendFrom,
}

impl CliServer {
    pub fn into_server(
        self,
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Server> {
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from, Some(connection_config.clone()))?,
            None => SendFrom::choose_send_from(Some(connection_config.clone()))?,
        };
        Ok(Server {
            connection_config: Some(connection_config),
            send_from,
        })
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> color_eyre::eyre::Result<Server> {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoint?")
                .interact_text()
                .unwrap(),
        };
        let connection_config = Some(crate::common::ConnectionConfig::Custom { url: url.inner });
        let send_from = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from, connection_config.clone())?,
            None => SendFrom::choose_send_from(connection_config.clone())?,
        };
        Ok(Server {
            connection_config,
            send_from,
        })
    }
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.send_from
            .process(prepopulated_unsigned_transaction, self.connection_config)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendFrom {
    /// Specify a validator
    Validator(super::super::super::super::sender::CliSender),
}

#[derive(Debug)]
pub enum SendFrom {
    Validator(super::super::super::super::sender::Sender),
}

impl SendFrom {
    pub fn from(
        item: CliSendFrom,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendFrom::Validator(cli_sender) => Ok(Self::Validator(
                super::super::super::super::sender::Sender::from(cli_sender, connection_config)?,
            )),
        }
    }
}

impl SendFrom {
    pub fn choose_send_from(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliSendFrom::Validator(Default::default()),
            connection_config,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendFrom::Validator(sender) => {
                sender
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
