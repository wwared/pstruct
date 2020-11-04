#[macro_use]
extern crate pest_derive;

pub mod parser;
pub mod renderer;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // TODO add old tests back
        assert_eq!(2 + 2, 4);
    }
}
