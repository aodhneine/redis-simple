# redis-simple

Simple and naive implementation of Redis protocol in Rust, made to be as clear as possbile.

There's almost no abstraction over actual communication with the server.

## Examples

``` rust
use redis_simple::*;
let conn = redis_simple::Connection::new("localhost:6379")?;
conn.try_execute("SET name redis_simple")?;
let name = conn.try_execute("GET name")?;
```

## Installation

``` toml
# ...

[dependencies]
redis-simple = { git = "https://github.com/aodhneine/redis-simple.git" }

# ...
```
