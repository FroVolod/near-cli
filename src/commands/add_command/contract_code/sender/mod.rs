use dialoguer::Input;

/// данные об аккаунте контракта
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    contract: Option<super::contract::CliContract>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub contract: super::contract::Contract,
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
        let contract = match item.contract {
            Some(cli_contract) => super::contract::Contract::from(
                cli_contract,
                connection_config,
                sender_account_id.clone(),
            )?,
            None => super::contract::Contract::choose_contract(
                connection_config,
                sender_account_id.clone(),
            )?,
        };
        Ok(Self {
            sender_account_id,
            contract,
        })
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
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
            signer_id: self.sender_account_id.clone(),
            receiver_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.contract
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
