use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Receiver(CliReceiver),
}

#[derive(Debug)]
pub enum SendTo {
    Receiver(Receiver),
}

impl SendTo {
    pub fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Receiver(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver, connection_config, sender_account_id)?;
                Ok(Self::Receiver(receiver))
            }
        }
    }
}

impl SendTo {
    pub fn send_to(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliSendTo::Receiver(Default::default()),
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::Receiver(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о получателе транзакции
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliReceiver {
    receiver_account_id: Option<String>,
    #[clap(subcommand)]
    transfer: Option<super::transfer_near_tokens_type::CliTransfer>,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver_account_id: String,
    pub transfer: super::transfer_near_tokens_type::Transfer,
}

impl Receiver {
    fn from(
        item: CliReceiver,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let receiver_account_id: String = match item.receiver_account_id {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id(),
        };
        let transfer: super::transfer_near_tokens_type::Transfer = match item.transfer {
            Some(cli_transfer) => super::transfer_near_tokens_type::Transfer::from(
                cli_transfer,
                connection_config,
                sender_account_id,
            )?,
            None => super::transfer_near_tokens_type::Transfer::choose_transfer_near(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            receiver_account_id,
            transfer,
        })
    }
}

impl Receiver {
    pub fn input_receiver_account_id() -> String {
        Input::new()
            .with_prompt("What is the account ID of the receiver?")
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
        self.transfer
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
