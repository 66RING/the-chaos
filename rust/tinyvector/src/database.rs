use crate::similarity::{get_cache_attr, get_distance_fn, normalize, Distance, ScoreIndex};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info, trace};

lazy_static! {
    /// The path to the database
    pub static ref STORE_PATH: PathBuf = PathBuf::from("./storage/db");
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DBError {
    /// The table already exists
    #[error("The table already exists")]
    TableAlreadyExists,
    /// The table does not exist
    #[error("The table does not exist")]
    TableDoesNotExist,
    /// The record does not exist
    #[error("The record does not exist")]
    RecordDoesNotExist,
    /// The record does exist
    #[error("The record does exist")]
    RecordExist,
    /// Dimension mismatch
    #[error("Dimension mismatch")]
    DimensionMismatch,
}

/// The vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub tables: HashMap<String, Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Dimension of the vectors in the collection
    pub dimension: usize,
    /// Distance metric used for querying
    pub distance: Distance,
    /// Embeddings in the collection
    pub records: Vec<EmbeddingRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    /// Value of the embedding.
    pub id: String,
    /// The embedding.
    pub embedding: Vec<f32>,
}

impl Table {
    pub fn top_k_similarity(&self, query_embedding: &[f32], top_k: usize) -> Vec<EmbeddingRecord> {
        let memo_attr = get_cache_attr(self.distance, query_embedding);
        let distance_fn = get_distance_fn(self.distance);

        // Compute score and corresponding index for each record.
        let scores = self
            .records
            .par_iter()
            .enumerate()
            .map(|(index, record)| {
                let score = distance_fn(&record.embedding, query_embedding, memo_attr);
                ScoreIndex { score, index }
            })
            .collect::<Vec<_>>();

        // Sort the scores by top k binary heap.
        let mut heap = BinaryHeap::new();
        for score in scores {
            if heap.len() < top_k {
                heap.push(score);
            } else {
                if score.score > heap.peek().unwrap().score {
                    heap.pop();
                    heap.push(score);
                }
            }
        }

        heap.into_sorted_vec()
            .into_iter()
            .map(|score_index| self.records[score_index.index].clone())
            .collect::<Vec<_>>()
    }
}

impl Database {
    pub fn create_table(
        &mut self,
        table_name: String,
        dimension: usize,
        distance: Distance,
    ) -> Result<(), DBError> {
        // Check if table already exists.
        if self.tables.contains_key(&table_name) {
            return Err(DBError::TableAlreadyExists);
        }

        // Create new table.
        let table = Table {
            dimension,
            distance,
            records: Vec::new(),
        };
        info!("Create table {:#?}", table);
        self.tables.insert(table_name, table);
        Ok(())
    }

    pub fn insert_record(
        &mut self,
        table_name: String,
        mut record: EmbeddingRecord,
    ) -> Result<(), DBError> {
        // Check if table exists.
        let table = self
            .tables
            .get_mut(&table_name)
            .ok_or(DBError::TableDoesNotExist)?;

        // Check if record already exists.
        if table.records.iter().any(|r| r.id == record.id) {
            return Err(DBError::RecordExist);
        }

        // Check if record has the correct dimension.
        if record.embedding.len() != table.dimension {
            return Err(DBError::DimensionMismatch);
        }

        // Normalize the vector if the distance metric is cosine, so we can use dot product later
        if table.distance == Distance::Cosine {
            record.embedding = normalize(&record.embedding);
        }

        table.records.push(record);
        Ok(())
    }

    pub fn delete_record(&mut self, table_name: String, id: String) -> Result<(), DBError> {
        // Check if table exists.
        let table = self
            .tables
            .get_mut(&table_name)
            .ok_or(DBError::TableDoesNotExist)?;

        // Delete record from table.
        table
            .records
            .iter()
            .position(|r| r.id == id)
            .map(|i| table.records.remove(i))
            .ok_or(DBError::RecordDoesNotExist)?;

        Ok(())
    }

    pub fn drop_table(&mut self, table_name: impl Into<String>) -> Result<(), DBError> {
        if self.tables.remove(&table_name.into()).is_some() {
            Ok(())
        } else {
            Err(DBError::TableDoesNotExist)
        }
    }

