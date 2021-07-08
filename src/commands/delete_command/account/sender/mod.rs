use dialoguer::Input;

/// Specify the account to be deleted
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    send_to: Option<CliSendTo>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub send_to: SendTo,
}

impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let send_to: SendTo = match item.send_to {
            Some(cli_send_to) => {
                SendTo::from(cli_send_to, connection_config, sender_account_id.clone())?
            }
            None => SendTo::send_to(connection_config, sender_account_id.clone())?,
        };
        Ok(Self {
            sender_account_id,
            send_to,
        })
    }
}

impl Sender {
    fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Which account ID do you need to remove?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone(),
            receiver_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a beneficiary
    Beneficiary(super::CliDeleteAccountAction),
}

#[derive(Debug)]
pub enum SendTo {
    Beneficiary(super::DeleteAccountAction),
}

impl SendTo {
    fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Beneficiary(cli_delete_accaunt) => {
                let delete_accaunt = super::DeleteAccountAction::from(
                    cli_delete_accaunt,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(Self::Beneficiary(delete_accaunt))
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
            CliSendTo::Beneficiary(Default::default()),
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
            SendTo::Beneficiary(delete_account_action) => {
                delete_account_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
