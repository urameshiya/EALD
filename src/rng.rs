
pub enum RngNode<T> {
  End,
  Always(RngAction<T>),
  Two(RngInstance<T>, RngInstance<T>),
  Label(String, Box<Self>),
}

type RngAction<T> = Box<dyn FnOnce(&mut T) -> RngNode<T> + Send>;

pub struct RngInstance<T> {
  chance: f32,
  action: RngAction<T>,
}

impl<T> RngInstance<T> {
  pub fn run_action(self, ss: &mut T) -> RngNode<T> {
    (self.action)(ss)
  }
}

// constructors
impl<T> RngNode<T> {
  pub fn always(f: impl FnOnce(&mut T) -> RngNode<T> + Send + 'static) -> Self {
    RngNode::Always(Box::new(f))
  }

  pub fn branch_two(
    chance: f32,
    f: impl FnOnce(&mut T) -> RngNode<T> + Send + 'static,
  ) -> RngNodeNeedOne<T> {
    RngNodeNeedOne(RngInstance {
      chance,
      action: Box::new(f),
    })
  }
}

use RngNode::*;

impl<T: 'static> RngNode<T> {
  pub fn then(self, f: impl FnOnce(&mut T) -> RngNode<T> + Send + Clone + 'static) -> RngNode<T> {
    match self {
      End => RngNode::always(move |ss| f(ss)),
      Always(action) => RngNode::always(move |ss| {
        action(ss).then(f)
      }),
      Two(first, second) => Two(then_transform(first, f.clone()), then_transform(second, f)),
      Label(label, next) => Label(label, Box::new(next.then(f)))
    }
  }

  pub fn set_label(self, label: String) -> RngNode<T> {
    Label(label, Box::new(self))
  }
}

fn then_transform<T: 'static>(
  mut instance: RngInstance<T>,
  then: impl FnOnce(&mut T) -> RngNode<T> + Send + Clone + 'static,
) -> RngInstance<T> {
  let old = instance.action;
  instance.action = Box::new(move |ss| {
    old(ss).then(then)
  });
  instance
}

pub struct RngNodeNeedOne<T>(RngInstance<T>);

impl<T> RngNodeNeedOne<T> {
  pub fn or(self, f: impl FnOnce(&mut T) -> RngNode<T> + Send + 'static) -> RngNode<T> {
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