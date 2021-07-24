use crate::{
  rng::*,
  battle::*
};

#[derive(Copy, Clone)]
pub enum Targeting {
  SelfAOE,
  EnemyAOE,
  SelfSingle,
  EnemySingle,
  HighestAtk,
  HighestCR,
}

impl Targeting {
  pub fn get_target(&self, ss: &BattleSnapshot, src: HeroID) -> Vec<HeroID> {
    use Targeting::*;

    match self {
      SelfSingle => vec![src],
      SelfAOE => vec![],
      _ => vec![]
    }
  }
}

pub enum DualAtkTarget {
  NoDualAtk,
  Random,
  HighestAtk
}

struct DualAtk {
  chance: f32,
  target: DualAtkTarget
}

pub struct Skill {
  dual_atk_target: DualAtkTarget,
  pub components: Vec<SkillComponent>
}

pub struct SkillComponent {
  pub activate_condition: Option<fn(&BattleSnapshot, &Hero, &Hero) -> bool>,
  pub action: SkillAction,
  pub targeting: Targeting
}

struct DamageAction {
  pow: f32,
  atk_rate: f32,
  def_pen: f32,
}

impl DamageAction {
  fn pow(mut self, pow: f32) -> Self {
    self.pow = pow;
    self
  }

  fn atk_rate(mut self, atk_rate: f32) -> Self {
    self.atk_rate = atk_rate;
    self
  }
}

impl From<DamageAction> for SkillAction {
  fn from(a: DamageAction) -> Self {
    SkillAction::Damage(a)
  }
}

pub enum SkillAction {
  Effect { chance: f32, effect: Effect, duration: u32},
  Damage(DamageAction),
  Splash { atk_rate: f32, def_pen: f32},
  None
}

impl SkillAction {
  fn damage() -> DamageAction {
    DamageAction { pow: 1.0, atk_rate: 1.0, def_pen: 0.0 }
  }
}

impl Skill {
  fn new_component(&mut self, action: SkillAction) -> SkillComponentBuilder {
    let comp = SkillComponent { activate_condition: None, action: SkillAction::None, targeting: Targeting::EnemySingle };
    self.components.push(comp);
    let builder = SkillComponentBuilder { component: self.components.last_mut().unwrap() };
    builder
  }
}

pub struct SkillComponentBuilder<'a> {
  component: &'a mut SkillComponent,
}

impl<'a> SkillComponentBuilder<'a> {
  pub fn activate_condition(&mut self, f: fn(&BattleSnapshot, &Hero, &Hero) -> bool) -> &mut Self {
    self.component.activate_condition = Some(f);
    self
  }
}

