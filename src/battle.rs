
use std::cmp;
use std::cell::RefCell;

#[derive(Clone)]
struct Hero {
  stats: HeroStats,

  alive: bool,
  readiness: u32,
  effects: Effects,
  team: u32,
}

#[derive(Clone)]
struct HeroStats {
  max_hp: f32,
  hp: f32,
  atk: f32,
  spd: f32,
  def: f32,
  cc: f32,
  cdmg: f32,
  eff: f32,
  effres: f32,
  element: Element,
  hit_chance: f32,
  crit_resist: f32,
}

impl Hero {
  fn add_stat_change(&mut self, kind: StatKind, modifier: f32) {
    *self.member_for_stat(kind) += modifier;
  }

  fn remove_stat_change(&mut self, kind: StatKind, modifier: f32) {
    *self.member_for_stat(kind) -= modifier;
  }

  fn member_for_stat(&mut self, stat: StatKind) -> &mut f32 {
    use StatKind::*;

    match stat {
      ATK => &mut self.stats.atk,
      SPD => &mut self.stats.spd,
      DEF => &mut self.stats.def,
      CC => &mut self.stats.cc,
      CDmg => &mut self.stats.cdmg,
      HitChance => &mut self.stats.hit_chance,
      CritResist => &mut self.stats.crit_resist,
      EFF => &mut self.stats.eff,
      EffRes => &mut self.stats.effres,
    }
  }

  fn apply_effect(&mut self, effect: Effect, duration: u32) {
    let (success, removed) = self.effects.apply(effect, duration);

    if let Some(removed) = removed {
      self.undo_stat_changes(removed)
    };

    if success {
      match effect.get_kind() {
        EffectKind::StatModifier(stat, modf) => self.add_stat_change(stat, modf),
        EffectKind::Simple => ()
      }
    }
  }

  fn reduce_effect_duration(&mut self, num_turns: u32) {
    // let removed = self.effects.reduce_duration_all(num_turns);
    // removed.iter().for_each(|&x| self.on_effect_removed(x));
  }

  fn undo_stat_changes(&mut self, effect: Effect) {
    use EffectKind::*;
    match effect.get_kind() {
      StatModifier(stat, modf) => self.remove_stat_change(stat, modf),
      Simple => ()
    }
  }

  fn consume_effects(&mut self, mut f: impl FnMut(&EffectEntry) -> u32) {
    self.effects.consume_effects(f).iter().for_each(|removed| self.undo_stat_changes(removed.effect));
  }
}

#[derive(Copy, Clone)]
enum Element {
  Fire, Ice, Earth, Light, Dark
}

#[derive(Copy, Clone)]
enum StatKind {
  SPD, ATK, DEF, CC, CDmg, EffRes, EFF, HitChance, CritResist
}

#[derive(Copy, Clone, PartialEq)]
enum Effect {
  AtkBuff,
  AtkDown,
  GreaterAtk,
  DefBreak,
  DefBuff,
  SpdBuff,
  SpdDown,
  CritResist,
  CannotBuff,
  Immunity,
  Unhealable,
  ContinuousHealing,
  Bleed(f32),
  Burn(f32),
  Poison,
  Rage,
  Daydream,
  Invincible,
  SkillNull,
}

enum EffectKind {
  StatModifier(StatKind, f32),
  Simple
}

impl Effect {
  fn get_kind(&self) -> EffectKind {
    use StatKind::*;
    use Effect::*;
    use EffectKind::*;
    match self {
      GreaterAtk => StatModifier(ATK, 0.75),
      AtkBuff => StatModifier(ATK, 0.5),
      DefBuff => StatModifier(DEF, 0.6),
      DefBreak => StatModifier(DEF, -0.7),
      SpdBuff => StatModifier(SPD, 0.3),
      SpdDown => StatModifier(SPD, -0.3),
      Effect::CritResist => StatModifier(StatKind::CritResist, 0.5),
      _ => Simple
    }
  }

  fn can_dispel(&self) -> bool {
    use Effect::*;

    match self {
      Rage | Daydream => false,
      _ => true
    }
  }

  fn can_stack(&self) -> bool {
    use Effect::*;

    match self {
      Burn(_) | Bleed(_) | Poison | ContinuousHealing => true,
      _ => false
    }
  }

