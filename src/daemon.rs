use crate::config::TIMER_DURATION;
use std::collections::HashMap;
use tokio::{
    runtime::Runtime,
    time::{interval, Duration, MissedTickBehavior}
};

type ResponseType = Result<(), String>;
type ClosureType = dyn Fn() -> ResponseType;
pub type RefTimer = Box<Timer<ClosureType>>;
pub type Daemon = HashMap<String, RefTimer>;
// type AsyncFn = Box<dyn Fn(&Object<Manager>) -> Pin<Box<dyn Future<Output = ResponseType>>>>;

pub struct Timer<T>
    where T: Fn()->ResponseType + ?Sized
{
    runtime: Runtime,
    duration: u64,
    task: Box<T>
}

pub trait Cronie {
    fn new() -> Self;
    fn append_task(&mut self, task_name: &str, task: Box<ClosureType>) -> ResponseType;
    fn rm_task(&mut self, task_name: &String) -> ResponseType;
    fn update_duration(&mut self, task_name: &String, duration: u64) -> ResponseType;
    fn start(&self) -> ResponseType;
}

impl Cronie for Daemon {
    fn new() -> Self {
        HashMap::new()
    }

    fn append_task(&mut self, task_name: &str, task: Box<ClosureType>) -> ResponseType {
        let timer = Box::new(Timer {
            runtime: Runtime::new().unwrap(),
            duration: TIMER_DURATION,
            task
        });
        self.insert(task_name.to_string(), timer);
        Ok(())
    }

    fn rm_task(&mut self, task_name: &String) -> ResponseType {
        if self.contains_key(task_name) {
            self.remove(task_name);
            return Ok(());
        } else {
            return Err(format!("Task: {task_name} doesn't exist!"));
        }
    }

    fn update_duration(&mut self, task_name: &String, duration: u64) -> ResponseType {
        if self.contains_key(task_name) {
            let timer = self.get_mut(task_name).unwrap();
            timer.duration = duration;
            return Ok(());
        } else {
            return Err(format!("Task: {task_name} doesn't exist!"));
        }
    }

    // async fn __exec_raw(closure: &F, pool: &Object<Manager>) -> ResponseType {
    //     closure(pool);
    //     todo!()
    // }

    fn start(&self) -> ResponseType {
        let timers = self.values();
        for timer in timers {
            let rt = &timer.runtime;
            let duration = timer.duration;
            let task = &timer.task;
            
            let _ = rt.block_on(async {
                // let start = Instant::now();
                let dur = Duration::from_secs(duration);
                let mut intv = interval(dur);
                intv.set_missed_tick_behavior(MissedTickBehavior::Delay);
    
                intv.tick().await;
                let status = task();
                
                if let Err(e) = status {
                    return Err(e);
                }
                Ok(())
            });
        }
        Ok(())
    }
}

