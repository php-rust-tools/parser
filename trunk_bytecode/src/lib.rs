pub type Register = usize;

pub struct Instruction {
    pub dest: Register,
    pub code: Code,
}

#[derive(Debug, PartialEq)]
pub enum Code {

}