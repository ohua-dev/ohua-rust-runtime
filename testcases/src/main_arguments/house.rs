#![allow(dead_code)]

#[derive(Clone)]
pub struct Human {
    pub name: String,
    pub emotional_state: EmotionalState,
    pub age: u8,
}

#[derive(Clone)]
pub struct House {
    pub rooms: u8,
    pub inhabitants: Vec<Human>,
    pub address: String,
}

#[derive(Clone)]
pub enum EmotionalState {
    Happy,
    Content,
    Sad,
}

impl House {
    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn change_address(&mut self, new: String) {
        self.address = new;
    }
}

pub fn move_house(mut house: House, target_address: String) -> House {
    print!("Moving the House from {}. ", house.get_address());
    house.change_address(target_address);
    println!("Moved to {}", house.get_address());
    house
}

pub fn move_in_one(mut house: House, mut humans: Vec<Human>) -> (House, Vec<Human>) {
    if let Some(human) = humans.pop() {
        house.inhabitants.push(human);
    }
    (house, humans)
}

pub fn move_in_one_more(bundle_input: (House, Vec<Human>)) -> (House, Vec<Human>) {
    let (mut house, mut humans) = bundle_input;
    if let Some(human) = humans.pop() {
        house.inhabitants.push(human);
    }
    (house, humans)
}

pub fn house_information(bundle_input: (House, Vec<Human>)) {
    let house = bundle_input.0;
    println!("Address: {}, Rooms: {}", house.get_address(), house.rooms);
    let mut inhabitants = String::new();
    for inh in house.inhabitants {
        inhabitants += (inh.name + ", ").as_str();
    }
    println!("Inhabited by: {}", inhabitants)
}

pub fn evict_one(bundle_input: (House, Vec<Human>)) -> House {
    let mut house = bundle_input.0;
    let _ = house.inhabitants.pop();
    house
}
