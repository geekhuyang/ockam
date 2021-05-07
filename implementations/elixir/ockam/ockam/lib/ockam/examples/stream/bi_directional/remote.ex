defmodule Ockam.Example.Stream.BiDirectional.Remote do
  @moduledoc false

  alias Ockam.Example.Stream.Ping
  alias Ockam.Example.Stream.Pong

  alias Ockam.Message

  alias Ockam.Stream.Client.BiDirectional


  @hub_tcp %Ockam.Transport.TCPAddress{ip: {127, 0, 0, 1}, port: 4000}

  # def init_ping() do
  #   ensure_tcp(3000)
  #   Ping.create(address: "ping")
  #   BiDirectional.subscribe(stream_name: "ping_topic", stream_options: stream_options())
  #   PublisherRegistry.start_link([])
  # end

  def init_pong() do
    ensure_tcp(5000)

    ## We run a call named "pong" to create an alias route to "pong"
    ## This should happen before creating the proper "pong"
    alias_address = register_alias("pong")
    :timer.sleep(1000)
    {:ok, "pong"} = Pong.create(address: "pong")
    subscribe("pong_topic")

    alias_address
  end

  def register_alias(address) do

    reply = Ockam.Example.Call.call(%{
      onward_route: [@hub_tcp, "forwarding_service"],
      payload: "register"
      }, address: address)
    alias_route = Message.return_route(reply)
    List.last(alias_route)
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
    # BiDirectional.subscribe(stream, stream_options())
    # PublisherRegistry.start_link([])

    ## Remote subscribe

    subscribe_msg = %{
      onward_route: [@hub_tcp, "stream_subscribe"],
      return_route: [],
      payload: Ockam.Protocol.encode_payload(Ockam.Protocol.Stream.BiDirectional, :request, %{
        stream_name: stream,
        subscription_id: "foo"
        })
    }

    Ockam.Router.route(subscribe_msg)
    ## No return yet, so just wait
    :timer.sleep(2000)
  end


  def run(pong_address) do
    ensure_tcp(3000)
    Ping.create(address: "ping")

    subscribe("ping_topic")

    # Local publisher
    # {:ok, address} = init_publisher("pong_topic", "ping_topic")
    # send_message([address])

    # Remote publisher
    reply = Ockam.Example.Call.call(%{
      onward_route: [@hub_tcp, "stream_subscribe"],
      payload: Ockam.Protocol.encode_payload(Ockam.Protocol.Stream.BiDirectional.EnsurePublisher, :request, %{
        publisher_stream: "pong_topic",
        consumer_stream: "ping_topic",
        subscription_id: "foo"
        })
      })

    publisher_route = Message.return_route(reply)

    send_message(publisher_route, pong_address)
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
