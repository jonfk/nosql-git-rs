use anyhow::Result;
use clap::{AppSettings, Clap};
use commit::ToCommit;
use git2::Repository;
use git_ops::{clone, commit, log};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Jonfk <jfokkan@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    Clone {
        url: String,
        path: Option<String>,
    },
    Log {},
    Commit {
        path: String,
        #[clap(short, long)]
        message: String,
    },
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    println!("{:?}", opts);

    let res: Result<()> = match opts.subcommand {
        SubCommand::Clone { url, path } => {
            clone::clone_ssh(&url, path.unwrap_or(".".to_string()))?;
            Ok(())
        }
        SubCommand::Log {} => {
            let repo = Repository::open(".")?;
            log::git_log(repo)?;
            Ok(())
        }
        SubCommand::Commit { path, message } => {
            let repo = Repository::open(".")?;
            commit::commit(
                &repo,
                &ToCommit {
                    path: path,
                    message: message,
                },
            )?;
            Ok(())
        }
    };
    res?;
    Ok(())
}
