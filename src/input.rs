pub struct TextInput {
    pub value: String,
    pub cursor: usize,
}
impl TextInput {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
    }

    pub fn insert(&mut self, c: char) {
        if self.cursor > self.value.len() {
            self.cursor = self.value.len();
        }
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn delete(&mut self) {
        if self.cursor > 0 {
            self.value.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }
}
