pub struct Controller {
    pub controller1: u8,
    pub controller2: u8,
    pub shift_register_1: u8,
    pub shift_register_2: u8,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            controller1: 0,
            controller2: 0,
            shift_register_1: 0,
            shift_register_2: 0,
        }
    }

    pub fn reset_controllers(&mut self) {
        self.controller1 = 0;
        self.controller2 = 0;
    }
    pub fn right_p1(&mut self) {
        self.controller1 |= 0x01;
    }
    pub fn left_p1(&mut self) {
        self.controller1 |= 0x02;
    }
    pub fn down_p1(&mut self) {
        self.controller1 |= 0x04;
    }
    pub fn up_p1(&mut self) {
        self.controller1 |= 0x08;
    }
    pub fn start_p1(&mut self) {
        self.controller1 |= 0x10;
    }
    pub fn select_p1(&mut self) {
        self.controller1 |= 0x20;
    }
    pub fn b_p1(&mut self) {
        self.controller1 |= 0x40;
    }
    pub fn a_p1(&mut self) {
        self.controller1 |= 0x80;
    }
    pub fn right_p2(&mut self) {
        self.controller2 |= 0x01;
    }
    pub fn left_p2(&mut self) {
        self.controller2 |= 0x02;
    }
    pub fn down_p2(&mut self) {
        self.controller2 |= 0x04;
    }
    pub fn up_p2(&mut self) {
        self.controller2 |= 0x08;
    }
    pub fn start_p2(&mut self) {
        self.controller2 |= 0x10;
    }
    pub fn select_p2(&mut self) {
        self.controller2 |= 0x20;
    }
    pub fn b_p2(&mut self) {
        self.controller2 |= 0x40;
    }
    pub fn a_p2(&mut self) {
        self.controller2 |= 0x80;
    }
}
