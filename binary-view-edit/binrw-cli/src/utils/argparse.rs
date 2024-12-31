
mod argparse {
  use std::env;

  pub struct Argument {
    parameter: String,
    n_args: NArgs,
    action: Action,
    help: String
  }

  pub enum NArgs { 
    Value(u64),
    OptionalSingleValue,
    WildcardAnyVecValues,
    OnePlusVecValue
  }

  pub enum Value {
    String(String),
    Number(i64)
  }

  pub enum Action {
    StoreTrue,
    StoreFalse,
    Append,
    Count,
    StoreValue(Value),
    AppendValue(Value),
  }
  
  struct ArgParse {
    args: Vec<String>
  }
  
  impl ArgParse {
  
  
  
    pub fn new(&mut self) -> ArgParse {
      let args: Vec<String> = env::args().collect();
      ArgParse {
        args
      }
    }
  
    pub fn add_argument(&mut self, arg: Argument) {
  
    }
  
    pub fn parse_args(&mut self) {
  
    }
  
    // TODO: Implement
    // pub fn get_opts(&mut self): Vec<String> {
  
    // }
  }
}