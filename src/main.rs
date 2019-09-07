use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "git-backup")]
struct Opts {
    /// Location to place the backups. If not provided,
    /// the current working directory will be used
    #[structopt(short="l", long="location", parse(from_os_str))]
    backup_location: Option<PathBuf>,
    /// URL of repositories to back up
    #[structopt(name="URL")]
    url: String
}

fn main() {
    let opts = Opts::from_args();
    println!("{:#?}", opts);
}
