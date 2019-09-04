use std::io::{BufRead, BufReader, Read};

use regex::Regex;

use crate::jvmsig::{JvmSignature, JvmType};

#[derive(Debug)]
pub struct MojangMap {
  pub entries: Vec<MojangMapEntry>,
}

#[derive(Debug)]
pub enum MojangMapEntry {
  Class { deobf_name: String, obf_name: String },
  Field { deobf_class: String, obf_class: String, deobf_type: JvmType, deobf_name: String, obf_name: String },
  Method { deobf_class: String, obf_class: String, deobf_sig: JvmSignature, deobf_name: String, obf_name: String },
}

impl MojangMap {
  pub fn load(reader: impl Read) -> MojangMap {
    let class_re = Regex::new(r"^([\w$.]+|[\w$.]+\.package-info) -> ([\w$.]+):$").unwrap();
    let field_re = Regex::new(r"^\s+([\w$.\[\]]+) ([\w$]+) -> ([\w$]+)$").unwrap();
    let method_re = Regex::new(r"^\s+([0-9]+:[0-9]+:)?([\w$.\[\]]+) ([\w$]+|<clinit>|<init>)\(([\w$.\[\],]*)\) -> ([\w$]+|<clinit>|<init>)$").unwrap();

    let mut reader = BufReader::new(reader);
    let mut buf = String::new();
    let mut entries = Vec::new();
    let mut last_class: Option<(String, String)> = None;
    loop {
      buf.clear();
      let size = reader.read_line(&mut buf).unwrap();
      buf.pop();
      if size == 0 { break; }
      if buf.starts_with('#') { continue; }
      if let Some(c) = class_re.captures(&buf) {
        let deobf_name = c.get(1).unwrap().as_str().replace('.', "/");
        let obf_name = c.get(2).unwrap().as_str().replace('.', "/");
        last_class = Some((deobf_name.clone(), obf_name.clone()));
        entries.push(MojangMapEntry::Class { deobf_name, obf_name });
      } else if let Some(c) = field_re.captures(&buf) {
        let deobf_type = c.get(1).unwrap().as_str();
        let deobf_name = c.get(2).unwrap().as_str().to_owned();
        let obf_name = c.get(3).unwrap().as_str().to_owned();
        let (deobf_class, obf_class) = last_class.as_ref().expect("Class not defined");
        entries.push(MojangMapEntry::Field {
          deobf_class: deobf_class.clone(),
          obf_class: obf_class.clone(),
          deobf_type: JvmType::from_readable(deobf_type),
          deobf_name,
          obf_name,
        });
      } else if let Some(c) = method_re.captures(&buf) {
        let deobf_type = c.get(2).unwrap().as_str();
        let deobf_name = c.get(3).unwrap().as_str().to_owned();
        let deobf_params = c.get(4).unwrap().as_str();
        let obf_name = c.get(5).unwrap().as_str().to_owned();
        let (deobf_class, obf_class) = last_class.as_ref().expect("Class not defined");
        entries.push(MojangMapEntry::Method {
          deobf_class: deobf_class.clone(),
          obf_class: obf_class.clone(),
          deobf_sig: JvmSignature::from_readable(deobf_type, deobf_params),
          deobf_name,
          obf_name,
        });
      } else {
        panic!("Syntax error, ignoring: {}", buf);
      }
    }
    MojangMap { entries }
  }

  pub fn empty() -> MojangMap {
    MojangMap { entries: vec![] }
  }

  pub fn combine(&mut self, mut other: MojangMap) {
    self.entries.append(&mut other.entries);
  }
}