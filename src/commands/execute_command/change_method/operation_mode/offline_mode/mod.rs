/// аргументы, необходимые для offline mode
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOfflineArgs {
    #[clap(subcommand)]
    pub send_to: Option<super::super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct OfflineArgs {
    send_to: super::super::receiver::SendTo,
}

impl OfflineArgs {
    pub fn from(item: CliOfflineArgs) -> color_eyre::eyre::Result<Self> {
        let send_to = match item.send_to {
            Some(cli_send_to) => super::super::receiver::SendTo::from(cli_send_to, None)?,
            None => super::super::receiver::SendTo::send_to(None)?,
        };
        Ok(Self { send_to })
    }
}

impl OfflineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = None;
        self.send_to
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
