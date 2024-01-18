#![allow(unused)]

use serde::{Serialize,Deserialize};
use serde_json::{json, Value};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

#[derive(Serialize)]
enum Data {
    Unencrypted(Vec<u8>),
    Encrypted(String)
}

#[derive(Deserialize)]
struct Decrypted {
    filename: String, 
    data: Vec<u8>
}
fn main() {
    let help_message = "Usage : filecrypt [-e] [-d] [-o] [-k]\n\nOptions : \n\t-e\t\tEncrypt file -> filecrypt -e file.<extension> <filename> -k <key> -o <output file>\n\t-d\t\tDecrypt file -> filecrypt -d file.<extension> <filename> -k <key> -o <output file>\n\t--no-backup\tStore the backup of the files in the .filecrypt folder in the current directory";
    let arguments = std::env::args().collect::<Vec<String>>();
    if !((arguments.len() == 7) | (arguments.len() == 8)){
        eprintln!("{}", help_message);
        std::process::exit(1);
    }
    println!("[+] Filecrypt Started");


      
    let backup:bool;
    let filename:&str;
    let output_filename:&str;
    let encrypt:bool;
    let key:&str;

    if arguments.len() == 8 {
        if arguments.contains(&"--no-backup".to_string()) {
            backup = false;
        }else {
            backup = true ;
            println!("backup set to true")
        }
    }else {
        backup = true;
    }
 
    
    if (&arguments[1] == "-e") && (&arguments[3] == "-k") && (&arguments[5] == "-o") {
        if std::path::Path::new(&arguments[2]).exists(){
            filename = &arguments[2];
            output_filename = &arguments[6];
            encrypt = true;
            key = &arguments[4];
        } else {
            eprintln!("the file {} does not exists\nExiting...", &arguments[2]);
            std::process::exit(1);
        }
    } else if &arguments[1] == "-d" && &arguments[3] == "-k" && &arguments[5] == "-o" {
        if std::path::Path::new(&arguments[2]).exists() {
            filename = &arguments[2];
            key = &arguments[4];
            output_filename = &arguments[6];
            encrypt = false;

        } else {
            eprintln!("The file {} does not exists\nExiting", &arguments[2]);
            std::process::exit(1);
        }
    } else {
        eprintln!("Invalid arguments\nExiting...");
        std::process::exit(1);
    }
    if encrypt {
        let file_contents = get_contents(filename);
        if let Ok(data) = file_contents {
            //encrypts the string and saves it into a file
            encrypt_decrypt(encrypt, Data::Unencrypted(data), key, filename, output_filename, backup);
        }else if let Err(e) = file_contents {
            eprintln!("Error while reading file {filename}\n error : {e}");
        }        
    } else {
        let file_contents = get_str_contents(filename);
        if let Ok(data) = file_contents {
            encrypt_decrypt(encrypt, Data::Encrypted(data), key, filename, output_filename, backup);
        } else if let Err(e) = file_contents{
            if e.kind() == std::io::ErrorKind::InvalidData {
                eprintln!("[\\\\]The file you want to decrypt does not seem to be encrypted by filecrypt\nExiting...");
            }
        }
    }
    
}
// Converting the file data to a json is not necessary. found it on the way
fn encrypt_decrypt(encrypt:bool,data:Data, key:&str, filename:&str, rfilename:&str, if_backup:bool) { 
    if encrypt {
        match &data {
            Data::Unencrypted(vec)  => {
                let json = json!({
                    "filename" : filename, 
                    "data" : vec
                });
                let encrypted = new_magic_crypt!(&key, 256).encrypt_to_base64(&json.to_string());
                println!("[\\] Data Encrypted\n[\\] Writing Data...");
                let result = write_to_file(rfilename, encrypted);
                if let Ok(path) =  result{
                    println!("[+] Encrypted file stored successfully at {}", path)
                }else if let Err(e) = result {
                    eprintln!("Error while saving encrypted file\n error : {e}")
                }
            },
            _ => {}
        }
        
        save_to_json(filename, key, if_backup);
    } else {
        match &data {
            Data::Encrypted(string) => {
                println!("[//] Decrypting data...");
                let decrypted_string = new_magic_crypt!(key, 256).decrypt_base64_to_string(&string);
                if let Ok(decrypted) = decrypted_string {
                    println!("[+] Done");
                    let json_data:Result<Decrypted, serde_json::Error> = serde_json::from_str(&decrypted);
                    if let Ok(json_data) = json_data {
                        let file_data = json_data.data;
                        println!("Writing data to file...");
                        let res = std::fs::write(rfilename, file_data);
                        if let Ok(()) = res {
                            println!("[+] Done");
                            let path = std::fs::canonicalize(rfilename);
                            match path {
                                Ok(filepath) => {println!("File successfully decrypted at {}", filepath.to_str().unwrap_or_else(|| rfilename))}
                                Err(_) => println!("File successfully decrypted")
                            }
                        }else {
                            eprintln!("Error while writing data to file")
                        }
                    } else if let Err(e) = json_data {
                        eprintln!("Error while loading json\nError: {:?} ", e);
                    }
                } else if let Err(e) = decrypted_string {
                    eprintln!("Error while decrypting file \nWrong Password~!\nError: {}", e);
                }
            },
            _ => {}
        }
        
    }
} 

