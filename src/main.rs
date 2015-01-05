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

#![feature(phase,globs)]

extern crate regex;
#[phase(plugin)]
extern crate regex_macros;
extern crate tox;

use tox::core::*;

use std::slice::SliceExt;
use std::sync::mpsc::{Select};

static BOOTSTRAP_IP: &'static str = "192.254.75.102";
static BOOTSTRAP_PORT: u16 = 33445;
static BOOTSTRAP_KEY: &'static str =
"951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F";
static GROUPCHAT_ADDR: &'static str =
"56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5";
static BOT_NAME: &'static str = "DiceBot";

fn main() {
  let tox = Tox::new(ToxOptions::new()).unwrap();
  let av = tox.av(2).unwrap();

  tox.set_name(BOT_NAME.to_string()).unwrap();

  let bootstrap_key = BOOTSTRAP_KEY.parse().unwrap();
  tox.bootstrap_from_address(BOOTSTRAP_IP.to_string(), BOOTSTRAP_PORT,
      box bootstrap_key).unwrap();

  let groupchat_addr = GROUPCHAT_ADDR.parse().unwrap();
  let groupbot_id = tox.add_friend(box groupchat_addr, "Down with groupbot! Glory to Ukraine!".to_string()).ok().unwrap();

  let sel = Select::new();
  let mut tox_rx = sel.handle(tox.events());
  let mut av_rx = sel.handle(av.events());
  unsafe {
    tox_rx.add();
    av_rx.add();
  }

  loop {
    sel.wait();
    while let Ok(ev) = tox.events().try_recv() {
      match ev {
        StatusMessage(id, _) if id == groupbot_id => {
          if tox.count_chatlist() < 1 {
            tox.send_message(groupbot_id, "invite".to_string()).unwrap();
            println!("connected to groupbot");
          }
        },

          GroupInvite(id, ty, data) => {
            println!("GroupInvite(_, {}, _) ", ty);
            match ty {
              GroupchatType::Text => tox.join_groupchat(id, data).unwrap(),
                GroupchatType::Av => av.join_av_groupchat(id, data).unwrap(),
            };
          },

          GroupMessage(group, peer, msg) => {
            println!("{}: {}", tox.group_peername(group, peer).unwrap(), msg);
            if msg.starts_with("^dice") || msg.starts_with("^roll") {
              let user_name = tox.group_peername(group, peer).unwrap();
              for reply in tox::util::split_message(dice::get_response_dice_roll(msg, user_name).as_slice()).iter() {
                tox.group_message_send(group, reply.to_string());
                std::io::timer::sleep(std::time::duration::Duration::milliseconds(500));
              }
            } else if msg.starts_with("^flip") {
              let user_name = tox.group_peername(group, peer).unwrap();

              tox.group_message_send(group, dice::get_response_flip(user_name));
            } else if msg.starts_with("^zalgo") {
              for reply in tox::util::split_message(zalgo::make_zalgo(msg.replace("^zalgo", "")).as_slice()).iter() {
                tox.group_message_send(group, reply.to_string());
                std::io::timer::sleep(std::time::duration::Duration::milliseconds(200));
              }
            } else if msg.starts_with("^remember") {
              let result = remember::remember_assoc(msg.replace("^remember", ""));
              if result != "" {
                tox.group_message_send(group, result);
              }
            } else if msg.starts_with("^") {
              let result = remember::retrieve_assoc(msg.replace("^", ""));
              if result != None {
                tox.group_message_send(group, result.unwrap());
              }
            }
          },

          _ => { }
      }
    }
  }
}

mod remember {
  use std::io::*;
  use std::io::fs::PathExtensions;

  static filename: &'static str = "table.txt";

  pub fn remember_assoc(message: String) -> String {
    let processed_message = message.replace("\n", "").replace("^", "").trim().to_string() + "\n";
    let path = Path::new(filename);

    let mut file;
    if path.exists() {
      file = File::open_mode(&path, Append, Write)
    } else {
      file = File::open_mode(&path, Truncate, Write)
    }

    if !processed_message.contains(":") {
      return "Error. Could not find : in remember command.".to_string()
    }

    file.write(processed_message.into_bytes().as_slice());
    return String::new()
  }