  fn get_priority(first: Effect, second: Effect) -> EffectPriority {
    use Effect::*;

    match (first, second) {
      (AtkBuff, GreaterAtk) => EffectPriority::First,
      (GreaterAtk, AtkBuff) => EffectPriority::Second,
      _ => EffectPriority::NoOverwrite
    }
  }
}

enum EffectPriority {
  First,
  Second,
  NoOverwrite
}

#[derive(Clone)]
struct Effects {
  arr: Vec<EffectEntry>,
}

#[derive(Clone)]
struct EffectEntry {
  effect: Effect,
  duration: u32,
}

const EFFECT_MAX_COUNT: usize = 10;

impl Effects {
  fn apply(&mut self, effect: Effect, duration: u32) -> (bool, Option<Effect>) {
    let entry = EffectEntry { effect: effect, duration: duration };
    let eff = self;

    if !effect.can_stack() {
      let exist = eff.arr.iter().position(|entry|
        matches!(Effect::get_priority(effect, entry.effect), EffectPriority::First)
      );

      if let Some(exist) = exist {
        let temp = eff.arr[exist].effect;
        eff.arr[exist] = entry;

        return (true, Some(temp))
      }
    }

    if eff.arr.len() < EFFECT_MAX_COUNT {
      eff.arr.push(entry);

      (true, None)
    } else {
      (false, None)
    }
  }

  fn has_effect(&self, effect: Effect) -> bool {
    self.arr.iter().find(|&x| x.effect == effect).is_some()
  }

  fn get_bleed_effects<'a>(&'a self) -> impl Iterator<Item = &EffectEntry> + 'a {
    self.arr.iter().filter(|x| matches!(x.effect, Effect::Burn(_) | Effect::Bleed(_) | Effect::Poison))
  }

  fn dispel(&mut self, num_debuffs: u32) -> Vec<Effect> {
    let mut i = 0;
    let mut removed = Vec::<Effect>::new();
    while i != self.arr.len() && removed.len() < num_debuffs as usize {
      let entry = &self.arr[i];
      if entry.effect.can_dispel() {
        removed.push(entry.effect);
        self.arr.remove(i);
      } else {
        i += 1;
      }
    }
    removed
  }

  // fn reduce_duration_all(&mut self, num_turns: u32) -> Vec<EffectEntry> {
  //   self.consume_effects(|entry| {
  //     if entry.effect.can_dispel() { num_turns } else { 0 }
  //   })
  // }

  fn consume_effects(&mut self, mut f: impl FnMut(&EffectEntry) -> u32) -> Vec<EffectEntry> {
    let mut removed: Vec<EffectEntry> = Vec::new();
    let mut i = 0;
    while i != self.arr.len() {
      let entry = &mut self.arr[i];
      let turns_consumed = f(entry);
      entry.duration -= turns_consumed;
      if entry.duration < 1 {
          removed.push(self.arr.remove(i));
      } else {
        i += 1;
      }
    }
    removed
  }
}

#[derive(Clone)]
struct BattleSnapshot {
  heroes: Vec<Hero>,
  base_stats: Vec<Hero>
}

type HeroID = usize;

impl BattleSnapshot {
  fn get_turn_hero_id(&self) -> HeroID {
    (0..).zip(&self.heroes)
      .filter(|(_, x)| x.alive)
      .max_by(|(_, x), (_, y)| x.readiness.cmp(&y.readiness))
      .map(|(i, _)| i)
      .unwrap()
  }

  fn get_base_stats_for_hero(&self, id: HeroID) -> &Hero {
    &self.base_stats[id]
  }
}

enum DamageSource<'a> {
  Hero(&'a Hero),
  Effect
}

fn deal_damage(src: DamageSource, target: &mut Hero, dmg: &DamageInstance) -> bool {
  if target.effects.has_effect(Effect::Invincible) {
    return false
  }

  if matches!(src, DamageSource::Hero(_)) {
    let mut has_skill_null = false;
    target.consume_effects(|entry| {
      if matches!(entry.effect, Effect::SkillNull) {
        has_skill_null = true;
        1
      } else { 0 }
    });
    if has_skill_null {
      return false;
    }
  }

  target.stats.hp -= calculate_damage_taken(src, target, dmg);

  if target.stats.hp < 0.0 {
    target.alive = false;
  }
  true
}

