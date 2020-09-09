// useful things for dealing with gd level data
use crate::builtin::*;
use crate::compiler_types::*;
use std::collections::HashMap;
#[derive(Clone, PartialEq, Debug)]
pub struct GDObj {
    /*pub obj_id: u16,
    pub groups: Vec<Group>,
    pub target: Group,
    pub spawn_triggered: bool,*/
    pub func_id: usize,
    pub params: HashMap<u16, String>,
}

impl GDObj {
    pub fn context_parameters(&mut self, context: Context) -> GDObj {
        self.params.insert(57, context.start_group.id.to_string());
        self.params.insert(
            62,
            if context.spawn_triggered {
                String::from("1")
            } else {
                String::from("0")
            },
        );
        (*self).clone()
    }

    /*pub fn get_groups(&self) -> Vec<Group> {
        match self.params.get(&57) {
            Some(group_string) => group_string
                .split(".")
                .map(|x| Group {
                    id: x.parse::<u16>().unwrap(),
                })
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn obj_id(&self) -> &str {
        match self.params.get(&1) {
            Some(num) => num,
            None => "0",
        }
    }

    pub fn get_pos(&self) -> (f64, f64) {
        (
            match self.params.get(&2) {
                Some(x_pos) => x_pos.parse().unwrap(),
                None => 0.0,
            },
            match self.params.get(&3) {
                Some(y_pos) => y_pos.parse().unwrap(),
                None => 0.0,
            },
        )
    }*/
}

