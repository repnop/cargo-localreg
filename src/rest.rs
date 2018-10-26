use std::path::PathBuf;
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

        #[get("/index/*capture")]
        fn index(&self, capture: PathBuf) -> Result<String, ()> {
            Ok(format!("capture: {:?}", capture))
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
