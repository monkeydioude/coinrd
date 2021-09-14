use std::marker::PhantomData;

use mongodb::{bson::{Bson, doc, from_bson, to_bson, ser::Error, Document}, options::ReplaceOptions, sync::{Collection as MongoColl, Database as MongoDatabase}};
use serde::{Serialize, Deserialize};
use log::{warn, error};

pub trait Collection<T> {
  fn find_one(self: &Self, id: String) -> Option<T>
  where T: for <'a> Deserialize<'a> + std::fmt::Debug;

  fn save(self: &Self, id: String, entity: T)
  where T: Serialize;

  fn insert(self: &Self, entity: T)
  where T: Serialize;
}

// MongoDB acts as a light factory for
// MongoCollection<T> trait Collection
pub struct MongoDB {
  db: MongoDatabase,
}

impl MongoDB {
  pub fn new (db: MongoDatabase) -> Self {
    Self {
      db,
    }
  }

  pub fn new_collection<T>(&self, coll: &str) -> MongoCollection<T> {
    MongoCollection {
      collection: self.db.collection(coll),
      pd: PhantomData{},
    }
  }

  // to_mongo_db gives Mongo's Database struct
  pub fn to_mongo_db(&self) -> MongoDatabase {
    self.db.to_owned()
  }
}

pub struct MongoCollection<T> {
  collection: MongoColl,
  pd: PhantomData<T>,
}

// MongoCollection<T> implements Collection<T> traits
// for various operations on a MongoDB Collection
impl<T> Collection<T> for MongoCollection<T> {
  fn find_one(&self, id: String) -> Option<T>
  where T: for<'de> Deserialize<'de> + std::fmt::Debug
  {
    match self.collection.find_one(doc!{"id": &id}, None) {
      Ok(maybe_document) => match maybe_document {
          Some(doc) => match from_bson::<T>(Bson::Document(doc.to_owned())) {
              Ok(r) => return Some(r),
              Err(err) => {
                  println!("Could not unserialize document with id {} in {} collection: {} ", id, self.collection.name(), err);
                  warn!("Could not unserialize document with id {} in {} collection: {} ", id, self.collection.name(), err);
                  return None
              },
          },
          None => {
              println!("Could not find any document for id {} in {} collection", id, self.collection.name());
              warn!("Could not find any document for id {} in {} collection", id, self.collection.name());
              None
          }, 
      },
      Err(err) => {
          println!("Could not query any document with id {} in {} collection: {} ", id, self.collection.name(), err);
          warn!("Could not query any document with id {} in {} collection: {} ", id, self.collection.name(), err);
          None
      }
    }
  }

  // save of MongoCollection struct performs a 
  // replace_one operation on a MongoDB collection
  fn save(&self, id: String, entity: T)
  where T: Serialize {
    // secure the json serialization of the entity
    let doc = match unwrap_bson(to_bson(&entity)) {
      Ok(doc) => doc,
      Err(err) => {
        error!("Err save: {}", err);
        return
      },
    };
    // Actually performs replace_one operation on MongoDB Collection
    match self.collection.replace_one(
        doc!{"id": id},
        doc, 
        ReplaceOptions::builder().upsert(true).build(),
    ) {
        Err(err) => error!("Err save: {}", err),
        _ => (),
    };
  }

  fn insert(&self, entity: T)
  where T: Serialize {
    let doc = match unwrap_bson(to_bson(&entity)) {
      Ok(doc) => doc,
      Err(err) => {
        error!("Err insert: {}", err);
        return
      },
    };
    // Actually performs replace_one operation on MongoDB Collection
    match self.collection.insert_one(doc, None) {
        Err(err) => error!("Err insert: {}", err),
        _ => (),
    };
  }
}



// unwrap_bson secures the serialization of the entity
// and the unwrapping of the underlying document
fn unwrap_bson(bson: Result<Bson, Error>) -> Result<Document, String> {
  match bson {
    Ok(b) => match b.as_document() {
      Some(d) => return Ok(d.to_owned()),
      None => Err("Nothing to serde I guess".into()),
    },
    Err(err) => Err(err.to_string()),
  }
}