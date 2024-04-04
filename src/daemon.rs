use tokio::{
    runtime::Runtime,
    time::{interval, Duration, MissedTickBehavior}
};
use crate::config::TIMER_DURATION;

pub struct Task<F>
    where F: Fn() + ?Sized
{
    name: String,
    closure: F
}

impl<F> Task<F> 
    where F: Fn()
{
    pub fn new(name: &str, closure: F) -> Box<Self> {
        let name = name.to_string();
        Box::new(Task {
            name,
            closure
        })
    }
}

pub struct Timer {
    obj: Runtime,
    duration: u64
}


pub type RefTimer = Box<Timer>;
pub type RefTask = Box<Task<dyn Fn()>>;
pub type Daemon = Vec< (RefTimer, RefTask) >;
type ResponseType = Result<(), String>;

pub trait Cronie {
    fn new() -> Self;
    fn append_task(&mut self, task: RefTask) -> ResponseType;
    fn srch_task(&self, task_name: &String) -> Option<usize>;
    fn rm_task(&mut self, task_name: &String) -> ResponseType;
    fn update_duration(&mut self, task_name: &String, duration: u64) -> ResponseType;
    fn start(&self) -> ResponseType;
}

impl Cronie for Daemon {
    fn new() -> Self {
        Vec::new()
    }

    fn append_task(&mut self, task: RefTask) -> ResponseType {
        let timer = Box::new(Timer {
            obj: Runtime::new().unwrap(),
            duration: TIMER_DURATION
        });
        self.push((timer, task));
        Ok(())
    }

    fn srch_task(&self, task_name: &String) -> Option<usize> {
        self.iter().position(
            |(_, task)| task.name == *task_name 
        )
    }

    fn rm_task(&mut self, task_name: &String) -> ResponseType {
        let index = self.srch_task(&task_name);
        match index {
            None => return Err(format!("Task(named: {}) doesn't exist!", task_name)),
            Some(ind) => {
                let _ = self.remove(ind);
                Ok(())
            }
        }
    }

    fn update_duration(&mut self, task_name: &String, duration: u64) -> ResponseType {
        let index = self.srch_task(task_name);
        match index {
            None => return Err(format!("Task(named: {}) doesn't exist!", task_name)),
            Some(i) => {
                let timer = self.get_mut(i).unwrap().0.as_mut();
                timer.duration = duration; 
            }
        }
        Ok(())
    }

    fn start(&self) -> ResponseType {
        let tuples = self.iter();
        for tuple in tuples {
            let rt = &tuple.0.obj;
            let duration = tuple.0.duration;
            let task = &tuple.1.closure;
            
            rt.block_on(async {
                // let start = Instant::now();
                let dur = Duration::from_secs(duration);
                let mut intv = interval(dur);
                intv.set_missed_tick_behavior(MissedTickBehavior::Delay);
    
                intv.tick().await;
                task();
            });
        }
        Ok(())
    }
}

