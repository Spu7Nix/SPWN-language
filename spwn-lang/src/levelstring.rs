// useful things for dealing with gd level data
use crate::compiler_types::*;
use crate::native::*;

#[derive(Clone, PartialEq, Debug)]
pub struct GDObj {
    pub obj_id: u16,
    pub groups: Vec<Group>,
    pub target: Group,
    pub spawn_triggered: bool,
    pub x: u32,
    pub y: u16,
    pub params: Vec<(u16, String)>,
}

impl GDObj {
    pub fn context_parameters(&mut self, context: Context) -> GDObj {
        for g in context.added_groups.iter() {
            self.groups.push(*g);
        }
        self.spawn_triggered = context.spawn_triggered;
        (*self).clone()
    }
}

pub fn serialize_trigger(trigger: GDObj) -> String {
    //println!("{:?}", trigger);
    fn group_string(list: Vec<Group>) -> String {
        let mut string = String::new();
        for group in list.iter() {
            string += &(group.id.to_string() + ".");
        }
        string.pop();
        string
    }
    /*
    let mut obj_string = format!(
        "1,{},2,{},3,{},51,{}",
        trigger.obj_id, trigger.x, trigger.y, trigger.target.id
    );
    */

    let mut obj_string = String::new();

    let spawned = trigger.spawn_triggered && !trigger.params.iter().any(|x| x.0 == 62);

    let keys = [1, 2, 3, 51];
    let values = [
        trigger.obj_id as u32,
        if spawned { trigger.x } else { 0 },
        if spawned { trigger.y as u32 } else { 0 },
        trigger.target.id as u32,
    ];

    for i in 0..4 {
        if !trigger.params.iter().any(|x| x.0 == keys[i]) {
            obj_string += &format!("{},{},", keys[i].to_string(), values[i].to_string());
        }
    }

    if spawned {
        obj_string += "62,1,87,1,";
    }

    if !trigger.groups.is_empty() {
        obj_string += &(String::from("57,") + &group_string(trigger.groups) + ",");
    }

    for param in trigger.params {
        obj_string += &(param.0.to_string() + "," + &param.1 + ",");
    }

    obj_string + ";"
}
/* PYTHON CODE IM USING
def Xor(data,key):
    res = []
    for i in data:
        res.append(i^key)
    return bytearray(res).decode()

def Base64Decrypt(encodedData):
    while (len(encodedData) % 4 != 0):
        encodedData += "="
    encodedDataAsBytes = base64.b64decode(encodedData)
    return encodedDataAsBytes


def decrypt(ls):
    fin = ls.replace('-', '+').replace('_', '/').replace("\0", "")
    fin = Base64Decrypt(fin)
    fin = gzip.decompress(fin)
    return(fin)
*/

use base64;
use libflate::gzip;
use std::io::Read;

fn xor(data: Vec<u8>, key: u8) -> Vec<u8> {
    let mut new_data = Vec::new();

    for b in data {
        //let new_byte = u64::from(b).pow(key);
        new_data.push(b ^ key)
    }
    new_data
}
fn base_64_decrypt(encoded: Vec<u8>) -> Vec<u8> {
    let mut new_data = encoded;
    while new_data.len() % 4 != 0 {
        new_data.push(b'=')
    }
    base64::decode(String::from_utf8(new_data).unwrap().as_str()).unwrap()
}

use xmlparser;

pub fn get_level_string(ls: String) -> String {
    //decrypting the savefile
    let xor = xor(ls.as_bytes().to_vec(), 11);
    let replaced = String::from_utf8(xor)
        .unwrap()
        .replace("-", "+")
        .replace("_", "/");
    let b64 = base64::decode(replaced.as_str()).unwrap();
    let mut decoder = gzip::Decoder::new(&b64[..]).unwrap();
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf).unwrap();

    //getting level string
    let mut level_string = String::new();

    let mut k4_detected = false;
    for token in xmlparser::Tokenizer::from(String::from_utf8(buf).unwrap().as_str()) {
        if let xmlparser::Token::Text { text } = token.unwrap() {
            if k4_detected {
                level_string = text.as_str().to_string();
                break;
            }
            if text.as_str() == "k4" {
                k4_detected = true;
            }
        }
    }

    //decrypting level string
    let ls_b64 = base_64_decrypt(
        level_string
            .replace("-", "+")
            .replace("_", "/")
            .replace("\0", "")
            .as_bytes()
            .to_vec(),
    );

    let mut ls_decoder = gzip::Decoder::new(&ls_b64[..]).unwrap();
    let mut ls_buf = Vec::new();
    ls_decoder.read_to_end(&mut ls_buf).unwrap();

    String::from_utf8(ls_buf).unwrap()
}
