use tokio::{
    runtime::Runtime,
    time::{interval, Duration, MissedTickBehavior}
};
use crate::config::TIMER_DURATION;

struct Task<F>
    where F: Fn() + ?Sized
{
    name: String,
    closure: F
}

struct Timer {
    obj: Runtime,
    duration: u64
}

struct Daemon{
    tasks: Vec<Box<Task<dyn Fn()>>>,
    timer: Box<Timer>
}

type RefTask = Box<Task<dyn Fn()>>;
type ResponseType = Result<(), String>;

trait Cronie {
    fn new() -> Self;
    fn append_task(&mut self, task: RefTask) -> ResponseType;
    fn srch_task(&self, task_name: &String) -> Option<usize>;
    fn rm_task(&mut self, task: RefTask) -> ResponseType;
    fn update_duration(&mut self, duration: u64) -> ResponseType;
    fn start(&self) -> ResponseType;
    fn stop(&self) -> ResponseType;
}

impl Cronie for Daemon {
    fn new() -> Self {
        let timer = Timer {
            obj: Runtime::new().unwrap(),
            duration: TIMER_DURATION
        };
        Daemon {
            tasks: Vec::new(),
            timer: Box::new(timer)
        }
    }

    fn append_task(&mut self, task: RefTask) -> ResponseType {
        self.tasks.push(task);
        Ok(())
    }

    fn srch_task(&self, task_name: &String) -> Option<usize> {
        self.tasks.iter().position(
            |task| task.name == *task_name 
        )
    }

    fn rm_task(&mut self, task: RefTask) -> ResponseType {
        let index = self.srch_task(&task.name);
        match index {
            None => return Err(format!("Task(named: {}) doesn't exist!", task.name)),
            Some(ind) => {
                // may to stop task first
                let _ = self.tasks.remove(ind);
                Ok(())
            }
        }
    }

    fn update_duration(&mut self, duration: u64) -> ResponseType {
        let timer = self.timer.as_mut();
        timer.duration = duration; 
        Ok(())
    }

    fn start(&self) -> ResponseType {
        let rt = &self.timer.obj;
        let task_iter = self.tasks.iter();
        rt.block_on(async {
            // let start = Instant::now();
            let dur = Duration::from_secs(TIMER_DURATION);
            let mut intv = interval(dur);
            intv.set_missed_tick_behavior(MissedTickBehavior::Delay);

            for task_ref in task_iter {
                let task = &task_ref.closure;
                intv.tick().await;
                task();
            }
        });
        Ok(())
    }

    fn stop(&self) -> ResponseType {
        todo!()
    }
}