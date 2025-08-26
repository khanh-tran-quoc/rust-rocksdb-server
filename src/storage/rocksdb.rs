use rocksdb::{Error, DB};

pub fn put(db: &DB, key: &String, value: &String) -> Result<(), Error> {
    match db.put(key.as_bytes(), value.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error put key \"{:?}\": {:}", key, e);
            Err(e)
        }
    }
}

pub fn get(db: &DB, key: &String) -> Result<Option<String>, Error> {
    match db.get(key.as_bytes()) {
        Ok(Some(value)) => Ok(Some(String::from_utf8(value).unwrap())),
        Ok(None) => Ok(None),
        Err(e) => {
            println!("Error get key \"{:?}\": {:}", key, e);
            Err(e)
        }
    }
}
