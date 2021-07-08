use dialoguer::Input;

/// вызов CallFunction
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCallFunctionAction {
    method_name: Option<String>,
    args: Option<String>,
    #[clap(long = "attached-deposit")]
    deposit: Option<crate::common::NearBalance>,
    #[clap(long = "prepaid-gas")]
    gas: Option<crate::common::NearGas>,
    #[clap(subcommand)]
    send_from: Option<super::sender::CliSendFrom>,
}

#[derive(Debug)]
pub struct CallFunctionAction {
    method_name: String,
    args: Vec<u8>,
    gas: near_primitives::types::Gas,
    deposit: near_primitives::types::Balance,
    send_from: super::sender::SendFrom,
}

impl CallFunctionAction {
    pub fn from(
        item: CliCallFunctionAction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let method_name: String = match item.method_name {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionAction::input_method_name(),
        };
        let args: Vec<u8> = match item.args {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionAction::input_args(),
        };
        let gas: near_primitives::types::Gas = match item.gas {
            Some(cli_gas) => match cli_gas {
                crate::common::NearGas { inner: num } => num,
            },
            None => CallFunctionAction::input_gas(),
        };
        let deposit: near_primitives::types::Balance = match item.deposit {
            Some(cli_deposit) => cli_deposit.to_yoctonear(),
            None => CallFunctionAction::input_deposit(),
        };
        let send_from = match item.send_from {
            Some(cli_send_from) => super::sender::SendFrom::from(cli_send_from, connection_config)?,
            None => super::sender::SendFrom::choose_send_from(connection_config)?,
        };
        Ok(Self {
            method_name,
            args,
            gas,
            deposit,
            send_from,
        })
    }
}

impl CallFunctionAction {
    fn input_method_name() -> String {
        println!();
        Input::new()
            .with_prompt("Enter a method name")
            .interact_text()
            .unwrap()
    }

    fn input_gas() -> near_primitives::types::Gas {
        println!();
        let gas: u64 = loop {
            let input_gas: crate::common::NearGas = Input::new()
                .with_prompt("Enter a gas for function")
                .with_initial_text("100 TeraGas")
                .interact_text()
                .unwrap();
            let gas: u64 = match input_gas {
                crate::common::NearGas { inner: num } => num,
            };
            if gas <= 300000000000000 {
                break gas;
            } else {
                println!("You need to enter a value of no more than 300 TERAGAS")
            }
        };
        gas
    }

    fn input_args() -> Vec<u8> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()
            .unwrap();
        input.into_bytes()
    }

    fn input_deposit() -> near_primitives::types::Balance {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()
            .unwrap();
        deposit.to_yoctonear()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.method_name.clone(),
                args: self.args.clone(),
                gas: self.gas.clone(),
                deposit: self.deposit.clone(),
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.send_from
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
