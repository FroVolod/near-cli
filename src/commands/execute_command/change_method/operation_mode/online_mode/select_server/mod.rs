use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod server;

#[derive(Debug, clap::Clap)]
pub enum CliSelectServer {
    /// предоставление данных для сервера https://rpc.testnet.near.org
    Testnet(self::server::CliServer),
    /// предоставление данных для сервера https://rpc.mainnet.near.org
    Mainnet(self::server::CliServer),
    /// предоставление данных для сервера https://rpc.betanet.near.org
    Betanet(self::server::CliServer),
    /// предоставление данных для сервера, указанного вручную
    Custom(self::server::CliCustomServer),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SelectServer {
    #[strum_discriminants(strum(message = "Testnet"))]
    Testnet(self::server::Server),
    #[strum_discriminants(strum(message = "Mainnet"))]
    Mainnet(self::server::Server),
    #[strum_discriminants(strum(message = "Betanet"))]
    Betanet(self::server::Server),
    #[strum_discriminants(strum(message = "Custom"))]
    Custom(self::server::Server),
}

impl SelectServer {
    pub fn from(item: CliSelectServer) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSelectServer::Testnet(cli_server) => Ok(Self::Testnet(
                cli_server.into_server(crate::common::ConnectionConfig::Testnet)?,
            )),
            CliSelectServer::Mainnet(cli_server) => Ok(Self::Mainnet(
                cli_server.into_server(crate::common::ConnectionConfig::Mainnet)?,
            )),
            CliSelectServer::Betanet(cli_server) => Ok(Self::Betanet(
                cli_server.into_server(crate::common::ConnectionConfig::Betanet)?,
            )),
            CliSelectServer::Custom(cli_custom_server) => {
                Ok(Self::Custom(cli_custom_server.into_server()?))
            }
        }
    }
}

impl SelectServer {
    pub fn choose_server() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = SelectServerDiscriminants::iter().collect::<Vec<_>>();
        let servers = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_server = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select NEAR protocol RPC server:")
            .items(&servers)
            .default(0)
            .interact()
            .unwrap();
        let cli_select_server = match variants[selected_server] {
            SelectServerDiscriminants::Testnet => CliSelectServer::Testnet(Default::default()),
            SelectServerDiscriminants::Mainnet => CliSelectServer::Mainnet(Default::default()),
            SelectServerDiscriminants::Betanet => CliSelectServer::Betanet(Default::default()),
            SelectServerDiscriminants::Custom => CliSelectServer::Custom(Default::default()),
        };
        Ok(Self::from(cli_select_server)?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        Ok(match self {
            SelectServer::Testnet(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
            SelectServer::Mainnet(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
            SelectServer::Betanet(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
            SelectServer::Custom(server) => {
                server.process(prepopulated_unsigned_transaction).await?;
            }
        })
    }
}
