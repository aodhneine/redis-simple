# redis_simple

Simple and naive implementation of Redis protocol in Rust, made to be as clear as possbile.

There's almost no abstraction over actual communication with the server.

## Examples

``` rust
use redis_simple::*;
let conn = redis_simple::Connection::new("localhost:6379")?;
conn.try_execute("SET name redis_simple")?;
```

## Installation

``` toml
# ...

[dependencies]
redis_simple = { git = "https://github.com/aodhneine/redis_simple" }

# ...
```