fn heal(target: &mut Hero, amount: f32) -> bool {
  if target.effects.has_effect(Effect::Unhealable) {
    return false
  }
  target.stats.hp = target.stats.hp + amount;
  if target.stats.hp > target.stats.max_hp {
    target.stats.hp = target.stats.max_hp;
  }
  true
}

fn calculate_damage_taken(src: DamageSource, target: &Hero, raw_dmg: &DamageInstance) -> f32 {
  0.0
}

struct DamageInstance {
  raw_dmg: f32,
  def_pen: f32
}

fn hero_on_turn_start(hero: &mut Hero) {
  // Apply bleeds, continuous healing
  use Effect::*;

  let mut bleed_dmg = Vec::<DamageInstance>::new();
  let mut auto_heal = 0_f32;

  let hero_hp = hero.stats.max_hp;
  hero.consume_effects(|entry| {
    match entry.effect {
      Burn(dmg) | Bleed(dmg) => {
        bleed_dmg.push(DamageInstance { raw_dmg: dmg, def_pen: 0.7 });
        1
      },
      Poison => {
        bleed_dmg.push(DamageInstance { raw_dmg: hero_hp * 0.05, def_pen: 1.0 });
        1
      },
      ContinuousHealing => {
        auto_heal += hero_hp * 0.15;
        1
      }
      _ => 0,
    }
  });

  let mut total_dmg = 0_f32;
  for dmg in &bleed_dmg {
    total_dmg += calculate_damage_taken(DamageSource::Effect, hero, dmg);
  }
  if total_dmg > auto_heal {
    deal_damage(DamageSource::Effect, hero, &DamageInstance { raw_dmg: total_dmg - auto_heal, def_pen: 1.0});
  } else {
    heal(hero, auto_heal - total_dmg);
  }
}

fn hero_on_turn_end(hero: &Hero) {

}

enum EffectSource<'a> {
  Enemy(&'a Hero),
  SelfTargeted
}

fn apply_effect(src: EffectSource, target: &mut Hero, effect: Effect, duration: u32, chance: f32, rng: &mut RngStream) {
  match src {
    EffectSource::SelfTargeted => target.apply_effect(effect, duration),
    EffectSource::Enemy(enemy) => {
      let eff = enemy.stats.eff;
      let resist = target.stats.effres;
      let chance = f32::min(0.85, chance * f32::max(0.0, eff - resist));
      rng.with_chance(chance, |rng| {
        let snapshot = &rng.snapshot;

      });
    }
  }
}

fn battle_tick<'a>(old: BattleSnapshot) -> BattleSnapshot {
  let mut battle = old.clone();

  let hero_id = battle.get_turn_hero_id();
  let hero = &mut battle.heroes[hero_id];

  hero_on_turn_start(hero);

  hero_on_turn_end(hero);

  battle
}

struct RngStream<'a> {
  branches: Vec<(f32, Box<dyn FnOnce(&mut RngStream) + 'a>)>,
  finally: Option<Box<dyn FnOnce(&mut RngStream) + 'a>>,
  snapshot: BattleSnapshot,
}

impl<'a> RngStream<'a> {

  fn with_chance(&mut self, chance: f32, f: impl FnOnce(&mut RngStream) + 'a) {
    let boxed = Box::new(f);
    self.branches.push((chance, boxed));
  }

  fn then(&mut self, f: impl FnOnce(&mut RngStream) + 'a) {
    self.finally = Some(Box::new(f));
  }

  fn run(&mut self, snapshot: &mut BattleSnapshot, index: usize) -> RngStream {
    let (chance, f) = self.branches.remove(index);
    let mut res = RngStream {
      branches: vec![],
      finally: None,
      snapshot: self.snapshot.clone(),
    };
    f(&mut res);
    if let Some(f) = self.finally.take() {
      f(&mut res);
    }
    res
  }
}

struct Possibility<T> {
  label: &'static str,
  chance: f32,
  event: Box<dyn Fn(&mut T)>,
}