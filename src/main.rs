use structopt::StructOpt;

mod rest;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cargo-localreg",
    about = "Local crate registry management tool"
)]
enum LocalReg {
    #[structopt(name = "start", about = "Starts the local registry server")]
    Start { port: Option<u16> },
    #[structopt(
        name = "publish",
        about = "Publishes the current crate to the local registry"
    )]
    Publish {},
}

fn main() {
    match LocalReg::from_args() {
        LocalReg::Start { port } => {
            let port = port.unwrap_or(1234);
            rest::run(port);
        }
        LocalReg::Publish {} => {}
    }
}
