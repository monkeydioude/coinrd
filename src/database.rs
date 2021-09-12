use mongodb::{bson::{Bson, doc, from_bson, to_bson}, options::ReplaceOptions, sync::{Collection as MongoColl, Database as MongoDatabase}};
use serde::{Serialize, Deserialize};
use log::{warn, error};

pub trait Collection {
  fn find_one<T>(self: &Self, id: String) -> Option<T>
  where T: for<'de> Deserialize<'de> + std::fmt::Debug;

  fn save<T>(self: &Self, id: String, entity: T)
  where T: Serialize;
}

pub struct MongoDB {
  db: MongoDatabase,
}

pub struct MongoCollection {
  collection: MongoColl,
}

impl MongoDB {
  pub fn new (db: MongoDatabase) -> Self {
    MongoDB {
      db,
    }
  }

  pub fn new_collection(&self, coll: &str) -> MongoCollection {
    MongoCollection {
      collection: self.db.collection(coll),
    }
  }

  pub fn to_mongo_db(&self) -> MongoDatabase {
    self.db.to_owned()
  }
}

impl Collection for MongoCollection {
  fn find_one<T>(&self, id: String) -> Option<T>
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

  fn save<T>(&self, id: String, entity: T)
  where T: Serialize {
    match self.collection.replace_one(
        doc!{"id": id},
        to_bson(&entity).unwrap().as_document().unwrap().to_owned(), 
        ReplaceOptions::builder().upsert(true).build(),
    ) {
        Err(err) => error!("Err save_latest_entries: {}", err),
        _ => (),
    };
  }
}