  pub fn retrieve_assoc(message: String) -> Option<String> {
    let file;
    let path = Path::new(filename);

    if path.exists() {
      file = File::open(&path);
    }else {
      return None
    }

    if file.is_err() { return None }

    for m_line in BufferedReader::new(file.unwrap()).lines() {
      if m_line.is_err() { println!("RROOJS"); break; }
      let line = m_line.unwrap();
      println!("{}", line);
      if line.splitn(1, ':').nth(0).unwrap() == message {
        return Some(line.splitn(1, ':').nth(1).unwrap().replace("\n", "").to_string());
      }
    }

    return None
  }
}

mod dice {
  use std::rand;

  pub fn get_response_flip(user_name: String) -> String {
    user_name + "'s coin landed on " + if rand::random::<bool>() == true { "heads." } else { "tails." }
  }

  pub fn get_response_dice_roll(message: String, user_name: String) -> String {
    let m_param = message.split(' ').nth(1);

    let param = m_param.unwrap_or("6");

    match param {
      "joint" => return "smoke weed everyday".to_string(),
        "rick" => return "never gonna give you up".to_string(),
        _ => { },
    }

    if param.contains(" 0") || param.contains("d0") {
      return "Error Dividing By Cucumber. Reinstall Universe And Try Again.".to_string();
    }

    let mut roll_result = None;
    let times;
    let roll_range;
    if param.contains("d") {
      let d_location = param.find('d').unwrap();
      times = param.slice_to(d_location).replace("d", "").parse::<uint>().unwrap_or(1);
      if times > 500 {
        return "Invalid request. You tried to roll too many times. My robot arms can only take so much. ;_;'".to_string()
      }
      roll_range = param.slice_from(d_location).replace("d", "").parse::<u64>();
    } else {
      times = 1;
      roll_range = param.parse::<u64>();
    }

    let mut roll_sum = 0;
    let mut result_builder = String::new();
    if roll_range != None && roll_range.unwrap() > 0 {
      for i in range(0, times) {
        if i != 0 { result_builder.push_str(", ") };
        if i == times - 1 && times > 1 { result_builder.push_str("and "); }
        let roll = roll_dice(roll_range.unwrap());
        roll_sum += roll;
        result_builder.push_str(add_formatting(roll).as_slice());
      }
      roll_result = Some(result_builder);
    }

    if roll_result == None {
      on_invalid_input(user_name)
    } else {
      if times == 1 {
        user_name + " rolled " + roll_result.unwrap().as_slice()
      } else {
        user_name + " rolled a total of " + roll_sum.to_string().as_slice() + " with rolls of " + roll_result.unwrap().as_slice()
      }
    }
  }

  fn on_invalid_input(user_name: String) -> String {
    if user_name == "{☯}S☠ǚll{☣}" {
      "Invalid request MORON. Please use your GAY head to type non-fucking-negative numbers.".to_string()
    } else if user_name == "Candy Gumdrop" {
      "Invalid request GORGEOUS. Please use your BEAUTIFUL head to type non-fucking-negative numbers.".to_string()
    } else {
      "Invalid request. Please use a non-negative number between 2 and 2^64.".to_string()
    }
  }

  fn roll_dice(roll_range: u64) -> u64 {
    ((rand::random::<u64>() % roll_range) + 1)
  }

  fn add_formatting(roll: u64) -> String {
    let die_face: String = match get_die_face(roll) {
      Some(face) => " ".to_string() + String::from_char(1, face).as_slice(),
        None => "".to_string(),
    };

    roll.to_string() + die_face.as_slice()
  }

  fn get_die_face(number: u64) -> Option<char> {
    match number {
      1 => Some('⚀'),
        2 => Some('⚁'),
        3 => Some('⚂'),
        4 => Some('⚃'),
        5 => Some('⚄'),
        6 => Some('⚅'),
        _ => None
    }
  }

}

mod zalgo {
  use std::rand;
  use std::rand::{thread_rng, Rng};

