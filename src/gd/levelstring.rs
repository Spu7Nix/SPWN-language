use std::io::Read;

use base64::{Engine, engine::general_purpose};
use libflate::{gzip, zlib};

fn xor(data: Vec<u8>, key: u8) -> Vec<u8> {
    data.into_iter().map(|b| b ^ key).collect()
}

fn base_64_decrypt(encoded: Vec<u8>) -> Vec<u8> {
    let l = encoded.len();
    general_purpose::URL_SAFE.decode(
        String::from_utf8([encoded, b"=".repeat(l % 4)].concat())
            .unwrap()
            .as_str(),
    )
    .unwrap()
}

use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
// use std::io::BufReader;

fn decrypt_savefile(mut sf: Vec<u8>) -> Result<Vec<u8>, String> {
    if cfg!(target_os = "macos") {
        use aes::Aes256;
        use block_modes::block_padding::Pkcs7;
        use block_modes::{BlockMode, Ecb};

        const IOS_KEY: &[u8] = &[
            0x69, 0x70, 0x75, 0x39, 0x54, 0x55, 0x76, 0x35, 0x34, 0x79, 0x76, 0x5D, 0x69, 0x73,
            0x46, 0x4D, 0x68, 0x35, 0x40, 0x3B, 0x74, 0x2E, 0x35, 0x77, 0x33, 0x34, 0x45, 0x32,
            0x52, 0x79, 0x40, 0x7B,
        ];

        type AesEcb = Ecb<Aes256, Pkcs7>;

        // re-create cipher mode instance
        let cipher = AesEcb::new_from_slices(IOS_KEY, &[]).unwrap();

        Ok(match cipher.decrypt(&mut sf) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}", e)),
        }
        .to_vec())
    } else {
        let xor = xor(sf.to_vec(), 11);
        let replaced = String::from_utf8_lossy(&xor)
            .replace('\0', "");
        let b64 = match general_purpose::URL_SAFE.decode(replaced.as_str()) {
            Ok(b) => b,
            Err(e) => return Err(format!("{}", e)),
        };
        let mut decoder = gzip::Decoder::new(&b64[..]).unwrap();
        let mut data = Vec::new();
        decoder.read_to_end(&mut data).unwrap();
        Ok(data)
    }
}

pub fn get_level_string(ls: Vec<u8>, level_name: Option<&String>) -> Result<String, String> {
    // decrypting the savefile
    let content = decrypt_savefile(ls)?;
    let string_content = String::from_utf8_lossy(&content);

    let mut reader = Reader::from_str(&string_content);
    reader.trim_text(true);

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    let mut level_string = String::new();
    let mut k4_detected = false;
    let mut k2_detected = false;
    let mut level_detected = false;

    loop {
        match reader.read_event() {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap().to_string();

                if text == "k2" {
                    k2_detected = true;
                    if level_detected {
                        return Err(
                            "Level is not initialized! Please open the level, place some objects, then save and quit to initialize the level."
                            .to_string()
                        );
                    }
                } else if k2_detected {
                    if let Some(level_name) = level_name {
                        if text == *level_name {
                            level_detected = true
                        }
                    } else {
                        level_detected = true
                    }
                    k2_detected = false
                }
                if level_detected && text == "k4" {
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
    }
    if level_detected && !k4_detected {
        return Err(
            "Level is not initialized! Please open the level, place some objects, then save and quit to initialize the level."
            .to_string()
        );
    } else if !k4_detected {
        if let Some(level_name) = level_name {
            return Err(format!("Level named \"{}\" was not found!", level_name));
        } else {
            return Err(
                "No level found! Please create a level for SPWN to operate on!".to_string(),
            );
        }
    }

    /*
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
    */

    // decrypting level string
    let ls_b64 = base_64_decrypt(
        level_string
            .replace('\0', "")
            .as_bytes()
            .to_vec(),
    );

    // println!("{}", String::from_utf8(ls_b64.clone()).unwrap());

    let mut ls_decoder = gzip::Decoder::new(&ls_b64[..]).unwrap();
    let mut ls_buf = Vec::new();
    ls_decoder.read_to_end(&mut ls_buf).unwrap();

    Ok(String::from_utf8(ls_buf).unwrap())
}

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use quick_xml::Writer;

pub fn encrypt_level_string(
    ls: String,
    old_ls: String,
    path: PathBuf,
    level_name: Option<String>,
) -> Result<(), String> {
    let mut file = fs::File::open(path.clone()).unwrap();
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).unwrap();

    // decrypting the savefile
    let content = decrypt_savefile(file_content)?;
    let string_content = String::from_utf8_lossy(&content);

    let mut reader = Reader::from_str(&string_content);
    reader.trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut k4_detected = false;
    let mut done = false;
    let mut k2_detected = false;
    let mut level_detected = false;

    // println!("{}", old_ls);

    let full_ls = old_ls + &ls;

    loop {
        match reader.read_event() {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap().to_string();
                if k4_detected && level_detected {
                    let encrypted_ls: String = {
                        let mut ls_encoder = gzip::Encoder::new(Vec::new()).unwrap();
                        ls_encoder.write_all(full_ls.as_bytes()).unwrap();
                        let b64_encrypted =
                            general_purpose::URL_SAFE.encode(ls_encoder.finish().into_result().unwrap());
                        "H4sIAAAAAAAAC".to_string() + &b64_encrypted[13..]
                    };

                    assert!(writer
                        .write_event(Event::Text(BytesText::new(&encrypted_ls)))
                        .is_ok());
                    done = true;
                    k4_detected = false;
                } else {
                    if k4_detected {
                        k4_detected = false;
                    }
                    assert!(writer.write_event(Event::Text(e)).is_ok());

                    if k2_detected {
                        if let Some(level_name) = &level_name {
                            if level_name == &text {
                                level_detected = true;
                                println!("Writing to level: {}", text);
                            }
                        } else {
                            level_detected = true;
                            println!("Writing to level: {}", text);
                        }

                        k2_detected = false;
                    }
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
    }
    let bytes = writer.into_inner().into_inner();
    // encrypt level save
    use std::io::Write;

    if cfg!(target_os = "macos") {
        use aes::Aes256;
        use block_modes::block_padding::Pkcs7;
        use block_modes::{BlockMode, Ecb};

        const IOS_KEY: &[u8] = &[
            0x69, 0x70, 0x75, 0x39, 0x54, 0x55, 0x76, 0x35, 0x34, 0x79, 0x76, 0x5D, 0x69, 0x73,
            0x46, 0x4D, 0x68, 0x35, 0x40, 0x3B, 0x74, 0x2E, 0x35, 0x77, 0x33, 0x34, 0x45, 0x32,
            0x52, 0x79, 0x40, 0x7B,
        ];

        type AesEcb = Ecb<Aes256, Pkcs7>;

        // re-create cipher mode instance
        let cipher = AesEcb::new_from_slices(IOS_KEY, &[]).unwrap();

        let fin = cipher.encrypt_vec(&bytes);
        assert!(fs::write(path, fin).is_ok());
    } else {
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

        let encoded = general_purpose::URL_SAFE.encode(&with_signature)
            .as_bytes()
            .to_vec();

        let fin = xor(encoded, 11);
        assert!(fs::write(path, fin).is_ok());
    }
    Ok(())
}
