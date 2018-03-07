pub struct Human {
    pub name: String,
    pub emotional_state: EmotionalState,
    pub age: u8
}

pub struct House {
    pub rooms: u8,
    pub inhabitants: Vec<Human>,
    address: String
}

pub enum EmotionalState {
    Happy,
    Content,
    Sad
}

impl House {
    pub fn getAddress(&self) -> String {
        self.address.clone()
    }

    pub fn changeAddress(&mut self, new: String) {
        self.address = new;
    }
}

pub fn move_house(mut house: House, target_address: String) -> House {
    print!("Moving the House from {}. ", house.getAddress());
    house.changeAddress(target_address);
    println!("Moved to {}", house.getAddress());
    house
}

pub fn move_in_one(mut house: House, mut humans: Vec<Human>) -> (House, Vec<Human>) {
    if let Some(human) = humans.pop() {
        house.inhabitants.push(human);
    }
    (house, humans)
}

pub fn house_information(house: House) {
    println!("Address: {}, Rooms: {}", house.getAddress(), house.rooms);
    let mut inhabitants = String::new();
    for inh in house.inhabitants {
        inhabitants += (inh.name + ", ").as_str();
    }
    println!("Inhabited by: {}", inhabitants)
}

pub fn evict_one(mut house: House) -> House {
    let _ = house.inhabitants.pop();
    house
}
