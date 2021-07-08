use async_recursion::async_recursion;
use dialoguer::Input;

/// создание ставки
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliStakeNEARTokensAction {
    stake: Option<crate::common::NearBalance>,
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug)]
pub struct StakeNEARTokensAction {
    pub stake: crate::common::NearBalance,
    pub public_key: near_crypto::PublicKey,
    pub next_action: Box<super::NextAction>,
}

impl StakeNEARTokensAction {
    pub fn from(
        item: CliStakeNEARTokensAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: String,
    ) -> color_eyre::eyre::Result<Self> {
        let stake: crate::common::NearBalance = match item.stake {
            Some(cli_stake) => cli_stake,
            None => StakeNEARTokensAction::input_stake(),
        };
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => StakeNEARTokensAction::input_public_key(),
        };
        let skip_next_action: super::NextAction = match item.next_action {
            Some(cli_skip_action) => super::NextAction::from_cli_skip_next_action(
                cli_skip_action,
                connection_config,
                sender_account_id,
            )?,
            None => super::NextAction::input_next_action(connection_config, sender_account_id)?,
        };
        Ok(Self {
            stake,
            public_key,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl StakeNEARTokensAction {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this stake")
            .interact_text()
            .unwrap()
    }

    fn input_stake() -> crate::common::NearBalance {
        Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake?")
            .interact_text()
            .unwrap()
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        mut prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake: self.stake.to_yoctonear(),
                public_key: self.public_key.clone(),
            },
        );
        prepopulated_unsigned_transaction.actions.push(action);
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
