use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod access_key;
mod contract_code;
mod implicit_account;
mod stake_proposal;
mod sub_account;

/// инструмент выбора to add action
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAddAction {
    #[clap(subcommand)]
    action: Option<CliAction>,
}

#[derive(Debug)]
pub struct AddAction {
    pub action: Action,
}

impl AddAction {
    pub fn from(item: CliAddAction) -> color_eyre::eyre::Result<Self> {
        let action = match item.action {
            Some(cli_action) => Action::from(cli_action)?,
            None => Action::choose_action()?,
        };
        Ok(Self { action })
    }
}

impl AddAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.action.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliAction {
    /// Add a new contract code
    ContractCode(self::contract_code::operation_mode::CliOperationMode),
    /// Add an implicit-account
    ImplicitAccount(self::implicit_account::CliImplicitAccount),
    /// Add a new stake proposal
    StakeProposal(self::stake_proposal::operation_mode::CliOperationMode),
    /// Add a new sub-account
    SubAccount(self::sub_account::operation_mode::CliOperationMode),
    /// Add a new access key for an account
    AccessKey(self::access_key::operation_mode::CliOperationMode),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Action {
    #[strum_discriminants(strum(message = "Add a new access key for an account"))]
    AccessKey(self::access_key::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new contract code"))]
    ContractCode(self::contract_code::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add an implicit-account"))]
    ImplicitAccount(self::implicit_account::ImplicitAccount),
    #[strum_discriminants(strum(message = "Add a new stake proposal"))]
    StakeProposal(self::stake_proposal::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Add a new sub-account"))]
    SubAccount(self::sub_account::operation_mode::OperationMode),
}

impl Action {
    fn from(item: CliAction) -> color_eyre::eyre::Result<Self> {
        match item {
            CliAction::AccessKey(cli_operation_mode) => Ok(Action::AccessKey(
                self::access_key::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
            CliAction::ContractCode(cli_operation_mode) => Ok(Action::ContractCode(
                self::contract_code::operation_mode::OperationMode::from(cli_operation_mode)
                    .unwrap(),
            )),
            CliAction::ImplicitAccount(cli_generate_keypair) => {
                Ok(Action::ImplicitAccount(cli_generate_keypair.into()))
            }
            CliAction::StakeProposal(cli_operation_mode) => Ok(Action::StakeProposal(
                self::stake_proposal::operation_mode::OperationMode::from(cli_operation_mode)
                    .unwrap(),
            )),
            CliAction::SubAccount(cli_operation_mode) => Ok(Action::SubAccount(
                self::sub_account::operation_mode::OperationMode::from(cli_operation_mode).unwrap(),
            )),
        }
    }
}

impl Action {
    fn choose_action() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = ActionDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to add")
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            ActionDiscriminants::AccessKey => CliAction::AccessKey(Default::default()),
            ActionDiscriminants::ContractCode => CliAction::ContractCode(Default::default()),
            ActionDiscriminants::ImplicitAccount => CliAction::ImplicitAccount(Default::default()),
            ActionDiscriminants::StakeProposal => CliAction::StakeProposal(Default::default()),
            ActionDiscriminants::SubAccount => CliAction::SubAccount(Default::default()),
        };
        Ok(Self::from(cli_action)?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Action::AccessKey(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::ContractCode(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::ImplicitAccount(generate_keypair) => generate_keypair.process().await,
            Action::StakeProposal(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Action::SubAccount(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
