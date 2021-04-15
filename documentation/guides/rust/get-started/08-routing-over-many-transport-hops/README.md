```
title: Routing over many transport hops
```

# Routing over many transport hops

## Responder node

```rust
// examples/08-routing-over-transport-many-hops-responder.rs

use ockam::{Context, Result};
use ockam_get_started::Echoer;
use ockam_transport_tcp::TcpTransport;

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    TcpTransport::create_listener(&ctx, "127.0.0.1:6000").await?;

    // Create an echoer worker
    ctx.start_worker("echoer", Echoer).await?;

    // This node never shuts down.
    Ok(())
}
```

## Middle node

```rust
// examples/08-routing-over-transport-many-hops-middle.rs

use ockam::{Context, Result};
use ockam_transport_tcp::TcpTransport;

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    TcpTransport::create_listener(&ctx, "127.0.0.1:4000").await?;
    TcpTransport::create(&ctx, "127.0.0.1:6000").await?;

    // This node never shuts down.
    Ok(())
}
```

## Initiator node

```rust
// examples/08-routing-over-transport-many-hops-initiator.rs

use ockam::{Context, Result, Route};
use ockam_transport_tcp::{TcpTransport, TCP};

#[ockam::node]
async fn main(mut ctx: Context) -> Result<()> {
    TcpTransport::create(&ctx, "127.0.0.1:4000").await?;

    ctx.send(
        Route::new()
            // Send a message to node B
            .append_t(TCP, "127.0.0.1:4000")
            // Send a message to node C
            .append_t(TCP, "127.0.0.1:6000")
            // Echo worker on node C
            .append("echoer"),
        "Hello Ockam!".to_string(),
    )
    .await?;

    // Wait to receive a reply and print it.
    let reply = ctx.receive::<String>().await?;
    println!("Initiator Received: {}", reply); // should print "Hello Ockam!"

    ctx.stop().await
}
```

<div style="display: none; visibility: hidden;">
<hr><b>Next:</b> <a href="../09-secure-channel-over-many-transport-hops">09. Secure Channel over many transport hops</a>
</div>