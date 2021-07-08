/// аргументы, необходимые для создания трансфера в offline mode
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOfflineArgs {
    #[clap(subcommand)]
    pub send_from: Option<super::online_mode::select_server::server::CliSendFrom>,
}

#[derive(Debug)]
pub struct OfflineArgs {
    send_from: super::online_mode::select_server::server::SendFrom,
}

impl OfflineArgs {
    pub fn from(item: CliOfflineArgs) -> color_eyre::eyre::Result<Self> {
        let send_from = match item.send_from {
            Some(cli_send_from) => {
                super::online_mode::select_server::server::SendFrom::from(cli_send_from, None)?
            }
            None => super::online_mode::select_server::server::SendFrom::choose_send_from(None)?,
        };
        Ok(Self { send_from })
    }
}

impl OfflineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = None;
        self.send_from
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
