/// Requires running Redis server!
#[test]
fn connection() {
  const ADDRESS: &str = "localhost:16379";
  assert!(crate::Connection::new(ADDRESS).is_ok(),
    "Failed to create connection to Redis database!");
}

/// Requires running Redis server AND clean redis instance.
#[test]
fn protocol() {
  const ADDRESS: &str = "localhost:16379";
  if let Ok(mut conn) = crate::Connection::new(ADDRESS) {
    assert_eq!(conn.try_execute("SET name aodhneine").unwrap(), crate::ReturnType::State("OK".to_string()));
    assert_eq!(conn.try_execute("GET name").unwrap(), crate::ReturnType::BulkString {
      data: "aodhneine".to_string(),
      size: 9,
    });
    assert_eq!(conn.try_execute("SET id 120").unwrap(), crate::ReturnType::State("OK".to_string()));
    assert_eq!(conn.try_execute("GET id").unwrap(), crate::ReturnType::BulkString {
      data: "120".to_string(),
      size: 3,
    });
    assert_eq!(conn.try_execute("INCR id").unwrap(), crate::ReturnType::Integer(121.to_string()));
    assert_eq!(conn.try_execute("RPUSH alist kyu").unwrap(), crate::ReturnType::Integer(1.to_string()));
    assert_eq!(conn.try_execute("LRANGE alist 0 0").unwrap(), crate::ReturnType::Array(vec![Box::new(crate::ReturnType::BulkString {
      data: "kyu".to_string(),
      size: 3,
    })]));
    assert_eq!(conn.try_execute("LSET alist 0 aodhneine").unwrap(), crate::ReturnType::State("OK".to_string()));
    assert_eq!(conn.try_execute("LRANGE alist 0 0").unwrap(), crate::ReturnType::Array(vec![Box::new(crate::ReturnType::BulkString {
      data: "aodhneine".to_string(),
      size: 9,
    })]));
  }
}
