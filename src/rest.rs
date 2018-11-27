use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};
use tokio::{fs::File, prelude::Future};
use tower_web::*;

struct RestServer;

impl_web! {
    impl RestServer {
        #[get("/root")]
        #[content_type("plain")]
        fn root(&self) -> impl Future<Item = File, Error = std::io::Error> {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push(file!());
            File::open(path)
        }

        #[get("/api/crates/:krate/:version/download")]
        fn download_crate(&self, krate: String, version: String) -> Result<String, ()> {
            Ok(format!("crate: {}\nversion: {}", krate, version))
        }

        #[post("/api/crates/:krate/:version/")]
        fn upload_crate(&self, body: Vec<u8>, krate: String, version: String) -> Result<String, ()> {
            let mut index = dirs::data_local_dir().unwrap();
            index.push("local_registry");
            index.push("dl");
            index.push(&krate);
            index.push(&version);
            index.push(&format!("{}-{}.crate", krate, version));

            std::fs::create_dir_all(index.parent().unwrap()).unwrap();

            std::fs::write(&index, &body).unwrap();
            println!("yeetus, {:?}", index);
            Ok("lol ok".into())
        }

        #[get("/index/*capture")]
        fn index(&self, capture: PathBuf) -> Result<String, ()> {
            println!("foo");
            Ok(format!("capture: {:?}", capture))
        }

        #[post("/index/*capture")]
        fn publish(&self, capture: PathBuf, body: Vec<u8>) -> Result<String, ()> {

            let mut index = dirs::data_local_dir().unwrap();
            index.push("local_registry");
            index.push("index");
            index.push(&capture);

            std::fs::create_dir_all(index.parent().unwrap()).unwrap();

            let body = String::from_utf8(body).unwrap();
            let body_data: crate::Data = serde_json::from_str(&body).unwrap();

            let mut file = BufReader::new(OpenOptions::new().append(true).read(true).create(true).open(&index).unwrap());

            for line in file.by_ref().lines() {
                let data: crate::Data = serde_json::from_str(&line.unwrap()).unwrap();
                if data.vers == body_data.vers {
                    return Err(());
                }
            }

            let mut file = file.into_inner();

            file.write_all(body.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
            println!("{:?}", index);

            Ok("lol ok".into())
        }
    }
}

pub fn run(port: u16) {
    let addr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Invalid address");
    println!("Listening on http://{}", addr);

    ServiceBuilder::new()
        .resource(RestServer)
        .run(&addr)
        .unwrap();
}
