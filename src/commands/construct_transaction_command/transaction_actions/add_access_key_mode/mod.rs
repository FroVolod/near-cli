use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod add_access_key;
mod generate_keypair;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAddAccessKeyMode {
    #[clap(subcommand)]
    public_key_mode: Option<CliPublicKeyMode>,
}

#[derive(Debug)]
pub struct AddAccessKeyMode {
    pub public_key_mode: PublicKeyMode,
}

impl AddAccessKeyMode {
    pub fn from(
        item: CliAddAccessKeyMode,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key_mode = match item.public_key_mode {
            Some(cli_public_key_mode) => {
                PublicKeyMode::from(cli_public_key_mode, connection_config, sender_account_id)?
            }
            None => PublicKeyMode::choose_public_key_mode(connection_config, sender_account_id)?,
        };
        Ok(Self { public_key_mode })
    }
}

impl AddAccessKeyMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.public_key_mode
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliPublicKeyMode {
    /// Enter public key
    PublicKey(self::add_access_key::CliAddAccessKeyAction),
    /// Generate key pair
    GenerateKeypair(self::generate_keypair::CliGenerateKeypair),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum PublicKeyMode {
    #[strum_discriminants(strum(message = "Enter public key"))]
    PublicKey(self::add_access_key::AddAccessKeyAction),
    #[strum_discriminants(strum(message = "Generate key pair"))]
    GenerateKeypair(self::generate_keypair::GenerateKeypair),
}

impl PublicKeyMode {
    fn from(
        item: CliPublicKeyMode,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliPublicKeyMode::PublicKey(cli_add_access_key_action) => Ok(PublicKeyMode::PublicKey(
                self::add_access_key::AddAccessKeyAction::from(
                    cli_add_access_key_action,
                    connection_config,
                    sender_account_id,
                )?,
            )),
            CliPublicKeyMode::GenerateKeypair(cli_generate_keypair) => Ok(
                PublicKeyMode::GenerateKeypair(self::generate_keypair::GenerateKeypair::from(
                    cli_generate_keypair,
                    connection_config,
                    sender_account_id,
                )?),
            ),
        }
    }
}

impl PublicKeyMode {
    fn choose_public_key_mode(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let variants = PublicKeyModeDiscriminants::iter().collect::<Vec<_>>();
        let modes = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a permission that you want to add to the access key:")
            .items(&modes)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_mode] {
            PublicKeyModeDiscriminants::PublicKey => Ok(Self::from(
                CliPublicKeyMode::PublicKey(Default::default()),
                connection_config,
                sender_account_id,
            )?),
            PublicKeyModeDiscriminants::GenerateKeypair => Ok(Self::from(
                CliPublicKeyMode::GenerateKeypair(Default::default()),
                connection_config,
                sender_account_id,
            )?),
        }
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            PublicKeyMode::PublicKey(add_access_key_action) => {
                add_access_key_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            PublicKeyMode::GenerateKeypair(cli_generate_keypair) => {
                cli_generate_keypair
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
