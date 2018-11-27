use serde_derive::Deserialize;
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Write},
};
use structopt::StructOpt;

mod cargo_manifest;

macro_rules! trybail {
    ($($x:tt)+) => {
        match $($x)+ {
            Ok(ok) => ok,
            Err(e) => {
                eprintln!("Error: {}", e.message());
                std::process::exit(1)
            }
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "cargo-localreg",
    about = "Local crate registry management tool"
)]
enum LocalReg {
    #[structopt(
        name = "publish",
        about = "Publishes the current crate to the local registry"
    )]
    Publish,
    #[structopt(name = "add")]
    Add { name: String },
}

#[derive(Deserialize)]
struct Data {
    name: String,
    vers: String,
}

fn commit(repo: &git2::Repository, message: &str) -> Result<(), git2::Error> {
    let mut repo_index = repo.index().unwrap();

    repo_index
        .add_all(&["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();

    repo_index.write().unwrap();

    let sig = repo.signature().unwrap();
    let tree = repo.find_tree(repo_index.write_tree().unwrap()).unwrap();

    let mut parents = Vec::new();
    match repo.head().ok().map(|h| h.target().unwrap()) {
        Some(parent) => parents.push(repo.find_commit(parent).unwrap()),
        None => {}
    }
    let parents = parents.iter().collect::<Vec<_>>();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .unwrap();

    repo.checkout_head(None).unwrap();

    Ok(())
}

fn main() {
    // HACK: Cargo (for whatever reason) puts the subcommand name in the args
    // which doesn't work with structopt subcommands.
    match LocalReg::from_iter(std::env::args().filter(|s| s != "localreg")) {
        LocalReg::Publish {} => {
            let (mut index, mut download) = {
                let mut idx = dirs::data_local_dir().unwrap();
                idx.push("local_registry");
                let mut dl = idx.clone();

                idx.push("index");
                dl.push("dl");

                (idx, dl)
            };

            let repo = if !index.exists() {
                let repo = trybail!(git2::Repository::init(&index));
                let mut config = index.clone();
                std::fs::create_dir_all(&config).unwrap();
                config.push("config.json");

                let mut f = std::fs::File::create(&config).unwrap();
                f.write_all(
                    format!(
                        include_str!("../config.json"),
                        localdir = config.parent().unwrap().parent().unwrap().to_str().unwrap()
                    )
                    .as_bytes(),
                )
                .unwrap();
                trybail!(commit(&repo, "Initializing repo.",));

                repo
            } else {
                trybail!(git2::Repository::open(&index))
            };

            // TODO: check for fail
            let _ = std::process::Command::new("cargo")
                .args(&["package", "--allow-dirty"])
                .stdout(std::process::Stdio::inherit())
                .output()
                .unwrap();

            let manifest = cargo_manifest::generate_registry_json().unwrap();
            let data = serde_json::from_str::<Data>(&manifest).unwrap();

            let index_append = match data.name.len() {
                1 => format!("1/{}", data.name),
                2 => format!("2/{}", data.name),
                3 => format!("3/{}/{}", data.name.chars().nth(0).unwrap(), data.name),
                _ => {
                    let first_two = data.name.chars().take(2).collect::<String>();
                    let second_two = data.name.chars().skip(2).take(2).collect::<String>();
                    format!("{}/{}/{}", first_two, second_two, data.name)
                }
            };

            index.push(&index_append);

            std::fs::create_dir_all(index.parent().unwrap()).unwrap();

            let mut file = BufReader::new(
                OpenOptions::new()
                    .append(true)
                    .read(true)
                    .create(true)
                    .open(&index)
                    .unwrap(),
            );

            let manifest_data: Data = serde_json::from_str(&manifest).unwrap();

            for line in file.by_ref().lines() {
                let data: Data = serde_json::from_str(&line.unwrap()).unwrap();
                if data.vers == manifest_data.vers {
                    eprintln!("Version already published.");
                    std::process::exit(1);
                }
            }

            let mut file = file.into_inner();

            file.write_all(manifest.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();

            trybail!(commit(
                &repo,
                &format!(
                    "Adding {krate}-{version}.",
                    krate = manifest_data.name,
                    version = manifest_data.vers
                ),
            ));

            download.push(&manifest_data.name);
            download.push(&manifest_data.vers);

            std::fs::create_dir_all(&download).unwrap();

            let crate_file = format!("{}-{}.crate", manifest_data.name, manifest_data.vers);
            download.push(&crate_file);

            std::fs::copy(&format!("./target/package/{}", crate_file), &download).unwrap();

            println!("Published crate successfully.");
        }
        LocalReg::Add { name: _name } => {}
    }
}
