defmodule Ockam.Example.Stream.BiDirectional.Local do
  @moduledoc false

  alias Ockam.Example.Stream.Ping
  alias Ockam.Example.Stream.Pong

  alias Ockam.Stream.Client.BiDirectional
  alias Ockam.Stream.Client.BiDirectional.PublisherRegistry


  @hub_tcp %Ockam.Transport.TCPAddress{ip: {127, 0, 0, 1}, port: 4000}

  def init_ping() do
    ensure_tcp(3000)
    Ping.create(address: "ping")
    subscribe("ping_topic")
  end

  def init_pong() do
    ensure_tcp(5000)
    {:ok, "pong"} = Pong.create(address: "pong")
    subscribe("pong_topic")
  end

  def stream_options() do
    [
      service_route: [@hub_tcp, "stream_service"],
      index_route: [@hub_tcp, "stream_index_service"],
      partitions: 1
    ]
  end

  def subscribe(stream) do
    ## Local subscribe
    BiDirectional.subscribe(stream, stream_options())
    PublisherRegistry.start_link([])

    ## Remote subscribe
  end


  def run() do
    ensure_tcp(3000)
    Ping.create(address: "ping")

    subscribe("ping_topic")

    # Local publisher
    {:ok, address} = init_publisher("pong_topic", "ping_topic")
    send_message([address], "pong")

  end

  def init_publisher(publisher_stream, consumer_stream) do
    BiDirectional.ensure_publisher(
      consumer_stream,
      publisher_stream,
      stream_options()
    )
  end

  def send_message(publisher_route, pong_address) do
    msg = %{
      onward_route: publisher_route ++ [pong_address],
      return_route: ["ping"],
      payload: "0"
    }

    Ockam.Router.route(msg)
  end

  def ensure_tcp(port) do
    Ockam.Transport.TCP.create_listener(port: port, route_outgoing: true)
  end
end