    pub fn query_record(
        &mut self,
        table_name: String,
        query_embedding: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<EmbeddingRecord>, DBError> {
        let table = self
            .tables
            .get(&table_name)
            .ok_or(DBError::TableDoesNotExist)?;

        // Check if query embedding has the correct dimension.
        if query_embedding.len() != table.dimension {
            return Err(DBError::DimensionMismatch);
        }

        let instant = Instant::now();
        let result = table.top_k_similarity(&query_embedding, top_k);
        trace!("Query to {table_name} took {:?}", instant.elapsed());
        Ok(result)
    }

    /// TODO: return the entire data base for debug
    pub fn get_entire_db() {}

    pub fn zero() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn load_from_file() -> Result<Self> {
        // Create storage file if file no exist.
        if !STORE_PATH.exists() {
            fs::create_dir_all(STORE_PATH.parent().context("Invalid storage path")?)?;
            // Create a new database.
            return Ok(Self::zero());
        }

        debug!(
            "Loading database from file: {:?}",
            STORE_PATH.as_path().to_str()
        );
        let db_data = fs::read(STORE_PATH.as_path())?;
        let db = bincode::deserialize(&db_data)?;
        Ok(db)
    }

    pub fn save_to_file(&self) -> Result<()> {
        let db_data = bincode::serialize(&self)?;
        fs::write(STORE_PATH.as_path(), db_data)?;
        Ok(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        tracing::debug!("Saving database to store");
        self.save_to_file().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::*;

    #[test]
    fn simple_test_all() {
        let mut db = Database::load_from_file().unwrap();
        let table_name = "table".to_string();
        let dimension = 4;
        let top_k = 10;
        let distance = Distance::DotProduct;
        if let Err(e) = db.query_record(table_name.clone(), vec![1., 2., 3., 4.], top_k) {
            assert_eq!(e, DBError::TableDoesNotExist);

            db.create_table(table_name.clone(), dimension, distance)
                .unwrap();
            let records = db
                .query_record(table_name.clone(), vec![1., 2., 3., 4.], top_k)
                .unwrap();
            assert_eq!(0, records.len());
        }

        // Db reload from file.
        let table_name = "table".to_string();
        let top_k = 10;
        let record = EmbeddingRecord {
            id: "test record".to_string(),
            embedding: vec![1., 2., 3., 4.],
        };
        db.insert_record(table_name.clone(), record).unwrap();

        let records = db
            .query_record(table_name.clone(), vec![1., 2., 3., 4.], top_k)
            .unwrap();
        assert_eq!(1, records.len());
        assert_eq!(vec![1., 2., 3., 4.], records[0].embedding);

        // Delete record
        let table_name = "table".to_string();
        let top_k = 10;
        let id = "test record".to_string();
        let _ = db.delete_record(table_name.clone(), id);

        let records = db
            .query_record(table_name.clone(), vec![1., 2., 3., 4.], top_k)
            .unwrap();
        assert_eq!(0, records.len());

        let table_name = "table".to_string();
        let top_k = 10;
        let _ = db.drop_table(table_name.clone());

        let err = db.query_record(table_name.clone(), vec![1., 2., 3., 4.], top_k);
        assert!(err.is_err());
    }

    #[test]
    fn test_query_openai() {
        // Create database if not exist.
        let mut db = Database::load_from_file().unwrap();
        let table_name = "table2".to_string();
        let dimension = 1536;
        let top_k = 2;
        let distance = Distance::Cosine;
        let _ = db.create_table(table_name.clone(), dimension, distance);

        let _ = db.insert_record(table_name.clone(), get_dog_record());
        let _ = db.insert_record(table_name.clone(), get_cat_record());
        let _ = db.insert_record(table_name.clone(), get_openai_record());

        let query_embedding = get_ml_embedding();

        let records = db
            .query_record(table_name.clone(), query_embedding, top_k)
            .unwrap();
        assert_eq!(top_k, records.len());
        // assert_eq!(vec![1., 2., 3., 4.], records[0].embedding);
        assert_eq!("OpenAI change the world", records[0].id);
        assert_eq!("Cat", records[1].id);
        // OpenAi, Cat, Dog
    }

    #[test]
    fn test_query_Cat() {
        // Create database if not exist.
        let mut db = Database::load_from_file().unwrap();
        let table_name = "table2".to_string();
        let dimension = 1536;
        let top_k = 3;
        let distance = Distance::Cosine;
        let _ = db.create_table(table_name.clone(), dimension, distance);

        let _ = db.insert_record(table_name.clone(), get_dog_record());
        let _ = db.insert_record(table_name.clone(), get_cat_record());
        let _ = db.insert_record(table_name.clone(), get_openai_record());

        let query_embedding = get_cat_embedding();

        let records = db
            .query_record(table_name.clone(), query_embedding, top_k)
            .unwrap();
        assert_eq!(top_k, records.len());
        assert_eq!("Cat", records[0].id);
        assert_eq!("Dog", records[1].id);
        assert_eq!("OpenAI change the world", records[2].id);
    }


    #[test]
    fn test_query_dog() {
        // Create database if not exist.
        let mut db = Database::load_from_file().unwrap();
        let table_name = "table2".to_string();
        let dimension = 1536;
        let top_k = 3;
        let distance = Distance::Cosine;
        let _ = db.create_table(table_name.clone(), dimension, distance);

        let _ = db.insert_record(table_name.clone(), get_dog_record());
        let _ = db.insert_record(table_name.clone(), get_cat_record());
        let _ = db.insert_record(table_name.clone(), get_openai_record());

        let query_embedding = get_dog_embedding();

        let records = db
            .query_record(table_name.clone(), query_embedding, top_k)
            .unwrap();
        assert_eq!(top_k, records.len());
        assert_eq!("Dog", records[0].id);
        assert_eq!("Cat", records[1].id);
        assert_eq!("OpenAI change the world", records[2].id);
    }


    #[test]
    fn test_query_ml() {
        // Create database if not exist.
        let mut db = Database::load_from_file().unwrap();
        let table_name = "table2".to_string();
        let dimension = 1536;
        let top_k = 3;
        let distance = Distance::Cosine;
        let _ = db.create_table(table_name.clone(), dimension, distance);

        let _ = db.insert_record(table_name.clone(), get_dog_record());
        let _ = db.insert_record(table_name.clone(), get_cat_record());
        let _ = db.insert_record(table_name.clone(), get_openai_record());

        let query_embedding = get_ml_embedding();

        let records = db
            .query_record(table_name.clone(), query_embedding, top_k)
            .unwrap();
        assert_eq!(top_k, records.len());
        assert_eq!("OpenAI change the world", records[0].id);
        assert_eq!("Cat", records[1].id);
        assert_eq!("Dog", records[2].id);
    }
}
