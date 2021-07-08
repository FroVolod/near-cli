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
    public_key: Option<super::CliDeleteAccessKeyAction>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub public_key: super::DeleteAccessKeyAction,
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
        let public_key = match item.public_key {
            Some(cli_delete_access_key) => super::DeleteAccessKeyAction::from(
                cli_delete_access_key,
                connection_config,
                sender_account_id.clone(),
            )?,
            None => super::DeleteAccessKeyAction::choose_delete_access_key_action(
                connection_config,
                sender_account_id.clone(),
            )?,
        };
        Ok(Self {
            sender_account_id,
            public_key,
        })
    }
}

impl Sender {
    fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Which account ID do you need to remove the key from?")
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
        self.public_key
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
