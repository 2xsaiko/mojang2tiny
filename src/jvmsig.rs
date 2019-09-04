use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub enum JvmType {
  Void,
  Boolean,
  Byte,
  Short,
  Char,
  Int,
  Long,
  Float,
  Double,
  Class(String),
  Array(Box<JvmType>),
}

impl JvmType {
  pub fn read(s: &mut String) -> JvmType {
    let first = s.remove(0);
    match first {
      'V' => JvmType::Void,
      'Z' => JvmType::Boolean,
      'B' => JvmType::Byte,
      'S' => JvmType::Short,
      'C' => JvmType::Char,
      'I' => JvmType::Int,
      'J' => JvmType::Long,
      'F' => JvmType::Float,
      'D' => JvmType::Double,
      'L' => {
        let end = s.find(';').expect("Malformed JVM type");
        let class_name: String = s.drain(0..end).collect();
        s.remove(0);
        JvmType::Class(class_name)
      }
      '[' => JvmType::Array(Box::new(JvmType::read(s))),
      _ => panic!("Malformed JVM type: {}", first),
    }
  }

  pub fn from_readable(s: &str) -> JvmType {
    if s.ends_with("[]") {
      #[allow(clippy::unnecessary_operation)] {
        JvmType::Array(Box::new(JvmType::from_readable(&s[..s.len() - 2])));
      }
    }
    match s {
      "void" => JvmType::Void,
      "boolean" => JvmType::Boolean,
      "byte" => JvmType::Byte,
      "short" => JvmType::Short,
      "char" => JvmType::Char,
      "int" => JvmType::Int,
      "long" => JvmType::Long,
      "float" => JvmType::Float,
      "double" => JvmType::Double,
      _ => JvmType::Class(s.replace(".", "/")),
    }
  }
}

impl Display for JvmType {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    match self {
      JvmType::Void => write!(f, "V"),
      JvmType::Boolean => write!(f, "Z"),
      JvmType::Byte => write!(f, "B"),
      JvmType::Short => write!(f, "S"),
      JvmType::Char => write!(f, "C"),
      JvmType::Int => write!(f, "I"),
      JvmType::Long => write!(f, "J"),
      JvmType::Float => write!(f, "F"),
      JvmType::Double => write!(f, "D"),
      JvmType::Class(s) => write!(f, "L{};", s),
      JvmType::Array(intern) => write!(f, "[{}", intern),
    }
  }
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub struct JvmSignature {
  params: Vec<JvmType>,
  result: JvmType,
}

impl JvmSignature {
  pub fn from(params: &[JvmType], result: &JvmType) -> JvmSignature {
    JvmSignature {
      params: params.to_vec(),
      result: result.clone(),
    }
  }
  
  pub fn from_jvm_sig(s: &str) -> JvmSignature {
    let mut s = s.to_owned();
    let mut params = Vec::new();
    if s.remove(0) != '(' { panic!("Expected '('"); }
    while s.chars().next().expect("Expected char, got EOF") != ')' {
      params.push(JvmType::read(&mut s));
    }
    s.remove(0);
    let result = JvmType::read(&mut s);
    if !s.is_empty() { panic!("Expected EOF"); }
    JvmSignature { params, result }
  }

  pub fn from_readable(result: &str, params: &str) -> JvmSignature {
    let result = JvmType::from_readable(result);
    let params: Vec<JvmType> = match params {
      "" => vec![],
      _ => params.split(',')
        .map(|s| JvmType::from_readable(s.trim()))
        .collect(),
    };
    JvmSignature { params, result }
  }

  pub fn params(&self) -> &[JvmType] { &self.params }
  pub fn result(&self) -> &JvmType { &self.result }
}

impl Display for JvmSignature {
  fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
    write!(f, "(")?;
    for param in self.params.iter() {
      write!(f, "{}", param)?;
    }
    write!(f, "){}", self.result)
  }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct JvmField {
  pub class: String,
  pub field_type: JvmType,
  pub name: String,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct JvmMethod {
  pub class: String,
  pub signature: JvmSignature,
  pub name: String,
}