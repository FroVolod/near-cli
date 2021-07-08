use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod sign_manually;
pub mod sign_with_keychain;
pub mod sign_with_ledger;
pub mod sign_with_private_key;

#[derive(Debug, clap::Clap)]
pub enum CliSignTransaction {
    /// Provide arguments to sign a private key transaction
    SignPrivateKey(self::sign_with_private_key::CliSignPrivateKey),
    /// Provide arguments to sign a keychain transaction
    SignWithKeychain(self::sign_with_keychain::CliSignKeychain),
    /// Connect your Ledger device and sign transaction with it
    SignWithLedger(self::sign_with_ledger::CliSignLedger),
    /// Provide arguments to sign a manually transaction
    SignManually(self::sign_manually::CliSignManually),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SignTransaction {
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with my private key"
    ))]
    SignPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(message = "Yes, I want to sign the transaction with keychain"))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with Ledger device"
    ))]
    SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "No, I want to construct the transaction and sign it somewhere else"
    ))]
    SignManually(self::sign_manually::SignManually),
}

impl SignTransaction {
    pub fn from(
        item: CliSignTransaction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSignTransaction::SignPrivateKey(cli_private_key) => {
                let private_key = self::sign_with_private_key::SignPrivateKey::from(
                    cli_private_key,
                    connection_config,
                );
                Ok(SignTransaction::SignPrivateKey(private_key))
            }
            CliSignTransaction::SignWithKeychain(cli_key_chain) => {
                let key_chain = self::sign_with_keychain::SignKeychain::from(
                    cli_key_chain,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(SignTransaction::SignWithKeychain(key_chain))
            }
            CliSignTransaction::SignWithLedger(cli_ledger) => {
                let ledger =
                    self::sign_with_ledger::SignLedger::from(cli_ledger, connection_config)?;
                Ok(SignTransaction::SignWithLedger(ledger))
            }
            CliSignTransaction::SignManually(cli_manually) => {
                let manually =
                    self::sign_manually::SignManually::from(cli_manually, connection_config);
                Ok(SignTransaction::SignManually(manually))
            }
        }
    }
}

impl SignTransaction {
    pub fn choose_sign_option(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = SignTransactionDiscriminants::iter().collect::<Vec<_>>();
        let sign_options = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_sign_options = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to sign the transaction?")
            .items(&sign_options)
            .default(0)
            .interact()
            .unwrap();
        let cli_sign_option = match variants[select_sign_options] {
            SignTransactionDiscriminants::SignPrivateKey => {
                CliSignTransaction::SignPrivateKey(Default::default())
            }
            SignTransactionDiscriminants::SignWithKeychain => {
                CliSignTransaction::SignWithKeychain(Default::default())
            }
            SignTransactionDiscriminants::SignWithLedger => {
                CliSignTransaction::SignWithLedger(Default::default())
            }
            SignTransactionDiscriminants::SignManually => {
                CliSignTransaction::SignManually(Default::default())
            }
        };
        Self::from(cli_sign_option, connection_config, sender_account_id)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            SignTransaction::SignPrivateKey(keys) => {
                keys.process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignWithKeychain(chain) => {
                chain
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignWithLedger(ledger) => {
                ledger
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignManually(args_manually) => {
                args_manually
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

fn input_signer_public_key() -> near_crypto::PublicKey {
    Input::new()
        .with_prompt("To create an unsigned transaction enter sender's public key")
        .interact_text()
        .unwrap()
}

fn input_signer_secret_key() -> near_crypto::SecretKey {
    Input::new()
        .with_prompt("Enter sender's private key")
        .interact_text()
        .unwrap()
}

fn input_access_key_nonce(public_key: &str) -> u64 {
    println!("Your public key: `{}`", public_key);
    Input::new()
        .with_prompt(
            "Enter transaction nonce for this public key (query the access key information with \
            `./near-cli view nonce \
                network testnet \
                account 'volodymyr.testnet' \
                public-key ed25519:...` incremented by 1)",
        )
        .interact_text()
        .unwrap()
}

fn input_block_hash() -> near_primitives::hash::CryptoHash {
    let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
        .with_prompt(
            "Enter recent block hash (query information about the hash of the last block with \
            `./near-cli view recent-block-hash network testnet`)",
        )
        .interact_text()
        .unwrap();
    input_block_hash.inner
}
