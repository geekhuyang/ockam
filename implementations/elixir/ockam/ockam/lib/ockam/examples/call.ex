defmodule Ockam.Example.Call do
  use Ockam.Worker

  alias Ockam.Message

  require Logger

  def call(call, options \\ []) do
    {:ok, address} = __MODULE__.create(options ++ [call: call])
    GenServer.call(Ockam.Node.whereis(address), :fetch, 10000)
  end

  @impl true
  def setup(options, state) do
    call = Keyword.fetch!(options, :call)
    send_call(call, state)
    {:ok, state}
  end

  def send_call(call, state) do
    Ockam.Router.route(%{
      payload: Message.payload(call),
      onward_route: Message.onward_route(call),
      return_route: [state.address]
      })
  end

  @impl true
  def handle_message(message, state) do
    Map.put(state, :message, message)
    case Map.get(state, :wait) do
      nil ->
        {:ok, state}
      waiter ->
        GenServer.reply(waiter, message)
        ## Terminate here
        {:stop, :shutdown, state}
    end
  end

  @impl true
  def handle_call(:fetch, from, state) do
    case Map.get(state, :message) do
      nil ->
        {:noreply, Map.put(state, :wait, from)}
      message ->
        GenServer.reply(from, message)
        ## Terminate here
        {:stop, :shutdown, state}
    end
  end
end