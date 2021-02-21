use ockam::Address;
use std::net::SocketAddr;
use std::str::FromStr;
use tcp_examples::echoer::{EchoMessage, Echoer};

fn main() {
    let (ctx, mut exe) = ockam::start_node();

    exe.execute(async move {
        let listen_addr = SocketAddr::from_str("127.0.0.1:4051").unwrap();
        let mut listener = ockam_transport_tcp::TcpListener::create(listen_addr)
            .await
            .unwrap();
        let connection = listener.accept().await.unwrap();
        let echoer = Echoer {
            connection,
            count: 0,
        };

        let address: Address = "echoer".into();
        ctx.start_worker(address, echoer).await.unwrap();

        ctx.send_message("echoer", EchoMessage::Receive)
            .await
            .unwrap();

        ctx.send_message("echoer", EchoMessage::Send("Hello".into()))
            .await
            .unwrap();
    })
    .unwrap();
}