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
    permission: Option<super::add_access_key::CliAccessKeyPermission>,
}

#[derive(Debug)]
pub struct GenerateKeypair {
    pub permission: super::add_access_key::AccessKeyPermission,
}

impl GenerateKeypair {
    pub fn from(
        item: CliGenerateKeypair,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let permission: super::add_access_key::AccessKeyPermission = match item.permission {
            Some(cli_permission) => super::add_access_key::AccessKeyPermission::from(
                cli_permission,
                connection_config,
                sender_account_id,
            )?,
            None => super::add_access_key::AccessKeyPermission::choose_permission(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self { permission })
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
            &prepopulated_unsigned_transaction.signer_id,
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;

        match self.permission {
            super::add_access_key::AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        0,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                    )
                    .await
            }
            super::add_access_key::AccessKeyPermission::GrantFunctionCallAccess(
                function_call_type,
            ) => {
                function_call_type
                    .process(
                        0,
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                    )
                    .await
            }
        }
    }
}
