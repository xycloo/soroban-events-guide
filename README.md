# Playing with Soroban Events

### TL;DR
Soroban provides an `events()` handle in the environment that allows us to publish events. In this post we'll explore how to publish events, why a contract would publish one, and how to read these events (so one can set up a listener for them).

## What are smart contract events?

Events are something that exists on most smart contract platforms, and are a way of dispatching signals to the outer ecosystem (they are not readable from contracts). Events are the way contracts can comunicate with a dApp or frontend in general.

## Why events exist?
When a user interacts with a contracts, knowing what happens in an invocation is extremely important, and there are many case scenarios where the returned value from the contract invocation is opaque (we know that the transaction succeeded, but we don't know what happened exactly). For example:

```rust
fn test_fn(e: Env) -> Result<(), Error> {
	let invoker = e.invoker();
	
	if some_condition(invoker) {
		do_this();
		Ok(())
	} else some_other_condition(invoker) {
		do_that();
		Ok(())
	} else {
	    Err(Error::NoCondition)
	}
}
```

As you can see, different things can happen during this invocation, and the contract would always return the same. In this situation, contracts could use events to log that an action triggered by a condition is happening, so that the invoker knows what happened without having to run checks themselves. Also, in most cases you might need to emit multiple events for one invocation, which means that a return type won't do the job (not in a clean manner at least).

Another advantage of events is that they are made to be indexable, easing the process of listening/searching.

## Publishing events

The syntax for publishing events is very simple on Soroban:

```rust
let event = env.events();
let topic = (symbol!("mytopic"),);
let data = Bytes::from_array(&env, &[104, 101, 108, 108, 111]);
event.publish(topic, data);
```
As you can see, we first build the `Events` type with `env.events()`, and then publish an event with certain topics and associated data.

### What are topics?
Generally speaking smart contract event topics are the indexed parameters to the event. For example in solidity, you define an Event object:

```solidity
event SomeEvent(uint indexed mynum, unint other);
```

And then use `emit` to log the data to the blockchain:

```solidity
emit SomeEvent(2, 0)
```

What happens here is that we are creating a log entry with the topics: `SomeEvent(uint, uint)` (as hash) and the topic `2` (the `indexed` param value). Note that `0` is also stored, but not as a topic, it will be in the event's data section.

In Soroban, things are currently slightly different:

```rust
let topics = (symbol!("mynum"),);
let data = (2u32, 0u32);

events.publish(topics, data);
```

This will create an event with topics `(mynum)` and data `(2, 0)`. So you can index the event based on `"mynum"`.


## Developing a contract that publishes events
Now that you know what events are, how they work and how topics work, you are ready to build a simple contract that publishes an event when it is initialized:

```rust
use soroban_auth::Identifier;
use soroban_sdk::{contractimpl, serde::Serialize, symbol, Env};

/// Contract trait
pub trait EventsContractTrait {
    fn init(e: Env, admin: Identifier);
}

pub struct EventsContract;

#[contractimpl]
impl EventsContractTrait for EventsContract {
    fn init(e: Env, admin: Identifier) {
        let event = e.events();
        let t1 = (symbol!("init"),);

        let id_bytes = admin.serialize(&e);
        event.publish(t1, (id_bytes, ));
    }
}

```

This contract comunicates to its users that it was initialized with a certain admin identifier (serialized into bytes, so a buffer of `0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 5, 0, 0, 0, 7, 65, 99, 99, 111, 117, 110, 116, 0, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0, 0,` + bytes of the strkey decoded public key).

In our case, testing doesn't really make sense unless you just want to verify that the `init` function executes, since the logged data cannot be accessed from a test/contract via the environment. The only way to thorughly test this contract is to deploy it to futurent and invoke the init function.

### Deployment

We first need to compile the contract (the built options are to build optimized WASM binaries):

```bash
~/De/soroban-events-playground main !5 ❯ cargo +nightly build \
    --target wasm32-unknown-unknown \
    --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort
   Compiling soroban-events-playground-contract v0.0.0 (/home/tommasodeponti/Desktop/soroban-events-playground)
    Finished release [optimized] target(s) in 0.37s

```

Then we can deploy the contract:

```bash
soroban deploy \      
    --wasm target/wasm32-unknown-unknown/release/soroban_events_playground_contract.wasm  --secret-key SDUOZUZMO7FDMBTAXVFMKLOGERC7AMGZB465LAAQ57NJWKDKG7FKSADY --rpc-url  https://future.stellar.kai.run:443/soroban/rpc --network-passphrase 'Test SDF Future Network ; October 2022'                          
success
d37dd09bca47a5bb3033655047ff61c90ed7107ade9c632603187a9a0dd5074c
```

### Invoking

Assuming that you are already familiar with the soroban CLI basics, invoking shouldn't be difficult:

```bash
soroban invoke --id d37dd09bca47a5bb3033655047ff61c90ed7107ade9c632603187a9a0dd5074c \
  --secret-key SB3YZR6KMXEOEWAMS4HUQX4JTWW6METEP3LAXSH2F3GQQT4LOYCR3A44 \
  --rpc-url https://future.stellar.kai.run:443/soroban/rpc \
  --network-passphrase 'Test SDF Future Network ; October 2022' \
  --fn init --arg '{"object":{"vec":[{"symbol":"Account"},{"object":{"accountId":{"publicKeyTypeEd25519":"5baa8f1a7526268d1faff4b04177800a5b323f00bc3d27fb6c33833e10d0518d"}}}]}}'
success
null
```

Remember that to build the `Identifier` we constructed the `Identifier::Account(AccountId)` as the JSON representation. If you're not familiar with this at all, you can check out [our guide](https://github.com/xycloo/soroban-cli-futurenet).

### Reading the event
Our invocation will result in a transaction with a `InvokeHostFunctionOp` operation. Currenlty, we can find our event in the transaction's result metadata.

This means that we'll have to locate the transaction, and then look at it from horizon to obtain the `TxResultMeta`'s XDR. In my case, [this](https://horizon-futurenet.stellar.org/transactions/914dea439a1f052d89039f438ffc95f707a67891ff15bd1eb17463020f0b8acc) was the invocation transaction. To find yours you can also look at [Soroban Fiddle](https://leighmcculloch.github.io/soroban-fiddle/). 

Now that we have the XDR (`AAAAAwAAAAIAAAADAA8powAAAAAAAAAAPoOJtiEgMrkMLE7ug0nnQZ2Jonc/Hdf/z+foCIsrUxsAAAAXSHbhwAAOTDAAAAAPAAAAAQAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAwAAAAAADl/LAAAAAGN9IpIAAAAAAAAAAQAPKaMAAAAAAAAAAD6DibYhIDK5DCxO7oNJ50GdiaJ3Px3X/8/n6AiLK1MbAAAAF0h24cAADkwwAAAAEAAAAAEAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAMAAAAAAA8powAAAABjgRP2AAAAAAAAAAEAAAAAAAAAAAAAAAEAAAAAAAAAAbcurZbKXu8AXk2iS+oXuPMJxF48RGRh6ddSk9aqrKAAAAAAAQAAAAAAAAABAAAABQAAAARpbml0AAAABAAAAAEAAAAEAAAAUAAAAAQAAAABAAAAAAAAAAIAAAAFAAAAB0FjY291bnQAAAAABAAAAAEAAAAHAAAAAFuqjxp1JiaNH6/0sEF3gApbMj8AvD0n+2wzgz4Q0FGNAAAAAAAAAGQAAAAAAAAAAQAAAAAAAAAYAAAAAAAAAAMAAAAAAAAAAPtKlS+NuolelIJrIn5jV26Hul945cSBGyXTmiwy50Wq6fNHY5h5xGw0fSQ+E4DLlQgS7wNFJuy7dbX1/S0VALkzhhDHHyug9wgX+sGGnfU5/niQQOWCkHZShBN5TwmwBA==`), we can open-up the laboratory and [decode](https://laboratory.stellar.org/#xdr-viewer?input=AAAAAwAAAAIAAAADAA8powAAAAAAAAAAPoOJtiEgMrkMLE7ug0nnQZ2Jonc%2FHdf%2Fz%2BfoCIsrUxsAAAAXSHbhwAAOTDAAAAAPAAAAAQAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAAAAAAAAAAAAAAAAAwAAAAAADl%2FLAAAAAGN9IpIAAAAAAAAAAQAPKaMAAAAAAAAAAD6DibYhIDK5DCxO7oNJ50GdiaJ3Px3X%2F8%2Fn6AiLK1MbAAAAF0h24cAADkwwAAAAEAAAAAEAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAAAAAAAAAAAAAAAAAMAAAAAAA8powAAAABjgRP2AAAAAAAAAAEAAAAAAAAAAAAAAAEAAAAAAAAAAbcurZbKXu8AXk2iS%2BoXuPMJxF48RGRh6ddSk9aqrKAAAAAAAQAAAAAAAAABAAAABQAAAARpbml0AAAABAAAAAEAAAAEAAAAUAAAAAQAAAABAAAAAAAAAAIAAAAFAAAAB0FjY291bnQAAAAABAAAAAEAAAAHAAAAAFuqjxp1JiaNH6%2F0sEF3gApbMj8AvD0n%2B2wzgz4Q0FGNAAAAAAAAAGQAAAAAAAAAAQAAAAAAAAAYAAAAAAAAAAMAAAAAAAAAAPtKlS%2BNuolelIJrIn5jV26Hul945cSBGyXTmiwy50Wq6fNHY5h5xGw0fSQ%2BE4DLlQgS7wNFJuy7dbX1%2FS0VALkzhhDHHyug9wgX%2BsGGnfU5%2FniQQOWCkHZShBN5TwmwBA%3D%3D&type=TransactionMeta&network=futurenet) it. You can then find the events logged in this transaction:

```
events: Array[1]
	[0]
		ext: [undefined]
		contractId: ty6tlspe7wBeTaJL6he48wnEXjxEZGHp11KT1qqsoAA=
		type
		body: [undefined]
		v0
			topics: Array[1]
				[0]: [scvSymbol]
					sym: aW5pdA==
				data: [scvObject]
					obj: [scoBytes]
						bin: AAAABAAAAAEAAAAAAAAAAgAAAAUAAAAHQWNjb3VudAAAAAAEAAAAAQAAAAcAAAAAW6qPGnUmJo0fr/SwQXeAClsyPwC8PSf7bDODPhDQUY0=

```

Congratulations! You have created and read your first Soroban event.