  static ZALGO_CHARS: [char; 113]  = [
    '\u030d', /*     Ì     */		'\u030e', /*     ÌŽ     */		'\u0304', /*     Ì„     */		'\u0305', /*     Ì…     */
    '\u033f', /*     Ì¿     */		'\u0311', /*     Ì‘     */		'\u0306', /*     Ì†     */		'\u0310', /*     Ì     */
    '\u0352', /*     Í’     */		'\u0357', /*     Í—     */		'\u0351', /*     Í‘     */		'\u0307', /*     Ì‡     */
    '\u0308', /*     Ìˆ     */		'\u030a', /*     ÌŠ     */		'\u0342', /*     Í‚     */		'\u0343', /*     Ì“     */
    '\u0344', /*     ÌˆÌ     */		'\u034a', /*     ÍŠ     */		'\u034b', /*     Í‹     */		'\u034c', /*     ÍŒ     */
    '\u0303', /*     Ìƒ     */		'\u0302', /*     Ì‚     */		'\u030c', /*     ÌŒ     */		'\u0350', /*     Í     */
    '\u0300', /*     Ì€     */		'\u0301', /*     Ì     */		'\u030b', /*     Ì‹     */		'\u030f', /*     Ì     */
    '\u0312', /*     Ì’     */		'\u0313', /*     Ì“     */		'\u0314', /*     Ì”     */		'\u033d', /*     Ì½     */
    '\u0309', /*     Ì‰     */		'\u0363', /*     Í£     */		'\u0364', /*     Í¤     */		'\u0365', /*     Í¥     */
    '\u0366', /*     Í¦     */		'\u0367', /*     Í§     */		'\u0368', /*     Í¨     */		'\u0369', /*     Í©     */
    '\u036a', /*     Íª     */		'\u036b', /*     Í«     */		'\u036c', /*     Í¬     */		'\u036d', /*     Í­     */
    '\u036e', /*     Í®     */		'\u036f', /*     Í¯     */		'\u033e', /*     Ì¾     */		'\u035b', /*     Í›     */
    '\u0346', /*     Í†     */		'\u031a', /*     Ìš     */
    '\u0316', /*     Ì–     */		'\u0317', /*     Ì—     */		'\u0318', /*     Ì˜     */		'\u0319', /*     Ì™     */
    '\u031c', /*     Ìœ     */		'\u031d', /*     Ì     */		 '\u031e', /*     Ìž     */		 '\u031f', /*     ÌŸ     */
    '\u0320', /*     Ì      */		'\u0324', /*     Ì¤     */		'\u0325', /*     Ì¥     */		'\u0326', /*     Ì¦     */
    '\u0329', /*     Ì©     */		'\u032a', /*     Ìª     */		'\u032b', /*     Ì«     */		'\u032c', /*     Ì¬     */
    '\u032d', /*     Ì­     */		'\u032e', /*     Ì®     */		'\u032f', /*     Ì¯     */		 '\u0330', /*     Ì°     */
    '\u0331', /*     Ì±     */		'\u0332', /*     Ì²     */		'\u0333', /*     Ì³     */		'\u0339', /*     Ì¹     */
    '\u033a', /*     Ìº     */		'\u033b', /*     Ì»     */		'\u033c', /*     Ì¼     */		'\u0345', /*     Í…     */
    '\u0347', /*     Í‡     */		'\u0348', /*     Íˆ     */		'\u0349', /*     Í‰     */		'\u034d', /*     Í     */
    '\u034e', /*     ÍŽ     */		'\u0353', /*     Í“     */		'\u0354', /*     Í”     */		'\u0355', /*     Í•     */
    '\u0356', /*     Í–     */		'\u0359', /*     Í™     */		'\u035a', /*     Íš     */		'\u0323', /*     Ì£     */
    '\u0315', /*     Ì•     */		'\u031b', /*     Ì›     */		'\u0340', /*     Ì€     */		'\u0341', /*     Ì     */
    '\u0358', /*     Í˜     */		'\u0321', /*     Ì¡     */		'\u0322', /*     Ì¢     */		'\u0327', /*     Ì§     */
    '\u0328', /*     Ì¨     */		'\u0334', /*     Ì´     */		'\u0335', /*     Ìµ     */		'\u0336', /*     Ì¶     */
    '\u034f', /*     Í     */		'\u035c', /*     Íœ     */		'\u035d', /*     Í     */		  '\u035e', /*     Íž     */
    '\u035f', /*     ÍŸ     */		'\u0360', /*     Í      */		'\u0362', /*     Í¢     */		'\u0338', /*     Ì¸     */
    '\u0337', /*     Ì·     */		'\u0361', /*     Í¡     */		'\u0489' /*     Ò‰_     */
      ];

  pub fn make_zalgo(input: String) -> String {
    let mut result: String = String::new();
    for character in input.chars() {
      result.push_str(String::from_char(1, character).as_slice());

      if character == ' ' {
        continue;
      }

      for _ in range(0, 5 + (rand::random::<uint>() % 10)) {
        result.push_str(String::from_char(1, *thread_rng().choose(&ZALGO_CHARS).unwrap()).as_slice());
      }
    }

    result
  }
}

/* vim: set ts=2 sw=2 expandtab ai: */
