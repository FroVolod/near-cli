use std::str::FromStr;

/// Generate a key pair of secret and public keys (use it anywhere you need
/// Ed25519 keys)
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliGenerateKeypair {
    #[clap(subcommand)]
    pub deposit: Option<super::super::super::deposit::CliDeposit>,
}

#[derive(Debug)]
pub struct GenerateKeypair {
    pub deposit: super::super::super::deposit::Deposit,
}

impl GenerateKeypair {
    pub fn from(
        item: CliGenerateKeypair,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
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
        Ok(Self { deposit })
    }
}

impl GenerateKeypair {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        crate::common::save_access_key_to_keychain(
            network_connection_config.clone(),
            key_pair_properties.clone(),
            &prepopulated_unsigned_transaction.receiver_id,
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;

        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: 0,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
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