pub fn get_used_ids(ls: &String, globals: &mut Globals) {
    let objects = ls.split(";");
    for obj in objects {
        let props: Vec<&str> = obj.split(",").collect();
        for i in (0..props.len() - 1).step_by(2) {
            let key = props[i];
            let value = props[i + 1];

            match key {
                "57" => {
                    //GROUPS
                    let groups = value.split(".");
                    for g in groups {
                        let group = Group {
                            id: g.parse().unwrap(),
                        };
                        if !globals.closed_groups.contains(&group.id) {
                            (*globals).closed_groups.push(group.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

const START_HEIGHT: u16 = 10;
const MAX_HEIGHT: u16 = 40;

pub const SPWN_SIGNATURE_GROUP: &str = "1111";

pub fn serialize_triggers(objects: Vec<GDObj>) -> String {
    /*fn group_string(list: Vec<Group>) -> String {
        let mut string = String::new();
        for group in list.iter() {
            string += &(group.id.to_string() + ".");
        }
        string.pop();
        string
    }*/

    fn serialize_obj(mut trigger: GDObj) -> String {
        let mut obj_string = String::new();
        /*format!(
            "1,{},2,{},3,{},51,{}",
            trigger.obj_id,
            if trigger.spawn_triggered {
                x * 30 + 15
            } else {
                0
            },
            (80 - y) * 30 + 15,
            trigger.target.id
        );*/

        let spawned = match trigger.params.get(&62) {
            Some(s) => *s == String::from("1"),
            None => false,
        };

        /*let obj_id = match trigger.params.get(&1) {
            Some(s) => s,
            None => "0",
        };

        let target = match trigger.params.get(&51) {
            Some(s) => s,
            None => "0",
        };

        let keys = [1, 51];
        let values = [obj_id, trigger.target.id];

        for i in 0..2 {
            if !trigger.params.iter().any(|x| *x.0 == keys[i]) {
                obj_string += &format!("{},{},", keys[i].to_string(), values[i].to_string());
            }
        }

        if spawned {
            obj_string += "62,1,87,1,";
        }

        if !trigger.groups.is_empty() {
            obj_string += &(String::from("57,")
                + &group_string(trigger.groups)
                + "."
                + SPWN_SIGNATURE_GROUP
                + ",");
        }*/

        if spawned {
            obj_string += "87,1,";
        }

        match trigger.params.get_mut(&57) {
            Some(group_str) => (*group_str) += &format!(".{}", SPWN_SIGNATURE_GROUP),
            None => {
                trigger.params.insert(57, SPWN_SIGNATURE_GROUP.to_string());
            }
        };

        let mut param_list = trigger.params.iter().collect::<Vec<(&u16, &String)>>();

        param_list.sort_by(|a, b| (*a.0).cmp(b.0));

        for param in param_list {
            obj_string += &(param.0.to_string() + "," + &param.1 + ",");
        }
        //println!("{}", obj_string);
        obj_string + "108,1;" //linked group
    }

    let mut full_obj_string = String::new();

    for obj in objects {
        full_obj_string += &serialize_obj(obj)
    }
    full_obj_string
}

pub fn apply_fn_ids(func_ids: Vec<FunctionID>) -> Vec<GDObj> {
    //println!("{:?}", trigger);

    fn apply_fn_id(
        id_index: usize,
        func_ids: &Vec<FunctionID>,
        x_offset: u32,
        y_offset: u16,
    ) -> (Vec<GDObj>, u32) {
        let id = func_ids[id_index].clone();

        let mut objects = Vec::<GDObj>::new();

        let mut current_x = 0;
        /*if !id.obj_list.is_empty() {
            //add label
            obj_string += &format!(
                "1,914,2,{},3,{},31,{},32,0.5;",
                x_offset * 30 + 15,
                ((81 - START_HEIGHT) - y_offset) * 30 + 15,
                base64::encode(id.name.as_bytes())
            );
        }*/

        //add top layer
        let possible_height = MAX_HEIGHT - (START_HEIGHT + y_offset); //30 is max (TODO: case for if y_offset is more than 30)

        for (i, obj) in id.obj_list.iter().enumerate() {
            let y_pos = (i as u16) % possible_height + START_HEIGHT + y_offset;
            let x_pos = (i as f64 / possible_height as f64).floor() as u32 + x_offset;

            let mut new_obj = obj.clone();
            new_obj.params.insert(
                2,
                (if new_obj.params.get(&62) == Some(&String::from("1")) {
                    x_pos * 30 + 15
                } else {
                    0
                })
                .to_string(),
            );
            new_obj
                .params
                .insert(3, ((80 - y_pos) * 30 + 15).to_string());
            objects.push(new_obj);
        }
        if !id.obj_list.is_empty() {
            current_x += (id.obj_list.len() as f64 / possible_height as f64).floor() as u32 + 1;
        }

        //add all children
        for (i, func_id) in func_ids.iter().enumerate() {
            if func_id.parent == Some(id_index) {
                let (obj, new_length) =
                    apply_fn_id(i, func_ids, current_x + x_offset, y_offset + 1);
                objects.extend(obj);

                if new_length > 0 {
                    current_x += new_length + 1;
                }
            }
        }

        (objects, current_x)
    }

    let mut full_obj_list = Vec::<GDObj>::new();

    let mut current_x = 0;
    for (i, func_id) in func_ids.iter().enumerate() {
        if func_id.parent == None {
            let (objects, new_length) = apply_fn_id(i, &func_ids, current_x, 0);
            full_obj_list.extend(objects);

            current_x += new_length;
        }
    }
    full_obj_list
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

//<OLD>
/*
pub fn serialize_triggers_old(func_ids: Vec<FunctionID>) -> String {
    //println!("{:?}", trigger);
    fn group_string(list: Vec<Group>) -> String {
        let mut string = String::new();
        for group in list.iter() {
            string += &(group.id.to_string() + ".");
        }
        string.pop();
        string
    }

    fn serialize_obj(mut trigger: GDObj, x: u32, y: u16) -> String {
        let mut obj_string = String::new();

        let spawned = trigger.params.get(&62) == Some(&String::from("1"));

        if spawned {
            obj_string += "87,1,";
        }

        /*if !trigger.groups.is_empty() {
            obj_string += &(String::from("57,")
                + &group_string(trigger.groups)
                + "."
                + SPWN_SIGNATURE_GROUP
                + ",");
        }*/

        match trigger.params.get_mut(&2) {
            None => {
                trigger.params.insert(
                    2,
                    if spawned {
                        (x * 30 + 15).to_string()
                    } else {
                        "0".to_string()
                    },
                );
            }

            _ => (),
        };

        match trigger.params.get_mut(&3) {
            None => {
                trigger.params.insert(3, ((80 - y) * 30 + 15).to_string());
            }

            _ => (),
        };

        //((80 - y) * 30 + 15) as u32,)

        match trigger.params.get_mut(&57) {
            Some(group_str) => (*group_str) += &format!(".{}", SPWN_SIGNATURE_GROUP),
            None => {
                trigger.params.insert(57, SPWN_SIGNATURE_GROUP.to_string());
            }
        };

        for param in trigger.params {
            obj_string += &(param.0.to_string() + "," + &param.1 + ",");
        }
        obj_string + "108,1;" //linked group
    }

    fn serialize_func_id(
        id_index: usize,
        func_ids: Vec<FunctionID>,
        x_offset: u32,
        y_offset: u16,
    ) -> (String, u32) {
        let id = func_ids[id_index].clone();

        let mut obj_string = String::new();

        let mut current_x = 0;
        /*if !id.obj_list.is_empty() {
            //add label
            obj_string += &format!(
                "1,914,2,{},3,{},31,{},32,0.5;",
                x_offset * 30 + 15,
                ((81 - START_HEIGHT) - y_offset) * 30 + 15,
                base64::encode(id.name.as_bytes())
            );
        }*/

        //add top layer
        let possible_height = MAX_HEIGHT - (START_HEIGHT + y_offset); //30 is max (TODO: case for if y_offset is more than 30)

        for (i, obj) in id.obj_list.iter().enumerate() {
            let y_pos = (i as u16) % possible_height + START_HEIGHT + y_offset;
            let x_pos = (i as f64 / possible_height as f64).floor() as u32 + x_offset;
            obj_string += &serialize_obj(obj.clone(), x_pos, y_pos);
        }
        if !id.obj_list.is_empty() {
            current_x += (id.obj_list.len() as f64 / possible_height as f64).floor() as u32 + 1;
        }

        //add all children
        for (i, func_id) in func_ids.iter().enumerate() {
            if func_id.parent == Some(id_index) {
                let (child_string, new_length) =
                    serialize_func_id(i, func_ids.clone(), current_x + x_offset, y_offset + 1);
                obj_string += &child_string;
                if new_length > 0 {
                    current_x += new_length + 1;
                }
            }
        }
        (obj_string, current_x)
    }

    let mut full_obj_string = String::new();

    let mut current_x = 0;
    for (i, func_id) in func_ids.iter().enumerate() {
        if func_id.parent == None {
            let (obj_string, new_length) = serialize_func_id(i, func_ids.clone(), current_x, 0);
            full_obj_string += &obj_string;

            current_x += new_length;
        }
    }
    full_obj_string
}
*/
//</OLD>
use base64;
use libflate::{gzip, zlib};
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

use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use std::io::BufReader;

pub fn get_level_string(ls: String) -> String {
    //decrypting the savefile
    let xor = xor(ls.as_bytes().to_vec(), 11);
    let replaced = String::from_utf8(xor)
        .unwrap()
        .replace("-", "+")
        .replace("_", "/")
        .replace("\0", "");
    let b64 = base64::decode(replaced.as_str()).unwrap();
    let decoder = gzip::Decoder::new(&b64[..]).unwrap();

    //println!("{}", String::from_utf8(buf[..1000].to_vec()).unwrap());

    //getting level string

    let mut reader = Reader::from_reader(BufReader::new(decoder));
    reader.trim_text(true);

    let mut buf = Vec::new();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    let mut level_string = String::new();
    let mut k4_detected = false;
    loop {
        match reader.read_event(&mut buf) {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                if text == "k4" {
                    k4_detected = true
                } else if k4_detected {
                    level_string = text;
                    break;
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }

    /*let mut k4_detected = false;
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
    }*/
    //decrypting level string
    let ls_b64 = base_64_decrypt(
        level_string
            .replace("-", "+")
            .replace("_", "/")
            .replace("\0", "")
            .as_bytes()
            .to_vec(),
    );

    //println!("{}", String::from_utf8(ls_b64.clone()).unwrap());

    let mut ls_decoder = gzip::Decoder::new(&ls_b64[..]).unwrap();
    let mut ls_buf = Vec::new();
    ls_decoder.read_to_end(&mut ls_buf).unwrap();

    String::from_utf8(ls_buf).unwrap()
}

use quick_xml::Writer;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

pub fn encrypt_level_string(ls: String, old_ls: String, path: PathBuf) {
    let file_content = fs::read_to_string(path.clone()).unwrap();

    //decrypting the savefile
    let xor_encrypted = xor(file_content.as_bytes().to_vec(), 11);
    let replaced = String::from_utf8(xor_encrypted)
        .unwrap()
        .replace("-", "+")
        .replace("_", "/")
        .replace("\0", "");
    let b64 = base64::decode(replaced.as_str()).unwrap();
    let decoder = gzip::Decoder::new(&b64[..]).unwrap();

    //encrypt the ls
    //encrypting level string
    /*def encrypt(dls):
    fin = gzip.compress(dls)
    fin = base64.b64encode(fin)
    fin = fin.decode("utf-8").replace('+', '-').replace('/', '_')
    fin = 'H4sIAAAAAAAAC' + fin[13:]
    return(fin)*/

    //setting level string

    let mut reader = Reader::from_reader(BufReader::new(decoder));
    reader.trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut buf = Vec::new();

    let mut k4_detected = false;
    let mut done = false;
    let mut k2_detected = false;

    //println!("{}", old_ls);

    let full_ls = old_ls + &ls;

    loop {
        match reader.read_event(&mut buf) {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e.unescape_and_decode(&reader).unwrap();
                if k4_detected {
                    let encrypted_ls: String = {
                        let mut ls_encoder = gzip::Encoder::new(Vec::new()).unwrap();
                        ls_encoder.write_all(&full_ls.as_bytes()).unwrap();
                        let b64_encrypted =
                            base64::encode(&ls_encoder.finish().into_result().unwrap());
                        let fin = b64_encrypted.replace("+", "-").replace("/", "_");
                        "H4sIAAAAAAAAC".to_string() + &fin[13..]
                    };

                    assert!(writer
                        .write_event(Event::Text(BytesText::from_plain_str(&encrypted_ls)))
                        .is_ok());
                    done = true;
                    k4_detected = false;
                } else {
                    assert!(writer.write_event(Event::Text(e)).is_ok())
                }

                if k2_detected {
                    println!("Writing to level: {}", text);
                    k2_detected = false;
                }

                if !done && text == "k4" {
                    k4_detected = true
                }

                if !done && text == "k2" {
                    k2_detected = true
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }

        // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
        buf.clear();
    }
    let bytes = writer.into_inner().into_inner();
    //encrypt level save
    use std::io::Write;

    let mut encoder = zlib::Encoder::new(Vec::new()).unwrap();
    encoder.write_all(&bytes).unwrap();
    let compressed = encoder.finish().into_result().unwrap();
    use crc32fast::Hasher;

    let mut hasher = Hasher::new();
    hasher.update(&bytes);
    let checksum = hasher.finalize();

    let data_size = bytes.len() as u32;

    let mut with_signature = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x0b".to_vec();
    with_signature.extend(&compressed[2..compressed.len() - 4]);
    with_signature.extend(checksum.to_le_bytes().to_vec());
    with_signature.extend(data_size.to_le_bytes().to_vec());

    let encoded = base64::encode(&with_signature)
        .replace("+", "-")
        .replace("/", "_")
        .as_bytes()
        .to_vec();

    let fin = xor(encoded, 11);
    assert!(fs::write(path, fin).is_ok());
}
