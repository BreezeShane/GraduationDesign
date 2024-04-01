use data_encoding::HEXUPPER;
use serde::{Serialize, Deserialize};
use std::{fs::File, io::{BufRead, BufReader, Write}};

const QUEUE_STORED_PATH: &str = "./queue.db";
const DATASETS_STORED_PATH: &str = "./datasets.db";
const QUEUE_MAX_LENGTH: usize = 10;

#[derive(Serialize, Deserialize)]
struct Dataset {
    name: String,
    timestamp: i64,
    available: bool
}
type DatasetVec = Vec<Dataset>;

// const DATASET_NONE_VALUE: Option<Dataset> = None;

trait DatasetTrait {
    fn init_vec() -> DatasetVec;
    fn load<
        T: for<'a> Into< Option<&'a str> >
        >(file_path: T) -> Result<DatasetVec, String>;
    fn save(&self) -> Result<usize, std::io::Error>;
    fn append(&mut self, dataset: Dataset) -> Result<(), String>;

    fn search(vec: &mut DatasetVec, dataset_name: &str) -> Option<usize>;
    fn en_disable(vec: &mut DatasetVec, dataset_name: &str) -> Result<bool, String>;
    fn remove(vec: &mut DatasetVec, dataset_name: &str) -> Result<Dataset, String>;
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

    fn append(&mut self, dataset: Dataset) -> Result<(), String> {
        self.push(dataset);
        Ok(())
    }

    fn load<
        T: for<'a> Into< Option<&'a str> >
    >(file_path: T) -> Result<DatasetVec, String>{
        let file = 
            File::open(
                file_path.into()
                .unwrap_or(DATASETS_STORED_PATH)
            )
            .map_err(|err| err.to_string())?;

        let buffered = BufReader::new(file);

        let mut data_loaded = String::new();
        for data_stream in buffered.lines() {
            let line = data_stream.map_err(|err| err.to_string())?;
            data_loaded.push_str(line.as_str());
        }

        let data_vec_decoded = 
            HEXUPPER.decode(data_loaded.as_bytes()).unwrap();
        let data_string = 
            String::from_utf8(data_vec_decoded).unwrap();
        
        let dataset_vec: DatasetVec = 
            serde_json::from_str(&data_string.as_str())
                .map_err(|err| err.to_string())?;
        Ok(dataset_vec)
    }

    fn search(vec: &mut DatasetVec, dataset_name: &str) -> Option<usize> {
        vec.iter().position(
            |dataset| dataset.name == dataset_name
        )
    }

    fn en_disable(vec: &mut DatasetVec, dataset_name: &str) -> Result<bool, String> {
        let index_opt = Self::search(vec, dataset_name);
        match index_opt {
            None => return Err(format!("Dataset: {} does not exist!", dataset_name)),
            Some(index) => {
                let dataset = vec.get_mut(index).unwrap();
                let new_status = !dataset.available;
                dataset.available = new_status;
                Ok(new_status)
            }
        }
    }
    
    fn remove(vec: &mut DatasetVec, dataset_name: &str) -> Result<Dataset, String> {
        let index_opt = Self::search(vec, dataset_name);
        match index_opt {
            None => return Err(format!("Dataset: {} does not exist!", dataset_name)),
            Some(index) => return Ok(vec.remove(index))
        }
    }
}



/// Datasets ⮥
/// 
/// Training Queue ↴

#[derive(Serialize, Deserialize, Clone)]
pub struct TrainingUnit {
    // id: i8,
    pic_path: String,
    label: String
}

#[derive(Serialize, Deserialize)]
pub struct Queue {
    head: usize,
    tail: usize,
    count: i8,
    queue: [Option<TrainingUnit>; QUEUE_MAX_LENGTH]
}

const QUEUE_NONE_VALUE: Option<TrainingUnit> = None;

trait QueueTrait {
    fn init_queue() -> Queue;
    fn is_empty(&self) -> bool;
    fn is_full(&self) -> bool;
    fn append(&mut self, train_unit: TrainingUnit) -> Result<(), String>;
    fn pop(&mut self) -> Result<TrainingUnit, String>;
    fn save(&self) -> Result<usize, std::io::Error>;
    fn load<T: for<'a> Into<Option<&'a str>>>(file_path: T) -> Result<Queue, String>;
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

    fn append(&mut self, train_unit: TrainingUnit) -> Result<(), String> {
        if self.is_full() {
            return Err("Queue is full!".to_string())
        }
        self.queue[self.tail] = Some(train_unit);
        self.tail = (self.tail + 1) % QUEUE_MAX_LENGTH;
        self.count += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<TrainingUnit, String> {
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

    fn load<
        T: for<'a> Into<Option<&'a str>>
    >(file_path: T) -> Result<Queue, String> {
        let file = 
            File::open(
                file_path.into()
                .unwrap_or(QUEUE_STORED_PATH)
            )
            .map_err(|err| err.to_string())?;

        let buffered = BufReader::new(file);
    
        let mut data_loaded = String::new();
        for data_stream in buffered.lines() {
            let line = data_stream.map_err(|err| err.to_string())?;
            data_loaded.push_str(line.as_str());
        }

        let data_vec_decoded = 
            HEXUPPER.decode(data_loaded.as_bytes()).unwrap();
        let data_string = 
            String::from_utf8(data_vec_decoded).unwrap();
        
        let queue = 
            serde_json::from_str(&data_string.as_str())
                .map_err(|err| err.to_string())?;
        Ok(queue)
    }
}

