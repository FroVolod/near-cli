use dialoguer::{theme::ColorfulTheme, Input, Select};
use near_primitives::borsh::BorshSerialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

/// Sign constructed transaction with Ledger
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSignLedger {
    #[clap(long)]
    seed_phrase_hd_path: Option<slip10::BIP32Path>,
    #[clap(long)]
    nonce: Option<u64>,
    #[clap(long)]
    block_hash: Option<near_primitives::hash::CryptoHash>,
    #[clap(subcommand)]
    submit: Option<Submit>,
}

#[derive(Debug)]
pub struct SignLedger {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub signer_public_key: near_crypto::PublicKey,
    nonce: u64,
    block_hash: near_primitives::hash::CryptoHash,
    pub submit: Option<Submit>,
}

impl SignLedger {
    pub fn from(
        item: CliSignLedger,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path = match item.seed_phrase_hd_path {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = actix::System::new()
            .block_on(async { near_ledger::get_public_key(seed_phrase_hd_path.clone()).await })
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            })?;
        let signer_public_key = near_crypto::PublicKey::ED25519(
            near_crypto::ED25519PublicKey::from(public_key.to_bytes()),
        );
        let submit: Option<Submit> = item.submit;
        match connection_config {
            Some(_) => Ok(Self {
                seed_phrase_hd_path,
                signer_public_key,
                nonce: 0,
                block_hash: Default::default(),
                submit,
            }),
            None => {
                let nonce: u64 = match item.nonce {
                    Some(cli_nonce) => cli_nonce,
                    None => super::input_access_key_nonce(&signer_public_key.to_string().clone()),
                };
                let block_hash = match item.block_hash {
                    Some(cli_block_hash) => cli_block_hash,
                    None => super::input_block_hash(),
                };
                Ok(Self {
                    seed_phrase_hd_path,
                    signer_public_key,
                    nonce,
                    block_hash,
                    submit,
                })
            }
        }
    }
}

impl SignLedger {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub fn input_seed_phrase_hd_path() -> slip10::BIP32Path {
        Input::new()
            .with_prompt("Enter seed phrase HD Path (if you not sure leave blank for default)")
            .with_initial_text("44'/397'/0'/0'/1'")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let seed_phrase_hd_path = self.seed_phrase_hd_path.clone();
        let public_key = self.signer_public_key.clone();
        let nonce = self.nonce.clone();
        let block_hash = self.block_hash.clone();
        let submit: Option<Submit> = self.submit.clone();
        match network_connection_config {
            None => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    nonce,
                    block_hash,
                    ..prepopulated_unsigned_transaction
                };
                println!(
                    "{:#?}\n Confirm transaction signing on your Ledger device (HD Path: {})",
                    unsigned_transaction, seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!(
                    "\n\n---  Signed transaction:   ---\n    {:#?}",
                    &signed_transaction
                );
                match submit {
                    Some(submit) => submit.process_offline(signed_transaction, serialize_to_base64),
                    None => {
                        let submit = Submit::choose_submit();
                        submit.process_offline(signed_transaction, serialize_to_base64)
                    }
                }
            }
            Some(network_connection_config) => {
                let online_signer_access_key_response = self
                    .rpc_client(network_connection_config.rpc_url().as_str())
                    .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                        block_reference: near_primitives::types::Finality::Final.into(),
                        request: near_primitives::views::QueryRequest::ViewAccessKey {
                            account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                            public_key: public_key.clone(),
                        },
                    })
                    .await
                    .map_err(|err| {
                        color_eyre::Report::msg(format!(
                            "Failed to fetch public key information for nonce: {:?}",
                            err
                        ))
                    })?;
                let current_nonce =
                    if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                        online_signer_access_key,
                    ) = online_signer_access_key_response.kind
                    {
                        online_signer_access_key.nonce
                    } else {
                        return Err(color_eyre::Report::msg(format!("Error current_nonce")));
                    };
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    block_hash: online_signer_access_key_response.block_hash,
                    nonce: current_nonce + 1,
                    ..prepopulated_unsigned_transaction
                };

                println!(
                    "{:#?}\n Confirm transaction signing on your Ledger device (HD Path: {})",
                    unsigned_transaction, seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!(
                    "\n\n---  Signed transaction:   ---\n    {:#?}",
                    &signed_transaction
                );
                match submit {
                    None => {
                        let submit = Submit::choose_submit();
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                    Some(submit) => {
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                }
            }
        }
    }
}

#[derive(Debug, EnumDiscriminants, Clone, clap::Clap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(
        message = "Do you want send the transaction to the server (it's works only for online mode)"
    ))]
    Send,
    #[strum_discriminants(strum(message = "Do you want show the transaction on display?"))]
    Display,
}

impl Submit {
    pub fn choose_submit() -> Self {
        println!();
        let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();
        let submits = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_submit = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&submits)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_submit] {
            SubmitDiscriminants::Send => Submit::Send,
            SubmitDiscriminants::Display => Submit::Display,
        }
    }

    pub fn process_offline(
        self,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        println!("\n\n\n===========  DISPLAY  ==========");
        println!(
            "\n\n---  Signed transaction:   ---\n    {:#?}",
            &signed_transaction
        );
        println!(
            "\n\n---  serialize_to_base64:   --- \n   {}",
            &serialize_to_base64
        );
        Ok(None)
    }

    pub async fn process_online(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            Submit::Send => {
                println!("Transaction sent ...");
                let json_rcp_client =
                    near_jsonrpc_client::new_client(network_connection_config.rpc_url().as_str());
                let transaction_info = loop {
                    let transaction_info_result = json_rcp_client
                        .broadcast_tx_commit(near_primitives::serialize::to_base64(
                            signed_transaction
                                .try_to_vec()
                                .expect("Transaction is not expected to fail on serialization"),
                        ))
                        .await;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => {
                            if let Some(serde_json::Value::String(data)) = &err.data {
                                if data.contains("Timeout") {
                                    println!("Error transaction: {:?}", err);
                                    continue;
                                }
                            }
                            return Err(color_eyre::Report::msg(format!(
                                "Error transaction: {:?}",
                                err
                            )));
                        }
                    };
                };
                Ok(Some(transaction_info))
            }
            Submit::Display => {
                println!("\n\n\n===========  DISPLAY  ==========");
                println!(
                    "\n\n---  Signed transaction:   ---\n {:#?}",
                    &signed_transaction
                );
                println!(
                    "\n\n---  serialize_to_base64:   --- \n {}",
                    &serialize_to_base64
                );
                Ok(None)
            }
        }
    }
}
