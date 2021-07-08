use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliTransfer {
    /// Enter an amount
    Amount(CliTransferNEARTokensAction),
}

#[derive(Debug)]
pub enum Transfer {
    Amount(TransferNEARTokensAction),
}

impl Transfer {
    pub fn from(
        item: CliTransfer,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliTransfer::Amount(cli_transfer_near_action) => {
                Ok(Self::Amount(TransferNEARTokensAction::from(
                    cli_transfer_near_action,
                    connection_config,
                    sender_account_id,
                )?))
            }
        }
    }
}

impl Transfer {
    pub fn choose_transfer_near(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliTransfer::Amount(Default::default()),
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
            Transfer::Amount(transfer_near_action) => {
                transfer_near_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// создание перевода токенов
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliTransferNEARTokensAction {
    amount: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    sign_transactions: Option<super::transactions_signing::CliTransactionsSigning>,
}

#[derive(Debug)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    pub sign_transactions: super::transactions_signing::TransactionsSigning,
}

impl TransferNEARTokensAction {
    fn from(
        item: CliTransferNEARTokensAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let amount: crate::common::NearBalance = match item.amount {
            Some(cli_amount) => cli_amount,
            None => TransferNEARTokensAction::input_amount(),
        };
        let sign_transactions = match item.sign_transactions {
            Some(cli_sign_transaction) => super::transactions_signing::TransactionsSigning::from(
                cli_sign_transaction,
                connection_config,
                sender_account_id,
            )?,
            None => super::transactions_signing::TransactionsSigning::choose_sign_transactions(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            amount,
            sign_transactions,
        })
    }
}

impl TransferNEARTokensAction {
    fn input_amount() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.sign_transactions
            .process(
                prepopulated_unsigned_transaction,
                network_connection_config,
                self.amount.to_yoctonear(),
            )
            .await
    }
}
