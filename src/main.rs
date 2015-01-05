#![feature(phase,globs)]

extern crate regex;
#[phase(plugin)]
extern crate regex_macros;
extern crate tox;

use tox::core::*;

use std::rand;
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
    let groupbot_id = tox.add_friend(box groupchat_addr, "Hello".to_string()).ok().unwrap();

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
                  for reply in tox::util::split_message(get_response(msg, user_name).as_slice()).iter() {
                    tox.group_message_send(group, reply.to_string());
                    std::io::timer::sleep(std::time::duration::Duration::milliseconds(500));
                  }
                } else if msg.starts_with("^flip") {
                  let user_name = tox.group_peername(group, peer).unwrap();
                  let reply = user_name + "'s coin landed on " +
                              if rand::random::<bool>() == true { "heads." } else { "tails." };

                  tox.group_message_send(group, reply.to_string());
                }
              },

                _ => { }
            }
        }
    }
}

fn get_response(message: String, user_name: String) -> String {
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
