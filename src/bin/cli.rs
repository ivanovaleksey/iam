extern crate iam;
#[macro_use]
extern crate quicli;
extern crate uuid;

use iam::authn::jwt::{AccessToken, RawToken, RawTokenKind};
use quicli::prelude::*;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "decode", about = "Decode token")]
    Decode {
        #[structopt(long = "jwt")]
        jwt: String,
    },
    #[structopt(name = "encode", about = "Generate new token")]
    Encode {
        #[structopt(long = "aud")]
        aud: String,
        #[structopt(long = "exp")]
        exp: u32,
        #[structopt(long = "sub")]
        sub: uuid::Uuid,
    },
}

main!(|args: Cli| {
    if let Err(e) = iam::settings::init() {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    match args.cmd {
        Command::Decode { jwt } => {
            let raw_token = RawToken {
                kind: RawTokenKind::Iam,
                value: &jwt,
            };

            match AccessToken::decode(&raw_token) {
                Ok(token) => println!("{:?}", token),
                Err(e) => eprintln!("{}", e),
            }
        }
        Command::Encode { aud, exp, sub } => {
            let token = AccessToken::new(aud, exp, sub);
            match AccessToken::encode(token) {
                Ok(jwt) => println!("{}", jwt),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }
});
