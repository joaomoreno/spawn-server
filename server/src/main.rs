#[macro_use]
extern crate serde_derive;
extern crate rand;

extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;

mod codecs;
mod spawn;

use rand::os::OsRng;
use rand::Rng;
use futures::{Future, Stream, Sink};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;

use codecs::SpawnCodec;
use spawn::handle_spawn_requests;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let address = "127.0.0.1:0".parse().unwrap();
    let listener = TcpListener::bind(&address, &handle).unwrap();

    let port = listener.local_addr().unwrap().port();
    let token = OsRng::new().unwrap().gen_ascii_chars().take(32).collect::<String>();
    println!("{{\"port\": {}, \"token\": \"{}\"}}", port, token);

    let handle_connections = listener.incoming().for_each(move |(tcp_stream, _)| {
        let (responses_sink, requests_stream) = tcp_stream.framed(SpawnCodec).split();
        let responses = handle_spawn_requests(requests_stream, handle.clone());
        handle.spawn(responses_sink.send_all(responses).then(|_| Ok(())));
        Ok(())
    });

    core.run(handle_connections).unwrap();
}
