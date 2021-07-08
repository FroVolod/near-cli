use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod operation_mode;
mod receiver;
mod sender;
pub mod transfer_near_tokens_type;

/// инструмент выбора переводимой валюты
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCurrency {
    #[clap(subcommand)]
    currency_selection: Option<CliCurrencySelection>,
}

#[derive(Debug)]
pub struct Currency {
    currency_selection: CurrencySelection,
}

impl Currency {
    pub fn from(item: CliCurrency) -> color_eyre::eyre::Result<Self> {
        let currency_selection = match item.currency_selection {
            Some(cli_currency_selection) => CurrencySelection::from(cli_currency_selection)?,
            None => CurrencySelection::choose_currency()?,
        };
        Ok(Self { currency_selection })
    }
}

impl Currency {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.currency_selection
            .process(prepopulated_unsigned_transaction)
            .await
    }
}

#[derive(Debug, clap::Clap)]
enum CliCurrencySelection {
    /// отправка трансфера в NEAR tokens
    NEAR(self::operation_mode::CliOperationMode),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
enum CurrencySelection {
    #[strum_discriminants(strum(message = "NEAR tokens"))]
    NEAR(self::operation_mode::OperationMode),
}

impl CurrencySelection {
    fn from(item: CliCurrencySelection) -> color_eyre::eyre::Result<Self> {
        match item {
            CliCurrencySelection::NEAR(cli_operation_mode) => Ok(Self::NEAR(
                self::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
        }
    }
}

impl CurrencySelection {
    fn choose_currency() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = CurrencySelectionDiscriminants::iter().collect::<Vec<_>>();
        let currencies = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_currency = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What do you want to transfer?")
            .items(&currencies)
            .default(0)
            .interact()
            .unwrap();
        let cli_currency = match variants[selected_currency] {
            CurrencySelectionDiscriminants::NEAR => CliCurrencySelection::NEAR(Default::default()),
        };
        Ok(Self::from(cli_currency)?)
    }

    async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::NEAR(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
