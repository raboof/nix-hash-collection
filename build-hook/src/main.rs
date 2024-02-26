use libnixstore::{hash_path, query_references, sign_string, Radix::Base32};
use regex::Regex;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct OutputReport<'a> {
    output_path: &'a str,
    output_hash: String,
    output_sig: String,
}

fn read_env_var_or_panic(variable: &str) -> String {
    match env::var(variable) {
        Ok(v) => v,
        Err(_) => panic!("The {} variable is not set", variable),
    }
}

fn parse_drv_hash<'a>(drv_path: &'a str) -> &'a str {
    let re = Regex::new(r"\/nix\/store\/(.*)\.drv").unwrap();
    re.captures(drv_path).unwrap().get(1).unwrap().as_str()
}

fn fingerprint(out_path: &str) -> String {
    let _references = query_references(out_path);
    return "1;<out_path>;<narHash>;<narSize>;<references>".to_string();
}

#[tokio::main]
async fn main() -> Result<()> {
    libnixstore::init();
    let token = read_env_var_or_panic("HASH_COLLECTION_TOKEN");
    let secret_key = read_env_var_or_panic("HASH_COLLECTION_SECRET_KEY");
    let collection_server = read_env_var_or_panic("HASH_COLLECTION_SERVER");
    let out_paths = read_env_var_or_panic("OUT_PATHS");
    let drv_path = read_env_var_or_panic("DRV_PATH");
    let drv_ident = parse_drv_hash(&drv_path);

    println!(
        "Uploading hashes of build outputs for derivation {0} to {1}",
        drv_ident, collection_server
    );

    let client = reqwest::Client::new();
    let output_reports: Vec<_> = out_paths
        .split(" ")
        .map(|path| -> OutputReport {
          let hash = hash_path("sha256", Base32, path).unwrap();
          let fingerprint = fingerprint(path);
          let signature = sign_string(secret_key.as_str(), &fingerprint).expect("Failed to sign fingerprint");
          return OutputReport {
              output_path: path,
              output_hash: hash,
              output_sig: signature
          }})
        .collect();

    client
        .post(format!("{0}/report/{1}", collection_server, drv_ident))
        .bearer_auth(token)
        .json(&output_reports)
        .send()
        .await?;
    Ok(())
}
