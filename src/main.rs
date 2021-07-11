mod battle;
mod scheduler;
mod rng;

use battle::*;
use std::sync::{ Mutex, Arc };

fn main() {
	let hero1 = Hero {
		stats: HeroStats {
      max_hp: 10000.0,
      hp: 10000.0,
      atk: 3000.0,
      spd: 200.0,
      def: 1000.0,
      cc: 100.0,
      cdmg: 150.0,
      eff: 0.0,
      effres: 0.0,
      element: Element::Dark,
      hit_chance: 100.0,
      crit_resist: 0.0,
    },
    alive: true,
    readiness: 0,
    effects: Effects::new(),
    team: 1
	};
  let hero2 = Hero {
		stats: HeroStats {
      max_hp: 10000.0,
      hp: 10000.0,
      atk: 3000.0,
      spd: 200.0,
      def: 1000.0,
      cc: 100.0,
      cdmg: 150.0,
      eff: 0.0,
      effres: 0.0,
      element: Element::Dark,
      hit_chance: 100.0,
      crit_resist: 0.0,
    },
    alive: true,
    readiness: 0,
    effects: Effects::new(),
    team: 2
	};
	let ss = BattleSnapshot {
    base_stats: vec![hero1.clone(), hero2.clone()],
    heroes: vec![hero1, hero2]
	};

  let observer = scheduler::DataCollector::new();

  // let battle = battle::turn_start(&ss);
  // scheduler::rng_node_run(battle, ss, Arc::new(Mutex::new(observer)), 0);
}

struct PrintObserver {

}

// impl RngObserver for PrintObserver {
//   fn choose_branch(&self, branches: &Vec<Possibility>) -> Vec<usize> {
//     (0..branches.len()).collect()
//   }

//   fn snapshot_did_compute(&self, ss: &BattleSnapshot, p: &Possibility) {
//     println!("{}", p.label);
//   }
// }