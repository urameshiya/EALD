use std::thread;
use std::sync::Mutex;

trait Scheduler {
  type R: Send + 'static;

  fn run_for_each<T, Iter, F>(items: Iter, f: F)
    where Iter: Iterator<Item = T>,
          F: Fn(T) -> Self::R + Send + 'static + Copy,
          T: Send + 'static;
}

struct ParallelScheduler<R> where R: Send + 'static {
  handles: Mutex<Vec<thread::JoinHandle<R>>>
}

impl<R: Send + 'static> Scheduler for ParallelScheduler<R> {
  type R = R;

  fn run_for_each<T, Iter, F>(items: Iter, f: F)
    where Iter: Iterator<Item = T>,
          F: Fn(T) -> R + Send + 'static + Copy,
          T: Send + 'static,
  {
    for work in items {
      thread::spawn(move || f(work));
    }
  }
}

struct SameThreadScheduler<T: Send + 'static> {
  data: std::marker::PhantomData<T>
}

impl<U: Send + 'static> Scheduler for SameThreadScheduler<U> {
  type R = U;

  fn run_for_each<T, Iter, F>(items: Iter, f: F)
  where Iter: Iterator<Item = T>,
        F: Fn(T) -> Self::R + Send + 'static + Copy,
        T: Send + 'static,
  {
    for work in items {
      f(work);
    }
  }
}