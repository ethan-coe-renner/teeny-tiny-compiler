use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Emitter {
    full_path: &'static str,
    header: String,
    code: String
}

impl Emitter {
    pub fn new(full_path: &'static str) -> Self {
	Emitter {
	    full_path,
	    header: String::new(),
	    code: String::new(),
	}
    }

    pub fn emit(&mut self, code: &str) {
	self.code += code;
    }

    pub fn emit_line(&mut self, code: &str) {
	self.code += code;
	self.code += "\n";
    }

    pub fn header_line(&mut self, code: &str) {
	self.header += code;
	self.header += "\n";
    }

    pub fn write_file(&self) {
	let path = Path::new(self.full_path);
	let display = path.display();

	let full_text = self.header.clone() + &self.code;

	let mut file = match File::create(&path) {
	    Err(why) => panic!("couldn't create {}: {}", display, why),
	    Ok(file) => file,
	};
	
	match file.write_all(full_text.as_bytes()) {
            Err(why) => panic!("couldn't write to {}: {}", display, why),
            Ok(_) => println!("successfully wrote to {}", display),
	}
    }
}
