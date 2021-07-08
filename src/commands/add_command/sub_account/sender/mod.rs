use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub owner_account_id: Option<String>,
    #[clap(subcommand)]
    send_to: Option<super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct Sender {
    pub owner_account_id: String,
    pub send_to: super::receiver::SendTo,
}

impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let owner_account_id: String = match item.owner_account_id {
            Some(cli_owner_account_id) => cli_owner_account_id,
            None => Sender::input_owner_account_id(),
        };
        let send_to: super::receiver::SendTo = match item.send_to {
            Some(cli_send_to) => super::receiver::SendTo::from(
                cli_send_to,
                connection_config,
                owner_account_id.clone(),
            )?,
            None => super::receiver::SendTo::send_to(connection_config, owner_account_id.clone())?,
        };
        Ok(Self {
            owner_account_id,
            send_to,
        })
    }
}

impl Sender {
    fn input_owner_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the owner account ID?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.owner_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
