#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

use std::path::PathBuf;
use structopt::StructOpt;
use tracerbench_recorded_response_server::Config;
use tracerbench_recorded_response_server::Servers;

#[derive(StructOpt)]
pub struct Opt {
  #[structopt(parse(from_os_str))]
  pub cert: PathBuf,
  #[structopt(parse(from_os_str))]
  pub key: PathBuf,
  #[structopt(parse(from_os_str))]
  pub sets: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
  let opt = Opt::from_args();

  pretty_env_logger::init_timed();

  let config = Config::from_args(&opt.cert, &opt.key, &opt.sets)?;

  let servers: Servers = config.into();

  servers.start().await?;

  Ok(())
}
