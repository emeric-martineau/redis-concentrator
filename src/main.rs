#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod config;
mod lib;
mod sentinel;

use std::env;
use std::net::TcpStream;

use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::RedisConnector;
use crate::config::get_config;
use crate::sentinel::watch_sentinel;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn help() {
    println!("redis-concentrator {}", VERSION.unwrap_or("unknown"));
    println!();
    println!("Usage: redis-concentrator config-file");
    println!();
}

fn main() {
    // Get command line options
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        help();

        std::process::exit(-1);
    }

    let config_file = args[1].clone();

    // We load config file.
    let config = match get_config(config_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: give config file as argument");
            std::process::exit(-1);
        }
    };

    if config.sentinels.is_some() {
        watch_sentinel();
    } else {
        eprintln!("Error: no sentinels found in config file");
    }

    let tcp_stream = TcpStream::connect("127.0.0.1:26000").unwrap();

    let timeout = std::time::Duration::from_millis(1000);
    tcp_stream.set_read_timeout(Some(timeout));
    tcp_stream.set_nonblocking(false);

    let mut stream = NetworkStream::new(tcp_stream);
    let mut redis_connector = RedisConnector::new(&mut stream);

    /*println!("PING");

    match redis_connector.ping() {
        Ok(s) => println!("read: {:?}", s),
        Err(e) => println!("Error: {:?}", e)
    };*/

    println!("SUBSCRIBE");

    match redis_connector.subscribe("+switch-master") {
        Ok(mut s) => {
            loop {
                let a = s.pool();
                println!("Pool result: {:?}", a);
            }
        },
        Err(e) => println!("Error: {:?}", e)
    };

    /*match redis_connector.get_string("a") {
        Ok(s) => println!("read: {:?}", s),
        Err(e) => println!("Error: {:?}", e)
    };*/
}
