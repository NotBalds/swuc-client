mod changelog;
mod search;

use {
    base64::prelude::{Engine, BASE64_STANDARD as b64},
    clap::{Arg, Command},
    ecies,
    serde::Deserialize,
    serde_json::json,
    std::fs,
    tungstenite::{self as ws, Message},
    url::Url,
};

/// User config struct
#[derive(Deserialize, Debug)]
struct User {
    pub url: String,
    pub uuid: String,
    pub secret: String,
    pub public: String,
}

/// The main client function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("swuc")
        .version("0.1.0")
        .about("Software Updates Checker helps you to get info about available updates")
        .arg(Arg::new("file").required(true))
        .get_matches();
    if let Some(path) = matches.get_one::<String>("file") {
        let data = fs::read_to_string(path)?;

        let user: User = serde_json::from_str(&data)?;

        client(user, vec!["cwe-client-cli", "android", "rustlang", "nginx"])?;
    }
    Ok(())
}

/// Function to connect to server using User struct
fn client(user: User, names: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // Parsing the url
    let url = Url::parse(&user.url)?;

    // Connecting to the server
    let (mut ws_stream, _) = ws::client(
        url.as_str(),
        std::net::TcpStream::connect(&*url.socket_addrs(|| None)?)?,
    )?;

    // Prepare names (base64 encoded each and join with |)
    let encoded_names: Vec<String> = names.iter().map(|n| b64.encode(n.as_bytes())).collect();
    let plaintext_names = encoded_names.join("|");

    // Encrypting data to send it
    let encrypted_data = ecies::encrypt(&b64.decode(&user.public)?, plaintext_names.as_bytes())
        .expect("Can't encrypt");

    // Creating request in JSON format
    let request = json!({
        "uuid": user.uuid,
        "raw": b64.encode(encrypted_data)
    });
    let encoded_request = b64.encode(request.to_string());

    // Sending request to server
    ws_stream.send(Message::Text(encoded_request.into()))?;

    // Reading answer from server
    let msg = ws_stream.read()?;
    if let Message::Text(resp) = msg {
        // Decoding && Decrypting
        let encrypted_resp = b64.decode(resp)?;
        let decrypted_bytes =
            ecies::decrypt(&b64.decode(&user.secret)?, &encrypted_resp).expect("Can't decrypt");
        let response = String::from_utf8(decrypted_bytes)?;
        println!("Decrypted response: {}", response);
    }

    Ok(())
}
