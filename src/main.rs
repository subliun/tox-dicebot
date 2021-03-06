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

#![feature(box_syntax)]

extern crate "rstox" as tox;
extern crate markov;
extern crate time;

use tox::core::*;
use tox::av::*;
use markov::Chain;
use std::rand;

use std::slice::SliceExt;
use std::sync::mpsc::{Select};
use std::old_io::timer::{self, Timer};
use std::time::Duration;

mod battle;
mod dice;
mod zalgo;

static BOOTSTRAP_IP: &'static str = "192.254.75.102";
static BOOTSTRAP_PORT: u16 = 33445;
static BOOTSTRAP_KEY: &'static str =
"951C88B7E75C867418ACDB5D273821372BB5BD652740BCDF623A4FA293E75D2F";
static GROUPCHAT_ADDR: &'static str =
"56A1ADE4B65B86BCD51CC73E2CD4E542179F47959FE3E0E21B4B0ACDADE51855D34D34D37CB5";
static BOT_NAME: &'static str = "DiceBot";
static MARKOV_NAME: &'static str = "iranjontu";
static MARKOV_RANDOM_CHAT_TIME: f64 = 1500f64;

// TODO: ad-hoc. Remove when rstox implements message splitting.
pub fn split_message(mut m: &str) -> Vec<&str> {
    let mut ret = vec!();
    let mut last_whitespace = false;
    while m.len() > MAX_MESSAGE_LENGTH {
        let mut end = 0;
        for (i, c) in m.char_indices() {
            if c.is_whitespace() {
                if !last_whitespace {
                    last_whitespace = true;
                    end = i;
                }
            } else {
                last_whitespace = false;
            }
            if i + c.len_utf8() > MAX_MESSAGE_LENGTH {
                if end > 0 {
                    ret.push(&m[..end]);
                    m = &m[(end+m.char_at(end).len_utf8())..];
                } else {
                    ret.push(&m[..i]);
                    m = &m[i..];
                }
                break;
            }
        }
    }
    if m.len() > 0 {
        ret.push(m);
    }
    ret
}


// consider incapsulating this into a separate entity
fn do_msg(tox: &mut Tox, battle: &mut battle::Battle, chain: &mut Chain<String>, group: i32, peer: i32, msg: String, last_message: String) {
  let mut mit = msg.splitn(1, ' ');
  match mit.next().unwrap() {
    "^diceid" => {
      //tox.group_message_send(group, "My Tox ID is: " + tox.get_address().as_slice());
    },
    "^dice" | "^roll" => {
      let user_name = tox.group_peername(group, peer).unwrap();
      let roll = dice::get_response_dice_roll(mit.next().unwrap_or(""), user_name);
      // TODO: add a `split_send` function
      for reply in split_message(roll.as_slice()).iter() {
        tox.group_message_send(group, reply);
        timer::sleep(Duration::milliseconds(500));
      }
    },
    "^flip" => {
      let user_name = tox.group_peername(group, peer).unwrap();
      tox.group_message_send(group, &dice::get_response_flip(user_name));
    },
    "^chance" => {
      tox.group_message_send(group, ("There is a ".to_string() + dice::chance().as_slice() + " chance.".as_slice()).as_slice());
    },
    "^zalgo" => {
      let zalgo = zalgo::make_zalgo(mit.next().unwrap_or("").trim().to_string());
      for reply in split_message(zalgo.as_slice()).iter() {
        tox.group_message_send(group, reply);
        timer::sleep(Duration::milliseconds(200));
      }
    },
    "^question" => {
      tox.group_message_send(group, &question::retrieve_answer(mit.next().unwrap_or("").trim().to_string()));
    },
    "^fight" => {
      tox.group_message_send(group, &fight::get_response_fight(mit.next().unwrap_or("").trim().to_string()));
    },
    "^endchat" => {
      tox.set_name("DiceBot").unwrap();
    },
    "^chat" => {
      tox.set_name(MARKOV_NAME).unwrap();
      tox.group_message_send(group, &chain.generate_str());
    },
    "^translate" | "^tr" => {
      let mut new_iter = mit.next().unwrap().splitn(1, ' ');
      let lang = new_iter.next().unwrap_or("");
      let mut to_translate = new_iter.next().unwrap_or("");
      if to_translate == "^" {
        to_translate = last_message.as_slice();
      }
      let result = &translate::get_response_translate(to_translate.to_string(), lang.to_string());
      tox.group_message_send(group, result.trim());
    },
    "^remember" => {
      let result = remember::remember_assoc(mit.next().unwrap_or("").to_string());
      if result != "" {
        tox.group_message_send(group, &result);
      }
    },
    _ if msg.starts_with("^") => {
      let result = remember::retrieve_assoc(msg.replace("^", ""));
      if result != None {
        tox.group_message_send(group, &result.unwrap());
      }
    },
    _ => {},
  }
}

