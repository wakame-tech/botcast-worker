mod api_client;
mod credential;
mod trpc;

use anyhow::Result;
use api_client::ApiClient;
use clap::Parser;
use credential::Credential;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
enum Args {
    Login {
        endpoint: String,
        #[clap(long)]
        email: String,
        #[clap(long)]
        password: String,
    },
    Script {
        id: String,
    },
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    let credential_path = PathBuf::from(".credential.json");

    if let Args::Login {
        endpoint,
        email,
        password,
    } = args
    {
        let client = ApiClient::new(&endpoint);
        let token = client.sign_in(&email, &password)?;
        Credential::save(&credential_path, endpoint, token)?;
        println!("credential saved to {:?}", credential_path);
        return Ok(());
    }

    let credential = Credential::load(&credential_path)?;
    let client = ApiClient::from_credential(&credential);

    match args {
        Args::Script { id } => {
            let script = client.script(&id)?;
            println!("{:?}", script);
        }
        _ => {}
    }
    Ok(())
}
