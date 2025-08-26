use h_rocksdb::service::db::{get, put};
use rocksdb::{Options, DB};
use tempfile::TempDir;

fn create_test_db() -> (DB, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path();
    
    let mut opts = Options::default();
    opts.create_if_missing(true);
    let db = DB::open(&opts, path).expect("Failed to open test database");
    
    (db, temp_dir)
}

#[test]
fn test_put_success() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("test_key");
    let value = String::from("test_value");
    
    let result = put(&db, &key, &value);
    assert!(result.is_ok(), "Put operation should succeed");
}

#[test]
fn test_get_existing_key() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("test_key");
    let value = String::from("test_value");

    // First put the value
    db
        .put(key.as_bytes(), value.as_bytes())
        .expect("Direct put should succeed");

    // Then get it using our function
    let result = get(&db, &key);
    assert!(result.is_ok(), "Get operation should succeed");
    assert_eq!(result.unwrap(), Some(value), "Should retrieve the correct value");
}

#[test]
fn test_get_non_existing_key() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("non_existing_key");

    let result = get(&db, &key);
    assert!(result.is_ok(), "Get operation should succeed even for non-existing key");
    assert_eq!(result.unwrap(), None, "Should return None for non-existing key");
}

#[test]
fn test_put_and_get_integration() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("integration_key");
    let value = String::from("integration_value");

    // Put the value
    let put_result = put(&db, &key, &value);
    assert!(put_result.is_ok(), "Put operation should succeed");

    // Get the value back
    let get_result = get(&db, &key);
    assert!(get_result.is_ok(), "Get operation should succeed");
    assert_eq!(get_result.unwrap(), Some(value), "Should retrieve the same value that was put");
}

#[test]
fn test_put_empty_value() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("empty_value_key");
    let value = String::from("");

    let put_result = put(&db, &key, &value);
    assert!(put_result.is_ok(), "Should be able to put empty value");

    let get_result = get(&db, &key);
    assert!(get_result.is_ok(), "Should be able to get empty value");
    assert_eq!(get_result.unwrap(), Some(value), "Empty value should be preserved");
}

#[test]
fn test_put_overwrite_existing_key() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("overwrite_key");
    let value1 = String::from("original_value");
    let value2 = String::from("new_value");

    // Put first value
    let put_result1 = put(&db, &key, &value1);
    assert!(put_result1.is_ok(), "First put should succeed");

    // Overwrite with second value
    let put_result2 = put(&db, &key, &value2);
    assert!(put_result2.is_ok(), "Second put should succeed");

    // Verify the value was overwritten
    let get_result = get(&db, &key);
    assert!(get_result.is_ok(), "Get should succeed");
    assert_eq!(get_result.unwrap(), Some(value2), "Should get the new value after overwrite");
}

#[test]
fn test_unicode_key_value() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("こんにちは");
    let value = String::from("世界🌍");

    let put_result = put(&db, &key, &value);
    assert!(put_result.is_ok(), "Should handle Unicode characters in put");

    let get_result = get(&db, &key);
    assert!(get_result.is_ok(), "Should handle Unicode characters in get");
    assert_eq!(get_result.unwrap(), Some(value), "Unicode values should be preserved");
}

#[test]
fn test_large_value() {
    let (db, _temp_dir) = create_test_db();
    let key = String::from("large_value_key");
    let value = "x".repeat(1024 * 1024); // 1MB string

    let put_result = put(&db, &key, &value);
    assert!(put_result.is_ok(), "Should handle large values");

    let get_result = get(&db, &key);
    assert!(get_result.is_ok(), "Should retrieve large values");
    assert_eq!(get_result.unwrap(), Some(value), "Large value should be preserved");
}

#[test]
fn test_multiple_keys() {
    let (db, _temp_dir) = create_test_db();
    let test_data = vec![
        ("key1", "value1"),
        ("key2", "value2"),
        ("key3", "value3"),
    ];

    // Put all values
    for (key, value) in &test_data {
        let put_result = put(&db, &String::from(*key), &String::from(*value));
        assert!(put_result.is_ok(), "Put should succeed for key: {}", key);
    }

    // Get all values
    for (key, expected_value) in &test_data {
        let get_result = get(&db, &String::from(*key));
        assert!(get_result.is_ok(), "Get should succeed for key: {}", key);
        assert_eq!(
            get_result.unwrap(),
            Some(String::from(*expected_value)),
            "Value should match for key: {}",
            key
        );
    }
}
