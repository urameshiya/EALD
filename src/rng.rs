
pub enum RngNode<'r, T> {
  End,
  Always(RngAction<'r, T>),
  Two(RngInstance<'r, T>, RngInstance<'r, T>),
  Label(String, Box<Self>),
}

type RngAction<'r, T> = Box<dyn FnOnce(&mut T) -> RngNode<'r, T> + Send + 'r>;

pub struct RngInstance<'r, T> {
  chance: f32,
  action: RngAction<'r, T>,
}

impl<'r, T> RngInstance<'r, T> {
  pub fn run_action(self, ss: &mut T) -> RngNode<'r,T> {
    (self.action)(ss)
  }
}

// constructors
impl<'r, T: 'r> RngNode<'r, T> {
  pub fn always(f: impl FnOnce(&mut T) -> Self + Send + 'r) -> Self {
    RngNode::Always(Box::new(f))
  }

  pub fn branch_two(
    chance: f32,
    f: impl FnOnce(&mut T) -> Self + Send + 'r,
  ) -> RngNodeNeedOne<'r, T> {
    RngNodeNeedOne(RngInstance {
      chance,
      action: Box::new(f),
    })
  }

  // Create a sequence of 'then' nodes from each element of arr.
  pub fn for_each<'a, VecElement, Element>(
    arr: &'a Vec<VecElement>,
    f: impl FnOnce(&mut T, Element) -> Self + Send + Copy + 'r
  ) -> Self
    where 'r: 'a,
          VecElement: RngLifetimePreservable<'a, 'r, Element>,
          Element: Send + Clone + 'r
  {
    let mut rng = RngNode::End;
    for item in arr {
      let f = f;
      let item = item.preserve_lifetime();
      rng = rng.then(move |ss| {
        f(ss, item)
      });
    }
    rng
  }
}

pub trait RngLifetimePreservable<'source, 'target, Target> {
  fn preserve_lifetime(&'source self) -> Target where Target: 'target;
}

impl<'s, 't, T> RngLifetimePreservable<'s, 't, &'t T> for T where 's: 't {
  fn preserve_lifetime(&'s self) -> &'t T {
    self
  }
}

impl<'s, 't, T> RngLifetimePreservable<'s, 't, T> for T where T: Copy {
  fn preserve_lifetime(&'s self) -> T {
    *self
  }
}

// trait RngLifetimePreservable<T: ?Sized> {
//   fn preserve_lifetime<'original, 'target>(original: &'original T) -> T;
// }

// impl<T> RngLifetimePreservable<T> for &'_ T where T: ?Sized {
//   fn preserve_lifetime<'original, 'target>(original: &'original T) -> T {
//     *original
//   }
// }

// impl<T> RngLifetimePreservable<T> for T where T: Copy {
//   fn preserve_lifetime<'o, 't>(original: &'o T) -> &'t T {

//   }
// }

// trait Borrow<Borrowed: ?Sized> {
// }

// impl<T: ?Sized> Borrow<T> for T {

// }

// impl<T: ?Sized> Borrow<T> for &T {

// }

use RngNode::*;

impl<'r, T: 'r> RngNode<'r, T> {
  pub fn then(self, f: impl FnOnce(&mut T) -> Self + Send + Clone + 'r) -> Self  {
    match self {
      End => RngNode::always(move |ss| f(ss)),
      Always(action) => RngNode::always(move |ss| {
        action(ss).then(f)
      }),
      Two(first, second) => Two(then_transform(first, f.clone()), then_transform(second, f)),
      Label(label, next) => Label(label, Box::new(next.then(f)))
    }
  }

  pub fn set_label(self, label: String) -> Self {
    Label(label, Box::new(self))
  }
}

fn then_transform<'r, T: 'r>(
  mut instance: RngInstance<'r, T>,
  then: impl FnOnce(&mut T) -> RngNode<'r, T> + Send + Clone + 'r,
) -> RngInstance<'r, T> {
  let old = instance.action;
  instance.action = Box::new(move |ss| {
    old(ss).then(then)
  });
  instance
}

pub struct RngNodeNeedOne<'r, T>(RngInstance<'r, T>);

impl<'r, T> RngNodeNeedOne<'r, T> {
  pub fn or(self, f: impl FnOnce(&mut T) -> RngNode<'r, T> + Send + 'r) -> RngNode<'r, T> {
    let chance = 1.0 - self.0.chance;
    RngNode::Two(
      self.0,
      RngInstance {
        chance,
        action: Box::new(f),
      },
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use RngNode::*;

  #[test]
  fn test_structure() {
    let node = RngNode::always(|i: &mut i32| { // i = 0
      *i += 2; // 2
      RngNode::branch_two(0.32, |u| {
        *u *= 2; // 4
        RngNode::always(|z| {
          *z += 3; // 7
          RngNode::End
        })
      }).or(|u| {
        *u *= 3;
        RngNode::End
      }).then(|u| {
        *u -= 2;
        RngNode::End
      })
    });

    let mut i = 0;
    if let RngNode::Always(action) = node {
      if let RngNode::Two(a1, a2) = action(&mut i) { // i += 2
        assert_eq!(i, 2);
        assert_eq!(a1.chance, 0.32f32);
        assert_eq!(a2.chance, 1.0 - 0.32);

        let mut i1 = i; // i = 2
        if let RngNode::Always(a) = (a1.action)(&mut i1) { // i *= 2
          assert_eq!(i1, 4);
          if let Always(a) = a(&mut i1) { // i += 3
            assert_eq!(i1, 7); // then -2
            if let End = a(&mut i1) { // i -= 2
              assert_eq!(i1, 5);
            } else { assert!(false) }
          } else { assert!(false) }
        } else { assert!(false) }

        let mut i2 = i; // i = 2
        if let Always(a) = (a2.action)(&mut i2) { // i *= 3
          assert_eq!(i2, 6);
          if let End = a(&mut i2) { // i -= 2
            assert_eq!(i2, 4);
          } else { assert!(false) }
        } else { assert!(false) }
      } else { assert!(false) }
    } else { assert!(false) }
  }

}