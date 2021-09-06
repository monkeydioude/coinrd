use mongodb::{bson::{Bson, doc, from_bson, to_bson}, options::ReplaceOptions, sync::Database as MongoDatabase};
use serde::{Serialize, Deserialize};
use log::{warn, error};

pub trait Database {
  fn find_one<T>(self: &Self, id: String, c: &str) -> Option<T>
  where T: for<'de> Deserialize<'de> + std::fmt::Debug;

  fn save<T>(self: &Self, id: String, entity: T, coll: &str)
  where T: Serialize;
}

pub struct MongoDB {
  db: MongoDatabase,
}

impl MongoDB {
  pub fn new (db: MongoDatabase) -> MongoDB {
    MongoDB {
      db,
    }
  }
  pub fn to_mongo_db(&self) -> MongoDatabase {
    self.db.to_owned()
  }
}

impl Database for &MongoDB {
  fn find_one<T>(&self, id: String, coll: &str) -> Option<T>
  where T: for<'de> Deserialize<'de> + std::fmt::Debug
  {
    match self.db.collection(coll).find_one(doc!{"id": &id}, None) {
      Ok(maybe_document) => match maybe_document {
          Some(doc) => match from_bson::<T>(Bson::Document(doc.to_owned())) {
              Ok(r) => return Some(r),
              Err(err) => {
                  println!("Could not unserialize document with id {} in {} collection: {} ", id, coll, err);
                  warn!("Could not unserialize document with id {} in {} collection: {} ", id, coll, err);
                  return None
              },
          },
          None => {
              println!("Could not find any document for id {} in {} collection", id, coll);
              warn!("Could not find any document for id {} in {} collection", id, coll);
              None
          }, 
      },
      Err(err) => {
          println!("Could not query any document with id {} in {} collection: {} ", id, coll, err);
          warn!("Could not query any document with id {} in {} collection: {} ", id, coll, err);
          None
      }
    }
  }

  fn save<T>(&self, id: String, entity: T, coll: &str)
  where T: Serialize {
    match self.db.collection(coll).replace_one(
        doc!{"id": id},
        to_bson(&entity).unwrap().as_document().unwrap().to_owned(), 
        ReplaceOptions::builder().upsert(true).build(),
    ) {
        Err(err) => error!("Err save_latest_entries: {}", err),
        _ => (),
    };
  }
}