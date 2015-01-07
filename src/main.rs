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
            } else if msg.starts_with("^question"){
              tox.group_message_send(group, question::retrieve_answer(msg.replace("^question", "").trim().to_string()));
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

mod question {
  use std::hash;
  use std::ascii::AsciiExt;

  pub fn retrieve_answer(question: String) -> String {
    let question_words = ["do", "did", "does", "am", "is", "are", "has", "have", "was", "were", "will", "can", "could"];
    let mut good_question = false;
    for word in question_words.iter() {
        if question.as_slice().to_ascii_lowercase().to_string().starts_with(*word) {
          good_question = true;
          break;
        }
    }

    if !good_question { return "That's not a good question.".to_string() }

    let hash_result = hash::hash(question.as_slice()) % 4;
    println!("{}", hash_result);
    match hash_result {
      0 => "Yes.",
      1 => "No.",
      2 => "Maybe.",
      _ => "I cannot say."
    }.to_string()
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
      if m_line.is_err() { break; }
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
    '\u{30d}', /*     Ì     */		'\u{30e}', /*     ÌŽ     */		'\u{304}', /*     Ì„     */		'\u{305}', /*     Ì…     */
    '\u{33f}', /*     Ì¿     */		'\u{311}', /*     Ì‘     */		'\u{306}', /*     Ì†     */		'\u{310}', /*     Ì     */
    '\u{352}', /*     Í’     */		'\u{357}', /*     Í—     */		'\u{351}', /*     Í‘     */		'\u{307}', /*     Ì‡     */
    '\u{308}', /*     Ìˆ     */		'\u{30a}', /*     ÌŠ     */		'\u{342}', /*     Í‚     */		'\u{343}', /*     Ì“     */
    '\u{344}', /*     ÌˆÌ     */		'\u{34a}', /*     ÍŠ     */		'\u{34b}', /*     Í‹     */		'\u{34c}', /*     ÍŒ     */
    '\u{303}', /*     Ìƒ     */		'\u{302}', /*     Ì‚     */		'\u{30c}', /*     ÌŒ     */		'\u{350}', /*     Í     */
    '\u{300}', /*     Ì€     */		'\u{301}', /*     Ì     */		'\u{30b}', /*     Ì‹     */		'\u{30f}', /*     Ì     */
    '\u{312}', /*     Ì’     */		'\u{313}', /*     Ì“     */		'\u{314}', /*     Ì”     */		'\u{33d}', /*     Ì½     */
    '\u{309}', /*     Ì‰     */		'\u{363}', /*     Í£     */		'\u{364}', /*     Í¤     */		'\u{365}', /*     Í¥     */
    '\u{366}', /*     Í¦     */		'\u{367}', /*     Í§     */		'\u{368}', /*     Í¨     */		'\u{369}', /*     Í©     */
    '\u{36a}', /*     Íª     */		'\u{36b}', /*     Í«     */		'\u{36c}', /*     Í¬     */		'\u{36d}', /*     Í­     */
    '\u{36e}', /*     Í®     */		'\u{36f}', /*     Í¯     */		'\u{33e}', /*     Ì¾     */		'\u{35b}', /*     Í›     */
    '\u{346}', /*     Í†     */		'\u{31a}', /*     Ìš     */
    '\u{316}', /*     Ì–     */		'\u{317}', /*     Ì—     */		'\u{318}', /*     Ì˜     */		'\u{319}', /*     Ì™     */
    '\u{31c}', /*     Ìœ     */		'\u{31d}', /*     Ì     */		 '\u{31e}', /*     Ìž     */		 '\u{31f}', /*     ÌŸ     */
    '\u{320}', /*     Ì      */		'\u{324}', /*     Ì¤     */		'\u{325}', /*     Ì¥     */		'\u{326}', /*     Ì¦     */
    '\u{329}', /*     Ì©     */		'\u{32a}', /*     Ìª     */		'\u{32b}', /*     Ì«     */		'\u{32c}', /*     Ì¬     */
    '\u{32d}', /*     Ì­     */		'\u{32e}', /*     Ì®     */		'\u{32f}', /*     Ì¯     */		 '\u{330}', /*     Ì°     */
    '\u{331}', /*     Ì±     */		'\u{332}', /*     Ì²     */		'\u{333}', /*     Ì³     */		'\u{339}', /*     Ì¹     */
    '\u{33a}', /*     Ìº     */		'\u{33b}', /*     Ì»     */		'\u{33c}', /*     Ì¼     */		'\u{345}', /*     Í…     */
    '\u{347}', /*     Í‡     */		'\u{348}', /*     Íˆ     */		'\u{349}', /*     Í‰     */		'\u{34d}', /*     Í     */
    '\u{34e}', /*     ÍŽ     */		'\u{353}', /*     Í“     */		'\u{354}', /*     Í”     */		'\u{355}', /*     Í•     */
    '\u{356}', /*     Í–     */		'\u{359}', /*     Í™     */		'\u{35a}', /*     Íš     */		'\u{323}', /*     Ì£     */
    '\u{315}', /*     Ì•     */		'\u{31b}', /*     Ì›     */		'\u{340}', /*     Ì€     */		'\u{341}', /*     Ì     */
    '\u{358}', /*     Í˜     */		'\u{321}', /*     Ì¡     */		'\u{322}', /*     Ì¢     */		'\u{327}', /*     Ì§     */
    '\u{328}', /*     Ì¨     */		'\u{334}', /*     Ì´     */		'\u{335}', /*     Ìµ     */		'\u{336}', /*     Ì¶     */
    '\u{34f}', /*     Í     */		'\u{35c}', /*     Íœ     */		'\u{35d}', /*     Í     */		  '\u{35e}', /*     Íž     */
    '\u{35f}', /*     ÍŸ     */		'\u{360}', /*     Í      */		'\u{362}', /*     Í¢     */		'\u{338}', /*     Ì¸     */
    '\u{337}', /*     Ì·     */		'\u{361}', /*     Í¡     */		'\u{489}' /*     Ò‰_     */
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
