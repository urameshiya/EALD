use threadpool::ThreadPool;
use crate::{
  battle::BattleSnapshot,
  rng::RngNode
};
use std::sync::{ Mutex, Arc };
use lazy_static::lazy_static;

lazy_static! {
  static ref POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(8));
}

pub trait RngObserver<T>: Send {
  fn rng_did_reach_label(&mut self, label: String);
  fn rng_did_reach_end(&mut self, ss: T, depth: i32);
}

pub fn rng_node_run<O, T>(mut node: RngNode<T>, mut ss: T, observer: Arc<Mutex<O>>, depth: i32)
  where O: RngObserver<T> + 'static,
        T: Clone + Send + 'static
{
  use RngNode::*;

  loop {
    match node {
      End => {
        observer.lock().unwrap().rng_did_reach_end(ss, depth);
        return;
      },
      Always(a1) => {
        node = a1(&mut ss);
      },
      Two(a1, a2) => {
        if depth > 10 {
          return
        }
        let mut ss1 = ss.clone();
        let mut ss2 = ss;

        let pool = POOL.lock().unwrap();

        let mut _observer = Arc::clone(&observer);

        pool.execute(move || {
          let next = a1.run_action(&mut ss1);
          rng_node_run(next, ss1, _observer, depth + 1);
        });

        pool.execute(move || {
          let next = a2.run_action(&mut ss2);
          rng_node_run(next, ss2, observer, depth + 1);
        });

        return
      },
      Label(label, next) => {
        observer.lock().unwrap().rng_did_reach_label(label);
        node = *next;
      }
    }
  }
}

pub struct DataCollector {
  snapshots: Vec<(i32, BattleSnapshot)>
}

impl RngObserver<BattleSnapshot> for DataCollector {
  fn rng_did_reach_label(&mut self, label: String) {
    println!("Label {}", label)
  }

  fn rng_did_reach_end(&mut self, ss: BattleSnapshot, depth: i32) {
    println!("{}", ss.heroes[0].stats.hp);
    self.snapshots.push((depth, ss));
  }
}

impl DataCollector {
  pub fn new() -> Self {
    Self {
      snapshots: vec![]
    }
  }
}

mod tests {
  use super::*;
  use std::sync::Condvar;

  struct TestObserver {
    results: Vec<i32>,
    lock: Arc<(Mutex<i32>, Condvar)>
  }

  impl RngObserver<i32> for TestObserver {
    fn rng_did_reach_label(&mut self, label: String) {
      unimplemented!()
    }

    fn rng_did_reach_end(&mut self, ss: i32, depth: i32) {
      self.results.push(ss);
      inc_and_notify(self.lock.clone());
    }
  }

  #[test]
  fn test_multithread() {
    let pair = Arc::new((Mutex::new(0), Condvar::new()));

    let node = RngNode::always(|i: &mut i32| { // i = 0
      *i += 2; // 2
      RngNode::branch_two(0.32, |u| {
        *u *= 2; // 4
        RngNode::always(|z| {
          *z += 3; // 7
          RngNode::End
        })
      }).or(move |u| {
        *u *= 3;
        RngNode::End
      }).then(move |u| {
        *u -= 2;
        RngNode::End
      })
    });

    let observer = TestObserver { results: vec!(), lock: pair.clone() };
    let observer = Arc::new(Mutex::new(observer));

    rng_node_run(node, 0, observer.clone(), 0);

    let (count, cvar) = &*pair;
    let mut count = count.lock().unwrap();
    while *count < 2 {
      count = cvar.wait(count).unwrap();
    }

    let observer = observer.lock().unwrap();
    println!("{:?}", observer.results);
    assert!(observer.results.contains(&5));
    assert!(observer.results.contains(&4));
  }

  fn inc_and_notify(pair: Arc<(Mutex<i32>, Condvar)>) {
    let (count, cvar) = &*pair;
    *count.lock().unwrap() += 1;
    cvar.notify_one();
  }
}