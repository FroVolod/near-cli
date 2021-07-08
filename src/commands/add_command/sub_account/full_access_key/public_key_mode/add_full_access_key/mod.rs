use dialoguer::Input;

/// Add full access key to the sub-account
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    nonce: Option<u64>,
    #[clap(subcommand)]
    deposit: Option<super::super::super::deposit::CliDeposit>,
}

#[derive(Debug)]
pub struct AddAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub nonce: near_primitives::types::Nonce,
    pub deposit: super::super::super::deposit::Deposit,
}

impl AddAccessKeyAction {
    pub fn from(
        item: CliAddAccessKeyAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => AddAccessKeyAction::input_public_key(),
        };
        let deposit = match item.deposit {
            Some(cli_deposit) => super::super::super::deposit::Deposit::from(
                cli_deposit,
                connection_config,
                sender_account_id,
            )?,
            None => super::super::super::deposit::Deposit::choose_deposit(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            public_key,
            nonce: 0,
            deposit,
        })
    }
}

impl AddAccessKeyAction {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: self.nonce.clone(),
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: self.public_key.clone(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.deposit
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