fn get_contents(filename:&str) -> Result<Vec<u8>, std::io::Error> {
    std::fs::read(filename)
}
fn get_str_contents(filename:&str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(filename)
}
fn save_to_json(filename:&str, key:&str, if_backup:bool) {
    if if_backup {
        let raw_json = std::fs::read_to_string(".filecrypt/filecrypt.json");
        if let Ok(json_data) = &raw_json {
            let json_data:Result<Value, serde_json::Error> = serde_json::from_str(&json_data);
            if let Ok(Value::Object(parsed_map)) = json_data {
                let mut data:std::collections::HashMap<String, Value> = parsed_map.into_iter().collect();
                data.insert(filename.to_string(), Value::String(key.to_string()));
                let json_data = serde_json::to_string(&data);
                if let Ok(json_data) = json_data {
                    match std::fs::write(".filecrypt/filecrypt.json", json_data) {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error while writingbackup to json file!\nError:{}", e)
                    }
                }else if let Err(e) = json_data {
                    eprintln!("Error while converting json to string \njson:{:?}\nerror:{}", &data, e);
                }
            } else if let Err(e) = &json_data {
                eprintln!("Error while converting string to json\njson:{:?},\nError:{}", &json_data, e );
            }
        } else if let Err(e) = raw_json {
            if e.kind() == std::io::ErrorKind::NotFound {
                if !(std::path::Path::new(".filecrypt").is_dir()){
                    match std::fs::create_dir(".filecrypt") {
                        Ok(_) => {},
                        Err(e) => eprintln!("Error while creating backup directory \nerror: {}", e)
                    }
                }                
                match std::fs::File::create("./.filecrypt/filecrypt.json") {
                    Ok(_) => {
                        match std::fs::write("./.filecrypt/filecrypt.json", b"{}") {
                            Ok(_) => {save_to_json(filename, key, if_backup)},
                            Err(e) => eprintln!("Error while writing to backup file\nerror: {}", e)
                        }
                    },
                    Err(e) => eprintln!("Error while creating backup files\nerror: {}", e) 
                }
            } else {
                eprintln!("error while creating backup file \n{e}");
            }
        } 
        
        }
}


pub fn write_to_file(filename:&str, data:String) -> Result<String, std::io::Error>{
    std::fs::write(filename, &data)?;
    match std::fs::canonicalize(filename) {
        Ok(path) => Ok(path.to_str().unwrap().to_string()),
        Err(_) => Ok(filename.to_string())
    }

}

