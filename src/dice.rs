use std::rand;

pub fn get_response_flip(user_name: String) -> String {
  user_name + "'s coin landed on " + if rand::random::<bool>() == true { "heads." } else { "tails." }
}

pub fn get_response_dice_roll(message: &str, user_name: String) -> String {
  let param = if message == "" { "6" } else { message };

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
    times = param.slice_to(d_location).replace("d", "").parse::<u32>().unwrap_or(1);
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
  if roll_range.clone().is_ok() && roll_range.clone().unwrap() > 0 {
    for i in range(0, times) {
      if i != 0 { result_builder.push_str(", ") };
      if i == times - 1 && times > 1 { result_builder.push_str("and "); }
      let roll = roll_dice(roll_range.clone().unwrap());
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
  match &*user_name {
    "{☯}S☠ǚll{☣}" => "Invalid request MORON. Please use your GAY head to type non-fucking-negative numbers.",
    "Candy Gumdrop" => "Invalid request GORGEOUS. Please use your BEAUTIFUL head to type non-fucking-negative numbers.",
    _ => "Invalid request. Please use a non-negative number between 2 and 2^64.",
  }.to_string()
}

fn roll_dice(roll_range: u64) -> u64 {
  ((rand::random::<u64>() % roll_range) + 1)
}

fn add_formatting(roll: u64) -> String {
  let die_face: String = match get_die_face(roll) {
    Some(face) => " ".to_string() + face.to_string().as_slice(),
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


pub fn chance() -> String {
  (rand::random::<u64>() % (100 + 1)).to_string() + "%"
}