fn main() {
  let (tox_cell, mut av) = ToxAv::new(Tox::new(ToxOptions::new()), 1);
  let gr_audio = av.group_audio(box |_, _| {});

  let mut tox = tox_cell.borrow_mut();
  tox.set_name(BOT_NAME).unwrap();

  let bootstrap_key = BOOTSTRAP_KEY.parse().unwrap();
  tox.bootstrap_from_address(BOOTSTRAP_IP.to_string(), BOOTSTRAP_PORT,
      bootstrap_key).unwrap();

  let groupchat_addr = GROUPCHAT_ADDR.parse().unwrap();
  let groupbot_id = tox.add_friend(groupchat_addr, "Down with groupbot! Glory to Ukraine!").ok().unwrap();
  let mut group_num = 0;
  let mut last_message: String = String::new();
  let mut time_since_last_markov_message = time::precise_time_s();


  println!("My address is: {}", tox.get_address());

  let mut battle = battle::Battle::new();
  //let mut battle_timer = None;

  let mut chain = Chain::for_strings();
  chain.feed_file(&Path::new("markov.txt"));

  loop {
    timer::sleep(std::time::duration::Duration::milliseconds(50));

    if time::precise_time_s() - time_since_last_markov_message > MARKOV_RANDOM_CHAT_TIME {
      if rand::random::<u32>() % 2000 == 1 {
        tox.set_name(MARKOV_NAME).unwrap();
        tox.group_message_send(group_num, &chain.generate_str());
        time_since_last_markov_message = time::precise_time_s();
      }
    }

    for ev in tox.iter() {
      match ev {
        StatusMessage(id, _) if id == groupbot_id => {
          if tox.count_chatlist() < 1 {
            tox.send_message(groupbot_id, "invite").unwrap();
            println!("connected to groupbot");
          }
        },

        FriendRequest(friend_id, msg) => {
          tox.add_friend_norequest(*friend_id);
        },

        GroupInvite(id, kind, data) => {
          println!("GroupInvite(_, {:?}, _) ", kind);
          match kind {
            GroupchatType::Text => tox.join_groupchat(id, &data).unwrap(),
              GroupchatType::Av => gr_audio.join_groupchat(&mut tox, id, &data).unwrap(),
          };
        },

        GroupMessage(group, peer, msg) => {
          if tox.group_peername(group, peer).unwrap() == tox.get_self_name().unwrap() {
            last_message = msg.clone();
            return
          }
          
          println!("{}: {}", tox.group_peername(group, peer).unwrap(), msg);
          group_num = group;

          if msg.starts_with("^") && !msg.starts_with("^chat") {
              tox.set_name(BOT_NAME).unwrap();
          }

          if !msg.starts_with("^") && msg.len() < 600 && !msg.trim().is_empty() {
            let mut clean_message = msg.clone();
            for name in tox.group_get_names(group).unwrap().into_iter() {
              clean_message = clean_message.replace((name.unwrap().trim().to_string() + ":").as_slice(), "");
            }
            chain.feed_str(clean_message.trim().as_slice());
          }

          if msg.contains(MARKOV_NAME) {
            tox.set_name(MARKOV_NAME).unwrap();
            tox.group_message_send(group, &chain.generate_str());
          } else {
            do_msg(&mut tox, &mut battle, &mut chain, group, peer, msg.clone(), last_message);
          }

          last_message = msg.clone();
        },

        _ => { }
      }
    }
    tox.wait();
  }
}

mod fight {
  use std::rand;
  use std::rand::{thread_rng, Rng};
  use std::ascii::AsciiExt;

  pub fn get_response_fight(msg: String) -> String {
    let message = msg.to_ascii_lowercase().replace(".", "").to_string();
    if message.contains(" me") { return "m8".to_string() }
    if !message.contains(" vs ") { return "That's not a fight! This is a fight: ^fight person1 vs person2".to_string() }

    let winner: &str;
    let mut extra_message = "";
    if message.contains("qtox") {
      winner = "qtox";
      extra_message = "qTox is better.";
    } else if message.contains("subliun") {
      winner = "subliun";
      extra_message = "(subliun always wins)";
    } else {
      let mut fighters: Vec<&str> = vec!();
      for fighter in message.split_str(" vs ") {
        fighters.push(fighter);
      }
      winner = *thread_rng().choose(fighters.as_slice()).unwrap_or(&"A failure (that's you)");
    }

    winner.to_string() + " won the fight! " + extra_message
  }
}

mod question {
  use std::rand;
  use std::ascii::AsciiExt;

  pub fn retrieve_answer(question: String) -> String {
    let question_words = ["do", "did", "does", "am", "is", "are", "has",
                          "have", "was", "were", "will", "can",
                          "could", "shall", "should"];
    let mut good_question = false;
    for word in question_words.iter() {
        if question.as_slice().to_ascii_lowercase().to_string().starts_with(*word) {
          good_question = true;
          break;
        }
    }

    if !good_question { return "That's not a good question.".to_string() }

    match rand::random::<u32>() % 4 {
      0 => "Yes.",
      1 => "No.",
      2 => "Maybe.",
      _ => "I cannot say."
    }.to_string()
  }
}

mod remember {
  use std::old_io::*;
  use std::old_io::fs::PathExtensions;

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
    } else {
      return None
    }

    if file.is_err() { return None }

    let mut result = None;
    for m_line in BufferedReader::new(file.unwrap()).lines() {
      if m_line.is_err() { break; }
      let line = m_line.unwrap();
      if line.splitn(1, ':').nth(0).unwrap() == message {
        result = Some(line.splitn(1, ':').nth(1).unwrap().replace("\n", "").to_string());
      }
    }

    return result
  }
}

mod translate {
  use std::old_io::Command;

  pub fn get_response_translate(phrase: String, to_language: String) -> String {
    let mut result: String = String::new();
    for sentence in phrase.split('.') {
      let mut process = match Command::new("python").arg("translate.py").arg("-t").arg(to_language.clone()).arg(sentence).spawn() {
        Ok(p) => p,
        Err(e) => return "Error: translation failed. Did you input the right args?".to_string(),
      };
      result = result + String::from_utf8(process.stdout.as_mut().unwrap().read_to_end().unwrap()).unwrap().as_slice();
    }

    return result;
  }
}
/* vim: set ts=2 sw=2 expandtab ai: */
