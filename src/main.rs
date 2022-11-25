mod core;
use crate::core::utils::RunType;

#[tokio::main]
async fn main() {
    let args = core::utils::Args::handle();

    match args.runType {
        RunType::Master => core::run::master(args.address).await,
        _ => core::run::slave(args.address).await,
    }
}
