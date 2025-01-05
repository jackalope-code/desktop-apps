

use std::env;
use std::collections::HashMap;
use derive_builder::Builder;

#[derive(Debug)]
pub struct Argument {
  pub parameter: String,
  pub n_args: NArgs,
  pub action: Action,
  pub help: String
}

#[derive(Debug)]
pub enum NArgs { 
  Number(i32),
  OptionalSingleValue,
  WildcardAnyListValues,
  OnePlusListValue
}

#[derive(Debug)]
pub enum Value {
  ArgString(String),
  Number(i64),
  Array(Box<Value>)
}

#[derive(Debug)]
pub enum Action {
  Store,
  StoreTrue,
  StoreFalse,
  Append,
  Count,
  StoreConst(Value),
  AppendConst(Value),
}

#[derive(Builder, Debug)]
pub struct ArgParse {
  pub program_name: String,
  pub description: String,
  pub epilog: String,
  #[builder(setter(skip))]
  sys_args: Vec<String>,
  #[builder(setter(skip))]
  arg_map: HashMap<String, Argument>,
}

#[derive(Debug)]
pub struct ArgParseInstanceVars {
  pub program_name: String,
  pub description: String,
  pub epilog: String,
}

impl ArgParse {

  // TODO: Would some kind of builder pattern be better for constructing the initial part from a struct?
  pub fn new(instance_var_args: ArgParseInstanceVars) -> ArgParse {
    let sys_args: Vec<String> = env::args().collect();
    let ArgParseInstanceVars {program_name, description, epilog} = instance_var_args;
    let args = HashMap::new();
    ArgParse {
      program_name,
      description,
      epilog,
      sys_args,
      arg_map: args
    }
  }

  pub fn add_argument(&mut self, arg: Argument) {
    self.arg_map.insert(arg.parameter.clone(), arg);
  }

  
  pub fn get_usage_string(&mut self) -> String {
    let header = format!("{} {}\n", self.program_name, self.description);
    let usage = format!("Usage: {}", self.program_name);
    let mut required_commands = Vec::<String>::new();
    // TODO: idk how to splice this later
    required_commands.push(usage);
    for (command, arg) in self.arg_map.iter() {
      required_commands.push(command.to_string());
      match &arg.n_args {
        NArgs::Number(n_args) => {
          // const n_args = Value::ArgString::(arg.n_args);
          println!("n_args: {}", n_args);
          // for i in 0..n_args.parse::<i32>().unwrap() {
          for i in 0..*n_args {
            required_commands.push(format!("arg_{}", i));
          }
        },
        // Value::Number()
        // Value::Array()
        NArgs::OptionalSingleValue => {

        },
        NArgs::WildcardAnyListValues => {

        },
        NArgs::OnePlusListValue => {

        }
      }
    }
    header + &required_commands.join(" ") + "\n" + &self.epilog
  }

  // Value(u64),
  // OptionalSingleValue,
  // WildcardAnyListValues,
  // OnePlusListValue
  
  pub fn print_usage_string(&mut self) {
    println!("{}", self.get_usage_string());
  }
  
  pub fn get_full_command_help(&mut self) {
    for (command, arg) in self.arg_map.iter() {
      println!("COMMAND: {} | ARG: {:?}", command, arg);
    }

  }

  pub fn parse_args(&mut self) -> Vec<(String, Value)> {
    for i in 0..self.sys_args.len() {
      let current_token = &self.sys_args[i];
      if self.arg_map.contains_key(current_token.as_str()) {
        println!("FOUND COMMAND TOKEN: {}", current_token);
        println!("DATA: {:?}", self.arg_map.get(current_token));
      }
    }
    return vec![("--dummy-file-option".to_string(), Value::ArgString("dummy_file_dest".to_string()))];
  }

  pub fn add_subparser(&mut self, subparser_path: String) {
    
  }

  // TODO: Implement
  // pub fn get_opts(&mut self): Vec<String> {

  // }
}