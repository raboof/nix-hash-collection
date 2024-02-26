use libnixstore::{query_path_info, query_references, sign_string, Radix::Base32};
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

fn fingerprint(out_path: &str, nar_hash: &str, size: u64) -> String {
    let references = query_references(out_path).expect("Query references").join(",");
    let fingerprint = format!("1;{out_path};{nar_hash};{size};{references}").to_string();
    return fingerprint;
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
          let info = query_path_info(path, Base32).unwrap();
          let hash = info.narhash;
          let size = info.size;
          let fingerprint = fingerprint(path, &hash, size);
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
