use std::error::Error;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use base64::engine::general_purpose;
use base64::Engine;
use libflate::{gzip, zlib};
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};

use super::error::LevelError;

fn xor(data: Vec<u8>, key: u8) -> Vec<u8> {
    data.into_iter().map(|b| b ^ key).collect()
}

fn base_64_decrypt(encoded: Vec<u8>) -> Vec<u8> {
    let l = encoded.len();
    general_purpose::URL_SAFE
        .decode(
            String::from_utf8([encoded, b"=".repeat(l % 4)].concat())
                .unwrap()
                .as_str(),
        )
        .unwrap()
}

fn decrypt_savefile(mut sf: Vec<u8>) -> Result<Vec<u8>, LevelError> {
    if cfg!(target_os = "macos") {
        use aes::Aes256;
        use block_modes::block_padding::Pkcs7;
        use block_modes::{BlockMode, Ecb};

        type AesEcb = Ecb<Aes256, Pkcs7>;

        const IOS_KEY: &[u8] = &[
            0x69, 0x70, 0x75, 0x39, 0x54, 0x55, 0x76, 0x35, 0x34, 0x79, 0x76, 0x5D, 0x69, 0x73,
            0x46, 0x4D, 0x68, 0x35, 0x40, 0x3B, 0x74, 0x2E, 0x35, 0x77, 0x33, 0x34, 0x45, 0x32,
            0x52, 0x79, 0x40, 0x7B,
        ];

        // re-create cipher mode instance
        let cipher = AesEcb::new_from_slices(IOS_KEY, &[]).unwrap();

        Ok(cipher.decrypt(&mut sf)?.to_vec())
    } else {
        let xor = xor(sf.to_vec(), 11);
        let replaced = String::from_utf8_lossy(&xor).replace('\0', "");
        let b64 = general_purpose::URL_SAFE.decode(replaced.as_str())?;

        let mut decoder = gzip::Decoder::new(&b64[..])?;

        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;

        Ok(data)
    }
}

pub fn get_level_string(
    ls: Vec<u8>,
    level_name: Option<&String>,
) -> Result<(String, String), LevelError> {
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

    let mut level_name_out = String::new();

    loop {
        match reader.read_event() {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| LevelError::XMLErrorAtPosition {
                        position: reader.buffer_position(),
                        error: e,
                    })?
                    .to_string();

                if text == "k2" {
                    k2_detected = true;
                    if level_detected {
                        return Err(LevelError::UninitializedLevel);
                    }
                } else if k2_detected {
                    if let Some(level_name) = level_name {
                        if text == *level_name {
                            level_detected = true
                        }
                    } else {
                        level_detected = true
                    }

                    level_name_out = text.clone();

                    k2_detected = false
                }
                if level_detected && text == "k4" {
                    k4_detected = true
                } else if k4_detected {
                    level_string = text;
                    break;
                }
            },

            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => {
                return Err(LevelError::XMLErrorAtPosition {
                    position: reader.buffer_position(),
                    error: e,
                });
            },
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    if level_detected && !k4_detected {
        return Err(LevelError::UninitializedLevel);
    } else if !k4_detected {
        if let Some(level_name) = level_name {
            return Err(LevelError::UnknownLevel(level_name.to_string()));
        } else {
            return Err(LevelError::NoLevels);
        }
    }

    let ls_b64 = base_64_decrypt(level_string.replace('\0', "").as_bytes().to_vec());

    let mut ls_decoder = gzip::Decoder::new(&ls_b64[..])?;

    let mut ls_buf = Vec::new();
    ls_decoder.read_to_end(&mut ls_buf)?;

    Ok((String::from_utf8(ls_buf)?, level_name_out))
}

pub fn encrypt_level_string(
    ls: String,
    old_ls: String,
    path: PathBuf,
    level_name: &Option<String>,
) -> Result<(), LevelError> {
    let mut file = fs::File::open(path.clone())?;

    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

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

    let full_ls = old_ls + &ls;

    loop {
        match reader.read_event() {
            // unescape and decode the text event using the reader encoding
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| LevelError::XMLErrorAtPosition {
                        position: reader.buffer_position(),
                        error: e,
                    })?
                    .to_string();

                if k4_detected && level_detected {
                    let encrypted_ls: String = {
                        let mut ls_encoder = gzip::Encoder::new(Vec::new())?;

                        ls_encoder.write_all(full_ls.as_bytes())?;
                        let b64_encrypted =
                            general_purpose::URL_SAFE.encode(ls_encoder.finish().into_result()?);
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
                                //println!("Level: {}", text.bright_white().bold());
                            }
                        } else {
                            level_detected = true;
                            //println!("Level: {}", text.bright_white().bold());
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
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => {
                return Err(LevelError::XMLErrorAtPosition {
                    position: reader.buffer_position(),
                    error: e,
                });
            },
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }
    }
    let bytes = writer.into_inner().into_inner();
    // encrypt level save

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
        use crc32fast::Hasher;

        let mut encoder = zlib::Encoder::new(Vec::new())?;
        encoder.write_all(&bytes)?;
        let compressed = encoder.finish().into_result()?;

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        let checksum = hasher.finalize();

        let data_size = bytes.len() as u32;

        let mut with_signature = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\x0b".to_vec();
        with_signature.extend(&compressed[2..compressed.len() - 4]);
        with_signature.extend(checksum.to_le_bytes().to_vec());
        with_signature.extend(data_size.to_le_bytes().to_vec());

        let encoded = general_purpose::URL_SAFE
            .encode(&with_signature)
            .as_bytes()
            .to_vec();

        let fin = xor(encoded, 11);

        assert!(fs::write(path, fin).is_ok());
    }
    Ok(())
}
