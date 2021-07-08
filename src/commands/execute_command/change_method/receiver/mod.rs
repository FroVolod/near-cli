use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Contract(CliReceiver),
}

#[derive(Debug)]
pub enum SendTo {
    Contract(Receiver),
}

impl SendTo {
    pub fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Contract(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver, connection_config)?;
                Ok(Self::Contract(receiver))
            }
        }
    }
}

impl SendTo {
    pub fn send_to(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliSendTo::Contract(Default::default()),
            connection_config,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о контракте
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliReceiver {
    receiver_account_id: Option<String>,
    #[clap(subcommand)]
    call: Option<super::CliCallFunction>,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver_account_id: String,
    pub call: super::CallFunction,
}

impl Receiver {
    fn from(
        item: CliReceiver,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let receiver_account_id: String = match item.receiver_account_id {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id(),
        };
        let call = match item.call {
            Some(cli_call) => super::CallFunction::from(cli_call, connection_config)?,
            None => super::CallFunction::choose_call_function(connection_config)?,
        };
        Ok(Self {
            receiver_account_id,
            call,
        })
    }
}

impl Receiver {
    fn input_receiver_account_id() -> String {
        Input::new()
            .with_prompt("What is the account ID of the contract?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.call
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
