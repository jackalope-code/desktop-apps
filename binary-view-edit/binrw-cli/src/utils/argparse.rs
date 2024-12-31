
impl ArgParse {

  pub enum NArgs {
    Value(u64),
    OptionalSingleValue(None),
    WildcardAnyVecValues(None),
    OnePlusVecValue(None)
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

  pub struct Argument {
    parameter: String,
    n_args, NArgs,
    action: Action,
    help: String
  }

  pub fn add_argument(&mut self, arg: Argument) {

  }

  pub fn parse_args(&mut self) {

  }

  pub fn get_opts(&mut self): Vec<String> {

  }
}