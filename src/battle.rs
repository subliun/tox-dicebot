/*
Copyright (C) 2015 subliun <subliunisdev@gmail.com>
All Rights Reserved.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use tox::core::*;
use std::rand;

pub struct Battle {
  active: bool,
  people: Vec<Person>,
  pub duration: u32,
}

impl Battle {
  pub fn new() -> Battle {
    let mut people: Vec<Person> = vec!();
    return Battle { active: false, people: people, duration: 60 }
  }

  pub fn start_battle(&mut self, tox: &Tox, group_id: i32, people_names: Vec<String>) {
    for name in people_names.iter() {
      self.people.push(Person::new(name.clone(), 20));
    }
    tox.group_message_send(group_id, "A battle has begun! It will go for ".to_string() + self.duration.to_string().as_slice() + " seconds. Fight!");
  }

  pub fn get_person_by_name(&mut self, name: String) -> Option<&mut Person> {
    for person in self.people.iter_mut() {
      if person.name == name {
        return Some(person)
      }
    }

    return None
  }

  pub fn get_attack_by_name(name: String) -> Option<Attack> {
    for attack in Battle::get_attacks().into_iter() {
      if attack.name == name {
        return Some(attack)
      }
    }

    return None
  }

  fn get_attacks() -> Vec<Attack> {
    vec![
    Attack { name: "punch".to_string(), damage_low: 2, damage_high: 5, cooldown: 5 }
    ]
  }

  pub fn end_battle(&mut self) {
    self.active = false;
    self.people.clear();
  }
}

pub struct Person {
  name: String,
  health: i32,
  max_health: i32,
  curr_cooldown_time: i32,
}

impl Person {
  fn new(name: String, max_health: i32) -> Person {
    Person { name: name, health: max_health, max_health: max_health, curr_cooldown_time: 5 }
  }

  fn damage(&mut self, amount: i32) {
    self.health -= amount;
    if self.health < 0 {
      self.health = 0;
    }
  }

  fn heal(&mut self, amount: i32) {
    self.health += amount;
    if self.health > self.max_health {
      self.health = self.max_health;
    }
  }
}

struct Attack {
  name: String,
  damage_low: i32,
  damage_high: i32,
  cooldown: i32,
}

fn random_range(low: u32, high: u32) -> u32 {
  (rand::random::<u32>() % (high - low)) + low
}
