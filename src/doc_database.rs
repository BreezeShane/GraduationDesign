use data_encoding::HEXUPPER;
use serde::{Serialize, Deserialize};
use std::{fs::{self, File}, io::{BufRead, BufReader, Write}, path::PathBuf};
use crate::config::{
    QUEUE_MAX_LENGTH, 
    QUEUE_STORED_PATH, 
    DATASETS_STORED_PATH, 
};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dataset {
    pub name: String,
    pub timestamp: i64,
    pub available: bool
}
pub type DatasetVec = Vec<Dataset>;


pub trait DatasetTrait {
    fn init_vec() -> DatasetVec;
    fn load() -> DatasetVec;
    fn save(&self) -> Result<usize, std::io::Error>;
    fn append_dset(&mut self, dataset: Dataset) -> Result<(), String>;
    fn rm_dset(&mut self, dataset_name: &str) -> Result<(), String>;
    fn xch_stat(&mut self, dataset_name: &str) -> Result<bool, String>;
    fn srch(&self, dataset_name: &str) -> Option<usize>;
}

impl DatasetTrait for DatasetVec {
    fn init_vec() -> DatasetVec {
        Vec::new()
    }

    fn save(&self) -> Result<usize, std::io::Error> {
        let json_encoded = HEXUPPER.encode(serde_json::to_string(self).unwrap().as_bytes());

        let mut file = File::create(DATASETS_STORED_PATH).unwrap();
        file.write(json_encoded.as_bytes())
    }

    fn append_dset(&mut self, dataset: Dataset) -> Result<(), String> {
        self.push(dataset);
        Ok(())
    }

    fn load() -> DatasetVec {
        let file = File::open(DATASETS_STORED_PATH).unwrap();
        let buffered = BufReader::new(file);
        
        let mut data_loaded = String::new();
        for data_stream in buffered.lines() {
            let line = data_stream.map_err(|err| err.to_string()).unwrap();
            data_loaded.push_str(line.as_str());
        }
        
        let data_vec_decoded: Vec<u8> = match HEXUPPER.decode(data_loaded.as_bytes()) {
            Ok(data_vec) => data_vec,
            Err(_) => vec![]
        };
        let data_string = 
            String::from_utf8(data_vec_decoded).unwrap();
        let dataset_vec = match serde_json::from_str(&data_string.as_str()) {
            Ok(data_json) => data_json,
            Err(_) => DatasetVec::new()
        };
        dataset_vec
    }

    fn srch(&self, dataset_name: &str) -> Option<usize> {
        self.iter().position(
            |dataset| dataset.name == dataset_name
        )
    }

    fn xch_stat(&mut self, dataset_name: &str) -> Result<bool, String> {
        let index_opt = self.srch(dataset_name);
        match index_opt {
            None => return Err(format!("Dataset: {} does not exist!", dataset_name)),
            Some(index) => {
                let dataset: &mut Dataset = self.get_mut(index).unwrap();
                let new_status = !dataset.available;
                dataset.available = new_status;
                Ok(new_status)
            }
        }
    }
    
    fn rm_dset(&mut self, dataset_name: &str) -> Result<(), String> {
        let index_opt = self.srch(dataset_name);
        match index_opt {
            None => return Err(format!("Dataset: {} does not exist!", dataset_name)),
            Some(index) => {
                let _ = self.remove(index);
                let path_buf = PathBuf::from(DATASETS_STORED_PATH).join(dataset_name);
                let result = fs::remove_file(path_buf);
                match result {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(err.to_string())
                }
            }
        }
        
        
        
    }
}

/// Datasets ⮥
/// 
/// Training Queue ↴

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrainingTask {
    pub pic_path: String,
    pub label: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    head: usize,
    tail: usize,
    count: i8,
    queue: [Option<TrainingTask>; QUEUE_MAX_LENGTH]
}

const QUEUE_NONE_VALUE: Option<TrainingTask> = None;

pub trait QueueTrait {
    fn init_queue() -> Queue;
    fn is_empty(&self) -> bool;
    fn is_full(&self) -> bool;
    fn append_task(&mut self, train_unit: TrainingTask) -> Result<(), String>;
    fn pop(&mut self) -> Result<TrainingTask, String>;
    fn save(&self) -> Result<usize, std::io::Error>;
    fn load() -> Queue;
}

impl QueueTrait for Queue {
    fn init_queue() -> Queue {
        Queue {
            head: 0,
            tail: 0,
            count: 0,
            queue: [QUEUE_NONE_VALUE; QUEUE_MAX_LENGTH]
        }
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn is_full(&self) -> bool {
        self.count == (QUEUE_MAX_LENGTH as i8)
    }

    fn append_task(&mut self, train_unit: TrainingTask) -> Result<(), String> {
        if self.is_full() {
            return Err("Queue is full!".to_string())
        }
        self.queue[self.tail] = Some(train_unit);
        self.tail = (self.tail + 1) % QUEUE_MAX_LENGTH;
        self.count += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<TrainingTask, String> {
        if self.is_empty() {
            return Err("Queue is empty! There is no data in queue!".to_string());
        }
        let result = self.queue[self.head].clone().unwrap();
        self.head = (self.head + 1) % QUEUE_MAX_LENGTH;
        self.count -= 1;
        Ok(result)
    }

    fn save(&self)  -> Result<usize, std::io::Error> {
        let json_encoded = HEXUPPER.encode(serde_json::to_string(self).unwrap().as_bytes());

        let mut file = File::create(QUEUE_STORED_PATH).unwrap();
        file.write(json_encoded.as_bytes())
    }

    fn load() -> Queue {
        let file = File::open(QUEUE_STORED_PATH).unwrap();
        let buffered = BufReader::new(file);
    
        let mut data_loaded = String::new();
        for data_stream in buffered.lines() {
            let line = data_stream.map_err(|err| err.to_string()).unwrap();
            data_loaded.push_str(line.as_str());
        }

        let data_vec_decoded: Vec<u8> = match HEXUPPER.decode(data_loaded.as_bytes()) {
            Ok(data_vec) => data_vec,
            Err(_) => vec![]
        };
        let data_string = 
            String::from_utf8(data_vec_decoded).unwrap();
        
        let queue = match serde_json::from_str(&data_string.as_str()) {
            Ok(data_json) => data_json,
            Err(_) => Queue::init_queue()
        };
        queue
    }
}

